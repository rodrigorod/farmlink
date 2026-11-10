package main

import (
	"context"
	"errors"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"farmlink/backend/internal/config"
	"farmlink/backend/internal/database"
	"farmlink/backend/internal/handlers"
	"farmlink/backend/internal/storage"
)

func main() {
	logger := slog.New(slog.NewJSONHandler(os.Stdout, nil))
	slog.SetDefault(logger)

	cfg := config.Load()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	slog.Info("starting farmlink backend server", "port", cfg.Port)

	// Database connection & migration
	dbPool, err := database.NewPool(ctx, cfg.DatabaseURL)
	if err != nil {
		slog.Error("database connection failed", "err", err)
		os.Exit(1)
	}
	defer dbPool.Close()

	if err := database.Migrate(ctx, dbPool, cfg.Environment); err != nil {
		slog.Error("schema migration failed", "err", err)
		os.Exit(1)
	}

	if cfg.JWTSecret == "farmlink-dev-secret-key-fs25" {
		slog.Warn("using default development JWT_SECRET; set a secure JWT_SECRET environment variable for production")
	}

	// Storage connection
	s3Client, err := storage.NewClient(ctx, cfg.S3Endpoint, cfg.S3PublicURL, cfg.S3AccessKey, cfg.S3SecretKey, cfg.S3Bucket, cfg.S3Region)
	if err != nil {
		slog.Error("s3 storage client initialization failed", "err", err)
		os.Exit(1)
	}
	if err := s3Client.EnsureBucket(ctx); err != nil {
		slog.Error("s3 storage bucket check failed", "err", err)
		os.Exit(1)
	}

	// Handlers
	authHandler := handlers.NewAuthHandler(dbPool, cfg.JWTSecret)
	serverHandler := handlers.NewServerHandler(dbPool, s3Client)
	modHandler := handlers.NewModHandler(dbPool, s3Client)

	// Router using native Go 1.22+ pattern matching
	mux := http.NewServeMux()

	// Public routes
	mux.HandleFunc("GET /api/v1/health", handlers.Health)
	mux.HandleFunc("POST /api/v1/auth/register", authHandler.Register)
	mux.HandleFunc("POST /api/v1/auth/login", authHandler.Login)
	mux.HandleFunc("GET /api/v1/servers/{id}/manifest", serverHandler.Manifest)

	// Authenticated routes
	mux.Handle("GET /api/v1/servers", authHandler.Middleware(http.HandlerFunc(serverHandler.List)))
	mux.Handle("POST /api/v1/servers", authHandler.Middleware(http.HandlerFunc(serverHandler.Create)))
	mux.Handle("POST /api/v1/servers/join", authHandler.Middleware(http.HandlerFunc(serverHandler.Join)))
	mux.Handle("POST /api/v1/servers/{id}/mods/upload", authHandler.Middleware(http.HandlerFunc(modHandler.Upload)))
	mux.Handle("DELETE /api/v1/servers/{id}/mods/{mod_id}", authHandler.Middleware(http.HandlerFunc(serverHandler.RemoveMod)))
	mux.Handle("PATCH /api/v1/servers/{id}/mods/{mod_id}", authHandler.Middleware(http.HandlerFunc(serverHandler.ToggleMod)))

	// CORS and logging middleware wrapper
	handler := corsMiddleware(mux)

	server := &http.Server{
		Addr:              ":" + cfg.Port,
		Handler:           handler,
		ReadHeaderTimeout: 10 * time.Second,
		IdleTimeout:       120 * time.Second,
	}

	// Graceful shutdown channel
	stopChan := make(chan os.Signal, 1)
	signal.Notify(stopChan, os.Interrupt, syscall.SIGTERM)

	go func() {
		slog.Info("http server listening", "address", server.Addr)
		if err := server.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			slog.Error("server error", "err", err)
			os.Exit(1)
		}
	}()

	<-stopChan
	slog.Info("shutting down server gracefully...")

	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		slog.Error("graceful shutdown failed", "err", err)
	}
	slog.Info("server exited cleanly")
}

func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")

		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusOK)
			return
		}

		next.ServeHTTP(w, r)
	})
}
