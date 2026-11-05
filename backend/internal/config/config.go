package config

import (
	"os"
)

// Config holds runtime configuration loaded from environment variables.
type Config struct {
	Port        string
	DatabaseURL string
	S3Endpoint  string
	S3PublicURL string
	S3AccessKey string
	S3SecretKey string
	S3Bucket    string
	S3Region    string
	JWTSecret   string
	Environment string
}

// Load reads configuration from environment variables with sensible defaults.
func Load() *Config {
	env := getEnv("ENV", "development")
	jwtSecret := getEnv("JWT_SECRET", "farmlink-dev-secret-key-fs25")

	if env == "production" && jwtSecret == "farmlink-dev-secret-key-fs25" {
		panic("JWT_SECRET must be configured with a secure key in production")
	}

	s3Endpoint := getEnv("S3_ENDPOINT", "http://localhost:9000")

	return &Config{
		Port:        getEnv("PORT", "8088"),
		DatabaseURL: getEnv("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/farmlink?sslmode=disable"),
		S3Endpoint:  s3Endpoint,
		S3PublicURL: getEnv("S3_PUBLIC_URL", s3Endpoint),
		S3AccessKey: getEnv("S3_ACCESS_KEY", "minioadmin"),
		S3SecretKey: getEnv("S3_SECRET_KEY", "minioadmin"),
		S3Bucket:    getEnv("S3_BUCKET", "farmlink-mods"),
		S3Region:    getEnv("S3_REGION", "us-east-1"),
		JWTSecret:   jwtSecret,
		Environment: env,
	}
}

func getEnv(key, fallback string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return fallback
}
