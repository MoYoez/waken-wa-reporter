package activity

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

const defaultPath = "/api/activity"

// Post sends one activity report. Accepts HTTP 200/201 with success JSON.
func (c *Client) Post(ctx context.Context, req ReportRequest) error {
	if c.Token == "" {
		return fmt.Errorf("activity: empty token")
	}
	if req.Device == "" || req.ProcessName == "" {
		return fmt.Errorf("activity: device and process_name are required")
	}

	base := strings.TrimRight(c.BaseURL, "/")
	url := base + defaultPath

	body, err := json.Marshal(req)
	if err != nil {
		return fmt.Errorf("activity: encode body: %w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return fmt.Errorf("activity: build request: %w", err)
	}
	httpReq.Header.Set("Authorization", "Bearer "+c.Token)
	httpReq.Header.Set("Content-Type", "application/json")

	client := c.HTTPClient
	if client == nil {
		client = &http.Client{Timeout: 30 * time.Second}
	}

	resp, err := client.Do(httpReq)
	if err != nil {
		return fmt.Errorf("activity: request failed: %w", err)
	}
	defer resp.Body.Close()

	raw, _ := io.ReadAll(resp.Body)

	switch resp.StatusCode {
	case http.StatusCreated, http.StatusOK:
		var out apiResponse
		if err := json.Unmarshal(raw, &out); err != nil {
			return fmt.Errorf("activity: decode success body: %w", err)
		}
		if !out.Success {
			return fmt.Errorf("activity: server returned 201 but success=false")
		}
		return nil
	case http.StatusUnauthorized:
		return fmt.Errorf("activity: 401 unauthorized (invalid or disabled token)")
	case http.StatusBadRequest:
		return fmt.Errorf("activity: 400 bad request: %s", strings.TrimSpace(string(raw)))
	case http.StatusInternalServerError:
		return fmt.Errorf("activity: 500 server error: %s", strings.TrimSpace(string(raw)))
	default:
		return fmt.Errorf("activity: unexpected status %d: %s", resp.StatusCode, strings.TrimSpace(string(raw)))
	}
}
