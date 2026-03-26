package config

import (
	"bufio"
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strings"
)

// Resolve returns base URL and API token. Priority: env WAKEN_* > saved file > interactive wizard.
func Resolve() (baseURL, token string, err error) {
	token = strings.TrimSpace(os.Getenv("WAKEN_API_TOKEN"))
	baseURL = strings.TrimSpace(os.Getenv("WAKEN_BASE_URL"))
	if token != "" {
		if baseURL == "" {
			baseURL = defaultBaseURL
		}
		return strings.TrimRight(baseURL, "/"), token, nil
	}

	if encoded := strings.TrimSpace(os.Getenv("WAKEN_CONFIG_BASE64")); encoded != "" {
		cfg, err := FromBase64(encoded)
		if err != nil {
			return "", "", fmt.Errorf("解析 WAKEN_CONFIG_BASE64 失败: %w", err)
		}
		if strings.TrimSpace(cfg.APIToken) != "" {
			u := EffectiveBaseURL(cfg)
			if envURL := strings.TrimSpace(os.Getenv("WAKEN_BASE_URL")); envURL != "" {
				u = strings.TrimRight(envURL, "/")
			}
			return u, strings.TrimSpace(cfg.APIToken), nil
		}
	}

	path, err := DefaultFilePath()
	if err != nil {
		return "", "", err
	}
	if f, err := Load(path); err == nil && strings.TrimSpace(f.APIToken) != "" {
		u := EffectiveBaseURL(f)
		if envURL := strings.TrimSpace(os.Getenv("WAKEN_BASE_URL")); envURL != "" {
			u = strings.TrimRight(envURL, "/")
		}
		return u, strings.TrimSpace(f.APIToken), nil
	}

	if !isCharDevice(os.Stdin) {
		return "", "", errors.New("未设置 WAKEN_API_TOKEN 且无已保存配置：请设置环境变量，或在终端中首次运行以完成引导")
	}

	return RunWizard(path)
}

// FromBase64 decodes backend exported Base64 JSON and extracts base_url + api_token.
func FromBase64(encoded string) (*File, error) {
	raw, err := base64.StdEncoding.DecodeString(strings.TrimSpace(encoded))
	if err != nil {
		return nil, err
	}
	var c remoteConfig
	if err := json.Unmarshal(raw, &c); err != nil {
		return nil, err
	}

	token := strings.TrimSpace(c.APIKey)
	if token == "" && len(c.Token.Items) > 0 {
		token = strings.TrimSpace(c.Token.Items[0].Token)
	}
	if token == "" {
		return nil, errors.New("base64 config missing apiKey or token.items[0].token")
	}

	endpoint := strings.TrimSpace(c.Endpoint)
	if endpoint == "" {
		endpoint = strings.TrimSpace(c.Token.ReportEndpoint)
	}

	baseURL := defaultBaseURL
	if endpoint != "" {
		baseURL = strings.TrimRight(strings.TrimSuffix(endpoint, "/api/activity"), "/")
		if baseURL == "" {
			baseURL = defaultBaseURL
		}
	}
	return &File{
		BaseURL:  baseURL,
		APIToken: token,
	}, nil
}

func isCharDevice(f *os.File) bool {
	fi, err := f.Stat()
	if err != nil {
		return false
	}
	return fi.Mode()&os.ModeCharDevice != 0
}

// ResolveDeviceName resolves a user-facing device name.
// Priority: env WAKEN_DEVICE_NAME > saved file config > empty.
func ResolveDeviceName() (string, error) {
	if v := strings.TrimSpace(os.Getenv("WAKEN_DEVICE_NAME")); v != "" {
		return v, nil
	}

	path, err := DefaultFilePath()
	if err != nil {
		return "", err
	}
	f, err := Load(path)
	if err != nil {
		return "", nil
	}
	return strings.TrimSpace(f.DeviceName), nil
}

// ResolveGeneratedHashKey resolves a stable generatedHashKey.
// Priority: env WAKEN_GENERATED_HASH_KEY > saved file config > auto-generate and save.
func ResolveGeneratedHashKey() (string, error) {
	if v := strings.TrimSpace(os.Getenv("WAKEN_GENERATED_HASH_KEY")); v != "" {
		return v, nil
	}

	path, err := DefaultFilePath()
	if err != nil {
		return "", err
	}

	f, err := Load(path)
	if err != nil {
		if !os.IsNotExist(err) {
			return "", err
		}
		f = &File{}
	}

	if v := strings.TrimSpace(f.GeneratedHashKey); v != "" {
		return v, nil
	}

	key, err := generateRandomHashKey()
	if err != nil {
		return "", err
	}
	f.GeneratedHashKey = key
	if err := Save(path, f); err != nil {
		return "", err
	}
	return key, nil
}

func generateRandomHashKey() (string, error) {
	b := make([]byte, 16)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}

// RunWizard prompts for API base URL and token, then saves to savePath.
func RunWizard(savePath string) (baseURL, token string, err error) {
	fmt.Println()
	fmt.Println("  waken-wa — 首次配置")
	fmt.Println("  请填写 API 地址与 Token（后台 /admin → API Token）。")
	fmt.Println("  也可直接粘贴后台一键复制的 Base64 配置。")
	fmt.Println()

	reader := bufio.NewReader(os.Stdin)

	fmt.Print("  Base64 配置(可留空): ")
	line, err := reader.ReadString('\n')
	if err != nil {
		return "", "", err
	}
	base64Text := strings.TrimSpace(strings.TrimRight(line, "\r\n"))
	if base64Text != "" {
		cfg, err := FromBase64(base64Text)
		if err != nil {
			return "", "", fmt.Errorf("Base64 配置无效: %w", err)
		}
		fmt.Print("  设备名称(可留空): ")
		line, err = reader.ReadString('\n')
		if err != nil {
			return "", "", err
		}
		deviceName := strings.TrimSpace(strings.TrimRight(line, "\r\n"))

		f := &File{BaseURL: EffectiveBaseURL(cfg), APIToken: cfg.APIToken, DeviceName: deviceName}
		if err := Save(savePath, f); err != nil {
			fmt.Fprintf(os.Stderr, "  警告：无法保存配置：%v\n", err)
		} else {
			fmt.Printf("\n  已保存到 %s\n\n", savePath)
		}
		return f.BaseURL, f.APIToken, nil
	}

	fmt.Printf("  API 地址 [%s]: ", defaultBaseURL)
	line, err = reader.ReadString('\n')
	if err != nil {
		return "", "", err
	}
	baseURL = strings.TrimSpace(strings.TrimRight(line, "\r\n"))
	if baseURL == "" {
		baseURL = defaultBaseURL
	}
	baseURL = strings.TrimRight(baseURL, "/")

	fmt.Print("  API Token: ")
	line, err = reader.ReadString('\n')
	if err != nil {
		return "", "", err
	}
	token = strings.TrimSpace(strings.TrimRight(line, "\r\n"))
	if token == "" {
		return "", "", errors.New("必须填写 API Token")
	}

	fmt.Print("  设备名称(可留空): ")
	line, err = reader.ReadString('\n')
	if err != nil {
		return "", "", err
	}
	deviceName := strings.TrimSpace(strings.TrimRight(line, "\r\n"))

	f := &File{BaseURL: baseURL, APIToken: token, DeviceName: deviceName}
	if err := Save(savePath, f); err != nil {
		fmt.Fprintf(os.Stderr, "  警告：无法保存配置：%v\n", err)
	} else {
		fmt.Printf("\n  已保存到 %s\n\n", savePath)
	}

	return baseURL, token, nil
}
