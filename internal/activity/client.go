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

// ErrNilPending is returned when the server sends 202 without a usable approval URL.
var ErrNilPending = fmt.Errorf("活动上报：待审核响应缺少 approvalUrl")

const defaultPath = "/api/activity"

// Post sends one activity report. Accepts HTTP 200/201 with success JSON.
func (c *Client) Post(ctx context.Context, req ReportRequest) error {
	if c.Token == "" {
		return fmt.Errorf("活动上报：缺少 Token")
	}
	if req.GeneratedHashKey == "" || req.ProcessName == "" {
		return fmt.Errorf("活动上报：generatedHashKey 和 process_name 不能为空")
	}

	base := strings.TrimRight(c.BaseURL, "/")
	url := base + defaultPath

	body, err := json.Marshal(req)
	if err != nil {
		return fmt.Errorf("活动上报：编码请求体失败：%w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return fmt.Errorf("活动上报：构造请求失败：%w", err)
	}
	httpReq.Header.Set("Authorization", "Bearer "+c.Token)
	httpReq.Header.Set("Content-Type", "application/json")

	client := c.HTTPClient
	if client == nil {
		client = &http.Client{Timeout: 30 * time.Second}
	}

	resp, err := client.Do(httpReq)
	if err != nil {
		return fmt.Errorf("活动上报：请求失败：%w", err)
	}
	defer resp.Body.Close()

	raw, _ := io.ReadAll(resp.Body)

	switch resp.StatusCode {
	case http.StatusCreated, http.StatusOK:
		var out apiResponse
		if err := json.Unmarshal(raw, &out); err != nil {
			return fmt.Errorf("活动上报：解析成功响应失败：%w", err)
		}
		if !out.Success {
			return fmt.Errorf("活动上报：服务端返回成功状态码，但 success=false")
		}
		return nil
	case http.StatusAccepted:
		var pend pendingResponse
		if err := json.Unmarshal(raw, &pend); err != nil {
			return fmt.Errorf("活动上报：解析 202 响应失败：%w", err)
		}
		if !pend.Pending {
			return fmt.Errorf("活动上报：收到异常 202 响应（缺少 pending=true）：%s", strings.TrimSpace(string(raw)))
		}
		url := strings.TrimSpace(pend.ApprovalURL)
		if url == "" {
			return ErrNilPending
		}
		msg := strings.TrimSpace(pend.Error)
		return &PendingApprovalError{ApprovalURL: url, Message: msg}
	case http.StatusUnauthorized:
		return fmt.Errorf("活动上报：401 未授权（Token 无效或已停用）")
	case http.StatusBadRequest:
		return fmt.Errorf("活动上报：400 请求错误：%s", strings.TrimSpace(string(raw)))
	case http.StatusInternalServerError:
		return fmt.Errorf("活动上报：500 服务器错误：%s", strings.TrimSpace(string(raw)))
	default:
		return fmt.Errorf("活动上报：未预期的状态码 %d：%s", resp.StatusCode, strings.TrimSpace(string(raw)))
	}
}
