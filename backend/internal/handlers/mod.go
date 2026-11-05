package handlers

import (
	"archive/zip"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"farmlink/backend/internal/storage"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// ModHandler processes multipart uploads of FS25 mods.
type ModHandler struct {
	db      *pgxpool.Pool
	storage *storage.Client
}

// NewModHandler creates a ModHandler.
func NewModHandler(db *pgxpool.Pool, storage *storage.Client) *ModHandler {
	return &ModHandler{db: db, storage: storage}
}

// ModDescXML represents the Farming Simulator modDesc.xml document.
type ModDescXML struct {
	XMLName xml.Name `xml:"modDesc"`
	Version string   `xml:"version"`
	Author  string   `xml:"author"`
	Title   TitleXML `xml:"title"`
}

// TitleXML captures either localized children or plain text title.
type TitleXML struct {
	Raw string `xml:",chardata"`
	EN  string `xml:"en"`
	FR  string `xml:"fr"`
	DE  string `xml:"de"`
}

// String returns the best available title string.
func (t TitleXML) String() string {
	if s := strings.TrimSpace(t.EN); s != "" {
		return s
	}
	if s := strings.TrimSpace(t.FR); s != "" {
		return s
	}
	if s := strings.TrimSpace(t.DE); s != "" {
		return s
	}
	return strings.TrimSpace(t.Raw)
}

// Upload handles receiving a .zip file, hashing it, parsing modDesc.xml, and uploading to S3.
func (h *ModHandler) Upload(w http.ResponseWriter, r *http.Request) {
	serverIDStr := r.PathValue("id")
	serverID, err := uuid.Parse(serverIDStr)
	if err != nil {
		http.Error(w, `{"error":"invalid server id"}`, http.StatusBadRequest)
		return
	}

	claims, ok := UserFromContext(r.Context())
	if !ok {
		http.Error(w, `{"error":"unauthorized"}`, http.StatusUnauthorized)
		return
	}

	// Verify server existence and ownership before streaming file
	var ownerID uuid.UUID
	err = h.db.QueryRow(r.Context(), "SELECT owner_id FROM servers WHERE id = $1", serverID).Scan(&ownerID)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			http.Error(w, `{"error":"server not found"}`, http.StatusNotFound)
			return
		}
		http.Error(w, `{"error":"database error"}`, http.StatusInternalServerError)
		return
	}

	if claims.Role != "admin" && ownerID.String() != claims.UserID {
		http.Error(w, `{"error":"forbidden: only server owner or admin can upload mods"}`, http.StatusForbidden)
		return
	}

	// 500 MB max request body
	r.Body = http.MaxBytesReader(w, r.Body, 500<<20)

	mr, err := r.MultipartReader()
	if err != nil {
		http.Error(w, `{"error":"invalid multipart payload"}`, http.StatusBadRequest)
		return
	}

	var fileName string
	var part io.Reader
	for {
		p, err := mr.NextPart()
		if err == io.EOF {
			break
		}
		if err != nil {
			http.Error(w, `{"error":"reading form part"}`, http.StatusBadRequest)
			return
		}
		if p.FormName() == "file" {
			fileName = filepath.Base(p.FileName())
			part = p
			break
		}
	}

	if part == nil || fileName == "" {
		http.Error(w, `{"error":"missing file in form field 'file'"}`, http.StatusBadRequest)
		return
	}

	if !strings.HasSuffix(strings.ToLower(fileName), ".zip") {
		http.Error(w, `{"error":"only .zip mod files are allowed"}`, http.StatusBadRequest)
		return
	}

	// Buffer directly to temporary file to calculate SHA-256 and allow random read for zip
	tempFile, err := os.CreateTemp("", "fs25-upload-*.zip")
	if err != nil {
		http.Error(w, `{"error":"creating temporary storage"}`, http.StatusInternalServerError)
		return
	}
	defer func() {
		_ = tempFile.Close()
		_ = os.Remove(tempFile.Name())
	}()

	hasher := sha256.New()
	tee := io.TeeReader(part, hasher)

	sizeBytes, err := io.Copy(tempFile, tee)
	if err != nil {
		http.Error(w, `{"error":"saving uploaded file stream"}`, http.StatusInternalServerError)
		return
	}

	hashSHA256 := hex.EncodeToString(hasher.Sum(nil))

	// Inspect ZIP archive
	zipReader, err := zip.NewReader(tempFile, sizeBytes)
	if err != nil {
		http.Error(w, `{"error":"corrupted or invalid zip archive"}`, http.StatusBadRequest)
		return
	}

	// Find modDesc.xml case-insensitively
	var modDescFile *zip.File
	for _, f := range zipReader.File {
		if strings.EqualFold(filepath.Base(f.Name), "moddesc.xml") {
			modDescFile = f
			break
		}
	}

	if modDescFile == nil {
		http.Error(w, `{"error":"modDesc.xml not found in zip archive"}`, http.StatusBadRequest)
		return
	}

	rc, err := modDescFile.Open()
	if err != nil {
		http.Error(w, `{"error":"opening modDesc.xml from archive"}`, http.StatusBadRequest)
		return
	}
	defer rc.Close()

	// Limit XML read to 2 MB to prevent zip bomb attacks
	xmlData, err := io.ReadAll(io.LimitReader(rc, 2<<20))
	if err != nil {
		http.Error(w, `{"error":"reading modDesc.xml"}`, http.StatusBadRequest)
		return
	}

	var modDesc ModDescXML
	if err := xml.Unmarshal(xmlData, &modDesc); err != nil {
		http.Error(w, `{"error":"corrupted or invalid modDesc.xml format"}`, http.StatusBadRequest)
		return
	}

	modName := modDesc.Title.String()
	if modName == "" {
		modName = strings.TrimSuffix(fileName, filepath.Ext(fileName))
	}
	version := strings.TrimSpace(modDesc.Version)
	if version == "" {
		version = "1.0.0.0"
	}

	// Upload to S3
	s3Key := fmt.Sprintf("mods/%s/%s", hashSHA256, fileName)
	if _, err := tempFile.Seek(0, io.SeekStart); err != nil {
		http.Error(w, `{"error":"resetting stream for s3 upload"}`, http.StatusInternalServerError)
		return
	}

	if err := h.storage.Upload(r.Context(), s3Key, tempFile, sizeBytes, "application/zip"); err != nil {
		http.Error(w, fmt.Sprintf(`{"error":"s3 upload failed: %v"}`, err), http.StatusInternalServerError)
		return
	}

	// Database upsert into mods
	var uploaderUUID *uuid.UUID
	if uid, err := uuid.Parse(claims.UserID); err == nil {
		uploaderUUID = &uid
	}

	var modID uuid.UUID
	err = h.db.QueryRow(r.Context(),
		`INSERT INTO mods (mod_name, file_name, version, hash_sha256, file_size_bytes, s3_key, uploaded_by)
		 VALUES ($1, $2, $3, $4, $5, $6, $7)
		 ON CONFLICT (hash_sha256) DO UPDATE 
		 SET mod_name = EXCLUDED.mod_name, file_name = EXCLUDED.file_name, version = EXCLUDED.version
		 RETURNING id`,
		modName, fileName, version, hashSHA256, sizeBytes, s3Key, uploaderUUID,
	).Scan(&modID)
	if err != nil {
		_ = h.storage.Delete(r.Context(), s3Key)
		http.Error(w, `{"error":"recording mod in database"}`, http.StatusInternalServerError)
		return
	}

	// Associate with server in server_mods
	_, err = h.db.Exec(r.Context(),
		`INSERT INTO server_mods (server_id, mod_id, is_active)
		 VALUES ($1, $2, true)
		 ON CONFLICT (server_id, mod_id) DO UPDATE SET is_active = true`,
		serverID, modID,
	)
	if err != nil {
		_ = h.storage.Delete(r.Context(), s3Key)
		http.Error(w, `{"error":"associating mod with server modpack"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	_ = json.NewEncoder(w).Encode(map[string]interface{}{
		"id":              modID.String(),
		"mod_name":        modName,
		"file_name":       fileName,
		"version":         version,
		"hash_sha256":     hashSHA256,
		"file_size_bytes": sizeBytes,
		"s3_key":          s3Key,
		"download_url":    h.storage.GetURL(s3Key),
	})
}
