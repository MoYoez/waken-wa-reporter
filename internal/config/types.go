package config

// File is persisted JSON next to the user config dir.
type File struct {
	BaseURL          string `json:"base_url"`
	APIToken         string `json:"api_token"`
	DeviceName       string `json:"device_name,omitempty"`
	GeneratedHashKey string `json:"generated_hash_key,omitempty"`
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
