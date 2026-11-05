package handlers

import (
	"bytes"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestServerManifestInvalidUUID(t *testing.T) {
	handler := NewServerHandler(nil, nil)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/servers/invalid-uuid/manifest", nil)
	req.SetPathValue("id", "invalid-uuid")
	rec := httptest.NewRecorder()

	handler.Manifest(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Errorf("expected 400 Bad Request for invalid uuid, got %d", rec.Code)
	}
}

func TestServerToggleModInvalidParams(t *testing.T) {
	handler := NewServerHandler(nil, nil)

	// 1. Invalid server id
	req := httptest.NewRequest(http.MethodPatch, "/api/v1/servers/not-a-uuid/mods/123", bytes.NewBufferString(`{"is_active":false}`))
	req.SetPathValue("id", "not-a-uuid")
	req.SetPathValue("mod_id", "d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44")
	rec := httptest.NewRecorder()

	handler.ToggleMod(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Errorf("expected 400 for bad server id, got %d", rec.Code)
	}

	// 2. Invalid mod id
	req2 := httptest.NewRequest(http.MethodPatch, "/api/v1/servers/b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22/mods/not-uuid", bytes.NewBufferString(`{"is_active":false}`))
	req2.SetPathValue("id", "b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22")
	req2.SetPathValue("mod_id", "not-uuid")
	rec2 := httptest.NewRecorder()

	handler.ToggleMod(rec2, req2)

	if rec2.Code != http.StatusBadRequest {
		t.Errorf("expected 400 for bad mod id, got %d", rec2.Code)
	}
}
