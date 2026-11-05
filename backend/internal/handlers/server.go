package handlers

import (
	"crypto/rand"
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"time"

	"farmlink/backend/internal/storage"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// ServerHandler manages server listing, manifests, and mod associations.
type ServerHandler struct {
	db      *pgxpool.Pool
	storage *storage.Client
}

// NewServerHandler constructs a ServerHandler.
func NewServerHandler(db *pgxpool.Pool, storage *storage.Client) *ServerHandler {
	return &ServerHandler{db: db, storage: storage}
}

// ServerDTO is the serialized server output.
type ServerDTO struct {
	ID         string    `json:"id"`
	Name       string    `json:"name"`
	IPAddress  string    `json:"ip_address"`
	Port       int       `json:"port"`
	InviteCode string    `json:"invite_code"`
	OwnerID    string    `json:"owner_id"`
	CreatedAt  time.Time `json:"created_at"`
	ModCount   int       `json:"mod_count"`
}

// ManifestModDTO defines the mod schema consumed by the Rust client.
type ManifestModDTO struct {
	ID            string `json:"id"`
	ModName       string `json:"mod_name"`
	FileName      string `json:"file_name"`
	Version       string `json:"version"`
	HashSHA256    string `json:"hash_sha256"`
	FileSizeBytes int64  `json:"file_size_bytes"`
	DownloadURL   string `json:"download_url"`
	IsActive      bool   `json:"is_active"`
}

// ManifestResponse represents the full manifest returned to the Rust companion app.
type ManifestResponse struct {
	ServerID       string           `json:"server_id"`
	ServerName     string           `json:"server_name"`
	ModpackVersion string           `json:"modpack_version"`
	GeneratedAt    time.Time        `json:"generated_at"`
	TotalMods      int              `json:"total_mods"`
	TotalSizeBytes int64            `json:"total_size_bytes"`
	Mods           []ManifestModDTO `json:"mods"`
}

// List returns all servers accessible to the user.
func (h *ServerHandler) List(w http.ResponseWriter, r *http.Request) {
	claims, _ := UserFromContext(r.Context())

	var query string
	var args []interface{}

	if claims != nil && claims.Role != "admin" {
		userUUID, err := uuid.Parse(claims.UserID)
		if err == nil {
			query = `SELECT s.id, s.name, COALESCE(s.ip_address, ''), s.port, s.invite_code, s.owner_id, s.created_at,
					        COUNT(sm.mod_id) FILTER (WHERE sm.is_active = true) as mod_count
					 FROM servers s
					 LEFT JOIN server_mods sm ON s.id = sm.server_id
					 WHERE s.owner_id = $1 OR EXISTS (SELECT 1 FROM server_members sm2 WHERE sm2.server_id = s.id AND sm2.user_id = $1)
					 GROUP BY s.id
					 ORDER BY s.created_at DESC`
			args = append(args, userUUID)
		}
	}

	if query == "" {
		query = `SELECT s.id, s.name, COALESCE(s.ip_address, ''), s.port, s.invite_code, s.owner_id, s.created_at,
				        COUNT(sm.mod_id) FILTER (WHERE sm.is_active = true) as mod_count
				 FROM servers s
				 LEFT JOIN server_mods sm ON s.id = sm.server_id
				 GROUP BY s.id
				 ORDER BY s.created_at DESC`
	}

	rows, err := h.db.Query(r.Context(), query, args...)
	if err != nil {
		http.Error(w, `{"error":"fetching servers"}`, http.StatusInternalServerError)
		return
	}
	defer rows.Close()

	servers := make([]ServerDTO, 0)
	for rows.Next() {
		var s ServerDTO
		var ownerID uuid.UUID
		var serverID uuid.UUID
		if err := rows.Scan(&serverID, &s.Name, &s.IPAddress, &s.Port, &s.InviteCode, &ownerID, &s.CreatedAt, &s.ModCount); err != nil {
			http.Error(w, `{"error":"scanning server row"}`, http.StatusInternalServerError)
			return
		}
		s.ID = serverID.String()
		s.OwnerID = ownerID.String()
		servers = append(servers, s)
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(servers)
}

// Manifest returns the active modpack manifest for a specific server (used by the Rust companion client).
func (h *ServerHandler) Manifest(w http.ResponseWriter, r *http.Request) {
	serverIDStr := r.PathValue("id")
	serverID, err := uuid.Parse(serverIDStr)
	if err != nil {
		http.Error(w, `{"error":"invalid server id"}`, http.StatusBadRequest)
		return
	}

	var serverName string
	err = h.db.QueryRow(r.Context(), "SELECT name FROM servers WHERE id = $1", serverID).Scan(&serverName)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			http.Error(w, `{"error":"server not found"}`, http.StatusNotFound)
			return
		}
		http.Error(w, `{"error":"fetching server"}`, http.StatusInternalServerError)
		return
	}

	rows, err := h.db.Query(r.Context(),
		`SELECT m.id, m.mod_name, m.file_name, m.version, m.hash_sha256, m.file_size_bytes, m.s3_key, sm.is_active
		 FROM mods m
		 INNER JOIN server_mods sm ON m.id = sm.mod_id
		 WHERE sm.server_id = $1 AND sm.is_active = true
		 ORDER BY m.file_name ASC`,
		serverID,
	)
	if err != nil {
		http.Error(w, `{"error":"fetching server mods"}`, http.StatusInternalServerError)
		return
	}
	defer rows.Close()

	mods := make([]ManifestModDTO, 0)
	var totalSizeBytes int64

	for rows.Next() {
		var m ManifestModDTO
		var modID uuid.UUID
		var s3Key string
		if err := rows.Scan(&modID, &m.ModName, &m.FileName, &m.Version, &m.HashSHA256, &m.FileSizeBytes, &s3Key, &m.IsActive); err != nil {
			http.Error(w, `{"error":"scanning mod row"}`, http.StatusInternalServerError)
			return
		}
		m.ID = modID.String()
		m.DownloadURL = h.storage.GetURL(s3Key)
		totalSizeBytes += m.FileSizeBytes
		mods = append(mods, m)
	}

	manifest := ManifestResponse{
		ServerID:       serverID.String(),
		ServerName:     serverName,
		ModpackVersion: "v2.4", // shortcut: static modpack version tag, upgrade: store modpack version column in servers
		GeneratedAt:    time.Now().UTC(),
		TotalMods:      len(mods),
		TotalSizeBytes: totalSizeBytes,
		Mods:           mods,
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(manifest)
}

// RemoveMod unlinks a mod from a server modpack.
func (h *ServerHandler) RemoveMod(w http.ResponseWriter, r *http.Request) {
	serverID, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		http.Error(w, `{"error":"invalid server id"}`, http.StatusBadRequest)
		return
	}

	modID, err := uuid.Parse(r.PathValue("mod_id"))
	if err != nil {
		http.Error(w, `{"error":"invalid mod id"}`, http.StatusBadRequest)
		return
	}

	claims, ok := UserFromContext(r.Context())
	if !ok {
		http.Error(w, `{"error":"unauthorized"}`, http.StatusUnauthorized)
		return
	}

	var ownerID uuid.UUID
	err = h.db.QueryRow(r.Context(), "SELECT owner_id FROM servers WHERE id = $1", serverID).Scan(&ownerID)
	if err != nil || ownerID.String() != claims.UserID {
		http.Error(w, `{"error":"forbidden: only the server owner can remove mods"}`, http.StatusForbidden)
		return
	}

	result, err := h.db.Exec(r.Context(),
		"DELETE FROM server_mods WHERE server_id = $1 AND mod_id = $2",
		serverID, modID,
	)
	if err != nil {
		http.Error(w, `{"error":"deleting server mod link"}`, http.StatusInternalServerError)
		return
	}

	if result.RowsAffected() == 0 {
		http.Error(w, `{"error":"mod association not found"}`, http.StatusNotFound)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

// ToggleMod updates the is_active status of a mod in a server's modpack.
func (h *ServerHandler) ToggleMod(w http.ResponseWriter, r *http.Request) {
	serverID, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		http.Error(w, `{"error":"invalid server id"}`, http.StatusBadRequest)
		return
	}

	modID, err := uuid.Parse(r.PathValue("mod_id"))
	if err != nil {
		http.Error(w, `{"error":"invalid mod id"}`, http.StatusBadRequest)
		return
	}

	claims, ok := UserFromContext(r.Context())
	if !ok {
		http.Error(w, `{"error":"unauthorized"}`, http.StatusUnauthorized)
		return
	}

	var ownerID uuid.UUID
	err = h.db.QueryRow(r.Context(), "SELECT owner_id FROM servers WHERE id = $1", serverID).Scan(&ownerID)
	if err != nil || ownerID.String() != claims.UserID {
		http.Error(w, `{"error":"forbidden: only the server owner can modify mods"}`, http.StatusForbidden)
		return
	}

	var req struct {
		IsActive bool `json:"is_active"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid json body"}`, http.StatusBadRequest)
		return
	}

	result, err := h.db.Exec(r.Context(),
		"UPDATE server_mods SET is_active = $1 WHERE server_id = $2 AND mod_id = $3",
		req.IsActive, serverID, modID,
	)
	if err != nil {
		http.Error(w, `{"error":"updating mod status"}`, http.StatusInternalServerError)
		return
	}

	if result.RowsAffected() == 0 {
		http.Error(w, `{"error":"mod association not found"}`, http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(map[string]bool{"is_active": req.IsActive})
}

// CreateServerRequest defines the payload for creating a new server.
type CreateServerRequest struct {
	Name       string `json:"name"`
	IPAddress  string `json:"ip_address"`
	Port       int    `json:"port"`
	InviteCode string `json:"invite_code"`
}

// JoinServerRequest defines the payload for joining a server via invite code.
type JoinServerRequest struct {
	InviteCode string `json:"invite_code"`
}

// generateInviteCode produces a human-friendly uppercase invite code like "FLK-AB7CD9".
func generateInviteCode() (string, error) {
	const charset = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"
	b := make([]byte, 6)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	for i := range b {
		b[i] = charset[int(b[i])%len(charset)]
	}
	return "FLK-" + string(b), nil
}

// Create adds a new FS25 server where the caller is the owner.
func (h *ServerHandler) Create(w http.ResponseWriter, r *http.Request) {
	claims, ok := UserFromContext(r.Context())
	if !ok {
		http.Error(w, `{"error":"unauthorized"}`, http.StatusUnauthorized)
		return
	}

	userUUID, err := uuid.Parse(claims.UserID)
	if err != nil {
		http.Error(w, `{"error":"invalid user id in token"}`, http.StatusUnauthorized)
		return
	}

	var req CreateServerRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid json body"}`, http.StatusBadRequest)
		return
	}

	name := strings.TrimSpace(req.Name)
	if name == "" {
		http.Error(w, `{"error":"server name is required"}`, http.StatusBadRequest)
		return
	}
	if len(name) > 100 {
		name = name[:100]
	}

	ipAddress := strings.TrimSpace(req.IPAddress)
	if len(ipAddress) > 45 {
		ipAddress = ipAddress[:45]
	}

	port := req.Port
	if port <= 0 || port > 65535 {
		port = 10823
	}

	inviteCode := strings.ToUpper(strings.TrimSpace(req.InviteCode))
	if inviteCode != "" {
		if len(inviteCode) < 3 || len(inviteCode) > 20 {
			http.Error(w, `{"error":"invite code must be between 3 and 20 characters"}`, http.StatusBadRequest)
			return
		}
	} else {
		code, err := generateInviteCode()
		if err != nil {
			http.Error(w, `{"error":"failed to generate invite code"}`, http.StatusInternalServerError)
			return
		}
		inviteCode = code
	}

	serverID := uuid.New()
	var createdServer ServerDTO
	var ownerID uuid.UUID
	var sID uuid.UUID

	err = h.db.QueryRow(r.Context(),
		`INSERT INTO servers (id, name, ip_address, port, invite_code, owner_id)
		 VALUES ($1, $2, $3, $4, $5, $6)
		 RETURNING id, name, COALESCE(ip_address, ''), port, invite_code, owner_id, created_at`,
		serverID, name, ipAddress, port, inviteCode, userUUID,
	).Scan(&sID, &createdServer.Name, &createdServer.IPAddress, &createdServer.Port, &createdServer.InviteCode, &ownerID, &createdServer.CreatedAt)

	if err != nil {
		http.Error(w, `{"error":"failed to create server (invite code may already exist)"}`, http.StatusConflict)
		return
	}

	createdServer.ID = sID.String()
	createdServer.OwnerID = ownerID.String()
	createdServer.ModCount = 0

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	_ = json.NewEncoder(w).Encode(createdServer)
}

// Join associates the authenticated user to an existing server via its invite code.
func (h *ServerHandler) Join(w http.ResponseWriter, r *http.Request) {
	claims, ok := UserFromContext(r.Context())
	if !ok {
		http.Error(w, `{"error":"unauthorized"}`, http.StatusUnauthorized)
		return
	}

	userUUID, err := uuid.Parse(claims.UserID)
	if err != nil {
		http.Error(w, `{"error":"invalid user id in token"}`, http.StatusUnauthorized)
		return
	}

	var req JoinServerRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid json body"}`, http.StatusBadRequest)
		return
	}

	code := strings.ToUpper(strings.TrimSpace(req.InviteCode))
	if code == "" {
		http.Error(w, `{"error":"invite code is required"}`, http.StatusBadRequest)
		return
	}

	var s ServerDTO
	var serverID, ownerID uuid.UUID

	err = h.db.QueryRow(r.Context(),
		`SELECT id, name, COALESCE(ip_address, ''), port, invite_code, owner_id, created_at
		 FROM servers
		 WHERE UPPER(invite_code) = UPPER($1)`,
		code,
	).Scan(&serverID, &s.Name, &s.IPAddress, &s.Port, &s.InviteCode, &ownerID, &s.CreatedAt)

	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			http.Error(w, `{"error":"server not found for invite code"}`, http.StatusNotFound)
			return
		}
		http.Error(w, `{"error":"database error"}`, http.StatusInternalServerError)
		return
	}

	s.ID = serverID.String()
	s.OwnerID = ownerID.String()

	// If user is not the owner, link them as a server member
	if ownerID != userUUID {
		_, err = h.db.Exec(r.Context(),
			`INSERT INTO server_members (server_id, user_id)
			 VALUES ($1, $2)
			 ON CONFLICT (server_id, user_id) DO NOTHING`,
			serverID, userUUID,
		)
		if err != nil {
			http.Error(w, `{"error":"failed to join server"}`, http.StatusInternalServerError)
			return
		}
	}

	// Fetch active mod count
	_ = h.db.QueryRow(r.Context(),
		`SELECT COUNT(mod_id) FILTER (WHERE is_active = true) FROM server_mods WHERE server_id = $1`,
		serverID,
	).Scan(&s.ModCount)

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(s)
}
