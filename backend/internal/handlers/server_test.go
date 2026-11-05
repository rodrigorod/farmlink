package handlers

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
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

func TestGenerateInviteCode(t *testing.T) {
	code, err := generateInviteCode()
	if err != nil {
		t.Fatalf("unexpected error generating invite code: %v", err)
	}
	if !strings.HasPrefix(code, "FLK-") {
		t.Errorf("expected invite code to start with 'FLK-', got %s", code)
	}
	if len(code) != 10 { // "FLK-" + 6 chars = 10
		t.Errorf("expected length 10, got %d (%s)", len(code), code)
	}
}

func TestServerCreateValidation(t *testing.T) {
	handler := NewServerHandler(nil, nil)

	// 1. Unauthenticated -> 401
	req1 := httptest.NewRequest(http.MethodPost, "/api/v1/servers", bytes.NewBufferString(`{"name":"My Farm"}`))
	rec1 := httptest.NewRecorder()
	handler.Create(rec1, req1)
	if rec1.Code != http.StatusUnauthorized {
		t.Errorf("expected 401 Unauthorized for unauthenticated create, got %d", rec1.Code)
	}

	// Helper to attach valid user claims
	withUser := func(req *http.Request) *http.Request {
		claims := &UserClaims{UserID: "11111111-1111-1111-1111-111111111111", Username: "Farmer", Role: "player"}
		return req.WithContext(context.WithValue(req.Context(), userContextKey, claims))
	}

	// 2. Invalid JSON -> 400
	req2 := withUser(httptest.NewRequest(http.MethodPost, "/api/v1/servers", bytes.NewBufferString(`{invalid json`)))
	rec2 := httptest.NewRecorder()
	handler.Create(rec2, req2)
	if rec2.Code != http.StatusBadRequest {
		t.Errorf("expected 400 for invalid json, got %d", rec2.Code)
	}

	// 3. Empty name -> 400
	req3 := withUser(httptest.NewRequest(http.MethodPost, "/api/v1/servers", bytes.NewBufferString(`{"name":"   "}`)))
	rec3 := httptest.NewRecorder()
	handler.Create(rec3, req3)
	if rec3.Code != http.StatusBadRequest {
		t.Errorf("expected 400 for empty name, got %d", rec3.Code)
	}
}

func TestServerJoinValidation(t *testing.T) {
	handler := NewServerHandler(nil, nil)

	// 1. Unauthenticated -> 401
	req1 := httptest.NewRequest(http.MethodPost, "/api/v1/servers/join", bytes.NewBufferString(`{"invite_code":"FLK-TEST"}`))
	rec1 := httptest.NewRecorder()
	handler.Join(rec1, req1)
	if rec1.Code != http.StatusUnauthorized {
		t.Errorf("expected 401 Unauthorized for unauthenticated join, got %d", rec1.Code)
	}

	withUser := func(req *http.Request) *http.Request {
		claims := &UserClaims{UserID: "11111111-1111-1111-1111-111111111111", Username: "Farmer", Role: "player"}
		return req.WithContext(context.WithValue(req.Context(), userContextKey, claims))
	}

	// 2. Invalid JSON -> 400
	req2 := withUser(httptest.NewRequest(http.MethodPost, "/api/v1/servers/join", bytes.NewBufferString(`{bad-json`)))
	rec2 := httptest.NewRecorder()
	handler.Join(rec2, req2)
	if rec2.Code != http.StatusBadRequest {
		t.Errorf("expected 400 for invalid json, got %d", rec2.Code)
	}

	// 3. Empty invite code -> 400
	req3 := withUser(httptest.NewRequest(http.MethodPost, "/api/v1/servers/join", bytes.NewBufferString(`{"invite_code":"   "}`)))
	rec3 := httptest.NewRecorder()
	handler.Join(rec3, req3)
	if rec3.Code != http.StatusBadRequest {
		t.Errorf("expected 400 for empty invite code, got %d", rec3.Code)
	}
}
