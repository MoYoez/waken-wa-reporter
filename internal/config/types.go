package config

// File is persisted JSON next to the user config dir.
type File struct {
	BaseURL          string `json:"base_url"`
	APIToken         string `json:"api_token"`
	DeviceName       string `json:"device_name,omitempty"`
	GeneratedHashKey string `json:"generated_hash_key,omitempty"`
	// PollIntervalMs: foreground poll period; nil or <=0 → default (see DefaultPollIntervalMs).
	PollIntervalMs *int `json:"poll_interval_ms,omitempty"`
	// HeartbeatIntervalMs: max silence before re-reporting same foreground; nil → default;
	// explicit 0 disables heartbeat (matches WAKEN_HEARTBEAT_INTERVAL=0).
	HeartbeatIntervalMs *int `json:"heartbeat_interval_ms,omitempty"`
}

type remoteConfig struct {
	Endpoint string `json:"endpoint"`
	APIKey   string `json:"apiKey"`

	Token struct {
		ReportEndpoint string `json:"reportEndpoint"`
		Items          []struct {
			Token string `json:"token"`
		} `json:"items"`
	} `json:"token"`
}
