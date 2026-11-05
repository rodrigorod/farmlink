# Build stage
FROM golang:1.24-alpine AS builder

WORKDIR /app

# Install git and ca-certificates
RUN apk add --no-cache git ca-certificates

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o /farmlink-server ./cmd/server/main.go

# Production minimal scratch/alpine image
FROM alpine:3.19

RUN apk --no-cache add ca-certificates tzdata

WORKDIR /app

COPY --from=builder /farmlink-server /app/farmlink-server

EXPOSE 8088

ENTRYPOINT ["/app/farmlink-server"]
