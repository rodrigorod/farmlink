package handlers

import (
	"encoding/xml"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestModDescXMLParsing(t *testing.T) {
	sampleXML := `<?xml version="1.0" encoding="utf-8" standalone="no" ?>
<modDesc descVersion="94">
    <author>SMI Modding Team</author>
    <version>1.2.0.0</version>
    <title>
        <en>John Deere 7R Series 2024</en>
        <fr>John Deere Série 7R 2024</fr>
    </title>
</modDesc>`

	var desc ModDescXML
	if err := xml.Unmarshal([]byte(sampleXML), &desc); err != nil {
		t.Fatalf("unexpected xml unmarshal error: %v", err)
	}

	if desc.Author != "SMI Modding Team" {
		t.Errorf("expected author 'SMI Modding Team', got '%s'", desc.Author)
	}

	if desc.Version != "1.2.0.0" {
		t.Errorf("expected version '1.2.0.0', got '%s'", desc.Version)
	}

	if desc.Title.String() != "John Deere 7R Series 2024" {
		t.Errorf("expected title 'John Deere 7R Series 2024', got '%s'", desc.Title.String())
	}
}

func TestModDescXMLPlainTitle(t *testing.T) {
	sampleXML := `<?xml version="1.0" encoding="utf-8" standalone="no" ?>
<modDesc descVersion="94">
    <author>Giants</author>
    <version>2.0.0.0</version>
    <title>Courseplay FS25</title>
</modDesc>`

	var desc ModDescXML
	if err := xml.Unmarshal([]byte(sampleXML), &desc); err != nil {
		t.Fatalf("unexpected xml unmarshal error: %v", err)
	}

	if desc.Title.String() != "Courseplay FS25" {
		t.Errorf("expected title 'Courseplay FS25', got '%s'", desc.Title.String())
	}
}

func TestHealthHandler(t *testing.T) {
	req := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	w := httptest.NewRecorder()

	Health(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected status 200, got %d", w.Code)
	}

	contentType := w.Header().Get("Content-Type")
	if contentType != "application/json" {
		t.Errorf("expected application/json, got %s", contentType)
	}
}
