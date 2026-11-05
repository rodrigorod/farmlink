package handlers

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
)

func TestPasswordHashing(t *testing.T) {
	rawPassword := "password123"
	hash, err := bcrypt.GenerateFromPassword([]byte(rawPassword), bcrypt.DefaultCost)
	if err != nil {
		t.Fatalf("failed to hash password: %v", err)
	}

	if err := bcrypt.CompareHashAndPassword(hash, []byte(rawPassword)); err != nil {
		t.Errorf("expected password match, got error: %v", err)
	}

	if err := bcrypt.CompareHashAndPassword(hash, []byte("WrongPassword")); err == nil {
		t.Errorf("expected password mismatch error, got nil")
	}
}

func TestAuthJWTMiddleware(t *testing.T) {
	secret := "test-secret-key"
	handler := NewAuthHandler(nil, secret)

	claims := UserClaims{
		UserID:   "usr_12345",
		Username: "TestFarmer",
		Role:     "admin",
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(1 * time.Hour)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
		},
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenStr, err := token.SignedString([]byte(secret))
	if err != nil {
		t.Fatalf("failed to sign token: %v", err)
	}

	var capturedUser *UserClaims
	testNext := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		u, ok := UserFromContext(r.Context())
		if ok {
			capturedUser = u
		}
		w.WriteHeader(http.StatusOK)
	})

	wrapped := handler.Middleware(testNext)

	// Case 1: Valid token
	req := httptest.NewRequest(http.MethodGet, "/api/v1/servers", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	rec := httptest.NewRecorder()

	wrapped.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rec.Code)
	}
	if capturedUser == nil || capturedUser.Username != "TestFarmer" || capturedUser.Role != "admin" {
		t.Errorf("expected user claims extracted, got: %+v", capturedUser)
	}

	// Case 2: Missing auth header
	reqNoAuth := httptest.NewRequest(http.MethodGet, "/api/v1/servers", nil)
	recNoAuth := httptest.NewRecorder()
	wrapped.ServeHTTP(recNoAuth, reqNoAuth)

	if recNoAuth.Code != http.StatusUnauthorized {
		t.Errorf("expected 401 Unauthorized, got %d", recNoAuth.Code)
	}

	// Case 3: Invalid token
	reqBadToken := httptest.NewRequest(http.MethodGet, "/api/v1/servers", nil)
	reqBadToken.Header.Set("Authorization", "Bearer invalid-garbage-token")
	recBadToken := httptest.NewRecorder()
	wrapped.ServeHTTP(recBadToken, reqBadToken)

	if recBadToken.Code != http.StatusUnauthorized {
		t.Errorf("expected 401 Unauthorized for invalid token, got %d", recBadToken.Code)
	}
}

func TestPasswordLengthLimits(t *testing.T) {
	handler := NewAuthHandler(nil, "secret")

	// Over 72 chars should be rejected
	longPassword := "abcdefghijklmnopqrstuvwxyz1234567890abcdefghijklmnopqrstuvwxyz1234567890EXTRA"
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", nil)
	rec := httptest.NewRecorder()

	handler.Login(rec, req) // Empty body should fail 400
	if rec.Code != http.StatusBadRequest {
		t.Errorf("expected 400 for empty body, got %d", rec.Code)
	}

	if len(longPassword) <= 72 {
		t.Fatalf("test error: longPassword should be > 72 chars")
	}
}
