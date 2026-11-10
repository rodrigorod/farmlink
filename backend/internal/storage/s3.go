package storage

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	awsconfig "github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/credentials"
	"github.com/aws/aws-sdk-go-v2/service/s3"
)

// Client wraps an S3 client for object storage.
type Client struct {
	client    *s3.Client
	bucket    string
	endpoint  string
	publicURL string
}

// NewClient initializes a connection to MinIO or AWS S3.
func NewClient(ctx context.Context, endpoint, publicURL, accessKey, secretKey, bucket, region string) (*Client, error) {
	customResolver := aws.EndpointResolverWithOptionsFunc(func(service, reg string, options ...interface{}) (aws.Endpoint, error) {
		if endpoint != "" {
			return aws.Endpoint{
				URL:               endpoint,
				SigningRegion:     region,
				HostnameImmutable: true,
			}, nil
		}
		return aws.Endpoint{}, &aws.EndpointNotFoundError{}
	})

	cfg, err := awsconfig.LoadDefaultConfig(ctx,
		awsconfig.WithRegion(region),
		awsconfig.WithCredentialsProvider(credentials.NewStaticCredentialsProvider(accessKey, secretKey, "")),
		awsconfig.WithEndpointResolverWithOptions(customResolver),
	)
	if err != nil {
		return nil, fmt.Errorf("loading aws s3 config: %w", err)
	}

	s3Client := s3.NewFromConfig(cfg, func(o *s3.Options) {
		o.UsePathStyle = true
	})

	if publicURL == "" {
		publicURL = endpoint
	}

	c := &Client{
		client:    s3Client,
		bucket:    bucket,
		endpoint:  endpoint,
		publicURL: publicURL,
	}

	return c, nil
}

// EnsureBucket creates the target bucket if it does not already exist and applies public read access.
func (c *Client) EnsureBucket(ctx context.Context) error {
	_, err := c.client.HeadBucket(ctx, &s3.HeadBucketInput{
		Bucket: aws.String(c.bucket),
	})
	if err != nil {
		slog.Info("creating s3 bucket", "bucket", c.bucket)
		_, err = c.client.CreateBucket(ctx, &s3.CreateBucketInput{
			Bucket: aws.String(c.bucket),
		})
		if err != nil {
			slog.Warn("bucket creation note", "err", err)
		}
	}

	// Apply public read bucket policy on mods/* so companion clients can download directly
	policy := fmt.Sprintf(`{
		"Version": "2012-10-17",
		"Statement": [{
			"Effect": "Allow",
			"Principal": "*",
			"Action": ["s3:GetObject"],
			"Resource": ["arn:aws:s3:::%s/mods/*"]
		}]
	}`, c.bucket)
	_, _ = c.client.PutBucketPolicy(ctx, &s3.PutBucketPolicyInput{
		Bucket: aws.String(c.bucket),
		Policy: aws.String(policy),
	})

	return nil
}

// Upload uploads a stream directly to S3.
func (c *Client) Upload(ctx context.Context, key string, body io.Reader, size int64, contentType string) error {
	_, err := c.client.PutObject(ctx, &s3.PutObjectInput{
		Bucket:        aws.String(c.bucket),
		Key:           aws.String(key),
		Body:          body,
		ContentLength: aws.Int64(size),
		ContentType:   aws.String(contentType),
	})
	if err != nil {
		return fmt.Errorf("putting s3 object: %w", err)
	}
	return nil
}

// Delete removes an object from S3.
func (c *Client) Delete(ctx context.Context, key string) error {
	_, err := c.client.DeleteObject(ctx, &s3.DeleteObjectInput{
		Bucket: aws.String(c.bucket),
		Key:    aws.String(key),
	})
	if err != nil {
		return fmt.Errorf("deleting s3 object: %w", err)
	}
	return nil
}

// GetURL returns the accessible direct download URL for an object.
func (c *Client) GetURL(key string) string {
	return fmt.Sprintf("%s/%s/%s", strings.TrimRight(c.publicURL, "/"), c.bucket, key)
}
