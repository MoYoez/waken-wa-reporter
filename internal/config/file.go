package config

import (
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
)

const defaultBaseURL = "http://localhost:3000"

// DefaultFilePath returns e.g. %APPDATA%/waken-wa/config.json on Windows.
func DefaultFilePath() (string, error) {
	dir, err := os.UserConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(dir, "waken-wa", "config.json"), nil
}

// Load reads and parses the config file. Missing file is an error.
func Load(path string) (*File, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var f File
	if err := json.Unmarshal(b, &f); err != nil {
		return nil, err
	}
	return &f, nil
}

// Save writes JSON with restrictive permissions (0600).
func Save(path string, f *File) error {
	if f == nil {
		return errors.New("nil config")
	}
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0o700); err != nil {
		return err
	}
	b, err := json.MarshalIndent(f, "", "  ")
	if err != nil {
		return err
	}
	tmp := path + ".tmp"
	if err := os.WriteFile(tmp, b, 0o600); err != nil {
		return err
	}
	return os.Rename(tmp, path)
}

// EffectiveBaseURL returns a non-empty base URL.
func EffectiveBaseURL(f *File) string {
	if f == nil || f.BaseURL == "" {
		return defaultBaseURL
	}
	return f.BaseURL
}
