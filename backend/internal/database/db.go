package database

import (
	"context"
	_ "embed"
	"fmt"
	"log/slog"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
)

//go:embed schema.sql
var schemaSQL string

// NewPool initializes a connection pool to PostgreSQL.
func NewPool(ctx context.Context, connString string) (*pgxpool.Pool, error) {
	cfg, err := pgxpool.ParseConfig(connString)
	if err != nil {
		return nil, fmt.Errorf("parsing db config: %w", err)
	}

	cfg.MaxConns = 25
	cfg.MinConns = 5
	cfg.MaxConnLifetime = 1 * time.Hour
	cfg.MaxConnIdleTime = 30 * time.Minute

	pool, err := pgxpool.NewWithConfig(ctx, cfg)
	if err != nil {
		return nil, fmt.Errorf("creating db pool: %w", err)
	}

	pingCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	if err := pool.Ping(pingCtx); err != nil {
		return nil, fmt.Errorf("pinging db: %w", err)
	}

	return pool, nil
}

// Migrate executes the embedded schema DDL if tables do not exist.
func Migrate(ctx context.Context, pool *pgxpool.Pool) error {
	slog.Info("running database schema migration")
	if _, err := pool.Exec(ctx, schemaSQL); err != nil {
		return fmt.Errorf("executing schema migration: %w", err)
	}
	slog.Info("database schema migration completed successfully")
	return nil
}
