package activity

import (
	"encoding/json"
	"net/http"
)

// ReportRequest matches the activity API JSON body.
type ReportRequest struct {
	Device       string         `json:"device"`
	DeviceName   string         `json:"device_name,omitempty"`
	DeviceType   string         `json:"device_type,omitempty"`
	ProcessName  string         `json:"process_name"`
	ProcessTitle string         `json:"process_title,omitempty"`
	BatteryLevel *int           `json:"battery_level,omitempty"`
	PushMode     string         `json:"push_mode,omitempty"`
	Metadata     map[string]any `json:"metadata,omitempty"`
}

type apiResponse struct {
	Success bool            `json:"success"`
	Data    json.RawMessage `json:"data"`
}

// Client posts activity events to the configured server.
type Client struct {
	BaseURL    string
	Token      string
	HTTPClient *http.Client
}
