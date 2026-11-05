package handlers

import (
	"encoding/json"
	"net/http"
	"time"
)

// HealthResponse represents the ping response payload.
type HealthResponse struct {
	Status    string    `json:"status"`
	Timestamp time.Time `json:"timestamp"`
	Service   string    `json:"service"`
}

// Health handles the health check endpoint.
func Health(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(HealthResponse{
		Status:    "ok",
		Timestamp: time.Now().UTC(),
		Service:   "farmlink-backend",
	})
}
