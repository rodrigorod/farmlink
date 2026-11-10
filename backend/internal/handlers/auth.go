package handlers

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"golang.org/x/crypto/bcrypt"
)

type contextKey string

const userContextKey contextKey = "user_claims"

// UserClaims represents the JWT payload.
type UserClaims struct {
	UserID   string `json:"user_id"`
	Username string `json:"username"`
	Role     string `json:"role"`
	jwt.RegisteredClaims
}

// AuthHandler handles user registration and authentication.
type AuthHandler struct {
	db        *pgxpool.Pool
	jwtSecret []byte
}

// NewAuthHandler constructs a new AuthHandler.
func NewAuthHandler(db *pgxpool.Pool, jwtSecret string) *AuthHandler {
	return &AuthHandler{
		db:        db,
		jwtSecret: []byte(jwtSecret),
	}
}

type registerRequest struct {
	Username string `json:"username"`
	Email    string `json:"email"`
	Password string `json:"password"`
	Role     string `json:"role"`
}

type loginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type authResponse struct {
	Token string    `json:"token"`
	User  userDTO   `json:"user"`
}

type userDTO struct {
	ID       string `json:"id"`
	Username string `json:"username"`
	Email    string `json:"email"`
	Role     string `json:"role"`
}

// Register creates a new user and returns a signed JWT.
func (h *AuthHandler) Register(w http.ResponseWriter, r *http.Request) {
	var req registerRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid json body"}`, http.StatusBadRequest)
		return
	}

	req.Username = strings.TrimSpace(req.Username)
	req.Email = strings.TrimSpace(req.Email)
	if req.Username == "" || req.Email == "" || len(req.Password) < 6 || len(req.Password) > 72 {
		http.Error(w, `{"error":"username, email and password (6-72 chars) required"}`, http.StatusBadRequest)
		return
	}

	role := "player"
	if req.Role == "admin" {
		role = "admin"
	}

	hash, err := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
	if err != nil {
		http.Error(w, `{"error":"hashing password"}`, http.StatusInternalServerError)
		return
	}

	var userID uuid.UUID
	err = h.db.QueryRow(r.Context(),
		`INSERT INTO users (username, email, password_hash, role) 
		 VALUES ($1, $2, $3, $4) RETURNING id`,
		req.Username, req.Email, string(hash), role,
	).Scan(&userID)
	if err != nil {
		http.Error(w, `{"error":"username or email already registered"}`, http.StatusConflict)
		return
	}

	token, err := h.generateToken(userID.String(), req.Username, role)
	if err != nil {
		http.Error(w, `{"error":"generating token"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	_ = json.NewEncoder(w).Encode(authResponse{
		Token: token,
		User: userDTO{
			ID:       userID.String(),
			Username: req.Username,
			Email:    req.Email,
			Role:     role,
		},
	})
}

// Login verifies credentials and returns a JWT.
func (h *AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	var req loginRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"invalid json body"}`, http.StatusBadRequest)
		return
	}

	var (
		userID       uuid.UUID
		passwordHash string
		email        string
		role         string
	)

	if len(req.Password) > 72 {
		http.Error(w, `{"error":"invalid username or password"}`, http.StatusUnauthorized)
		return
	}

	err := h.db.QueryRow(r.Context(),
		`SELECT id, email, password_hash, role FROM users WHERE username = $1`,
		strings.TrimSpace(req.Username),
	).Scan(&userID, &email, &passwordHash, &role)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			http.Error(w, `{"error":"invalid username or password"}`, http.StatusUnauthorized)
			return
		}
		http.Error(w, `{"error":"database error"}`, http.StatusInternalServerError)
		return
	}

	if err := bcrypt.CompareHashAndPassword([]byte(passwordHash), []byte(req.Password)); err != nil {
		http.Error(w, `{"error":"invalid username or password"}`, http.StatusUnauthorized)
		return
	}

	token, err := h.generateToken(userID.String(), req.Username, role)
	if err != nil {
		http.Error(w, `{"error":"generating token"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(authResponse{
		Token: token,
		User: userDTO{
			ID:       userID.String(),
			Username: req.Username,
			Email:    email,
			Role:     role,
		},
	})
}

func (h *AuthHandler) generateToken(userID, username, role string) (string, error) {
	claims := UserClaims{
		UserID:   userID,
		Username: username,
		Role:     role,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(72 * time.Hour)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			Issuer:    "farmlink-api",
		},
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(h.jwtSecret)
}

// Middleware verifies Bearer JWT tokens.
func (h *AuthHandler) Middleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("Authorization")
		if !strings.HasPrefix(authHeader, "Bearer ") {
			http.Error(w, `{"error":"missing or invalid authorization header"}`, http.StatusUnauthorized)
			return
		}

		tokenStr := strings.TrimPrefix(authHeader, "Bearer ")
		claims := &UserClaims{}

		token, err := jwt.ParseWithClaims(tokenStr, claims, func(t *jwt.Token) (interface{}, error) {
			if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
				return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
			}
			return h.jwtSecret, nil
		})

		if err != nil || !token.Valid {
			http.Error(w, `{"error":"invalid token"}`, http.StatusUnauthorized)
			return
		}

		ctx := context.WithValue(r.Context(), userContextKey, claims)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

// UserFromContext retrieves authenticated user claims from context.
func UserFromContext(ctx context.Context) (*UserClaims, bool) {
	claims, ok := ctx.Value(userContextKey).(*UserClaims)
	return claims, ok
}
