package main

import (
	"context"
	"encoding/json"
	"flag"
	"log"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"syscall"
	"time"

	"github.com/MoYoez/waken-wa/internal/activity"
	"github.com/MoYoez/waken-wa/internal/config"
	"github.com/MoYoez/waken-wa/internal/platform/foreground"
)

func main() {
	setup := flag.Bool("setup", false, "run interactive setup (URL + API token), save, and exit")
	flag.Parse()
	if *setup {
		path, err := config.DefaultFilePath()
		if err != nil {
			log.Fatal(err)
		}
		if _, _, err := config.RunWizard(path); err != nil {
			log.Fatal(err)
		}
		return
	}

	baseURL, token, err := config.Resolve()
	if err != nil {
		log.Fatal(err)
	}

	device := os.Getenv("WAKEN_DEVICE")
	if device == "" {
		h, err := os.Hostname()
		if err != nil {
			log.Fatalf("hostname: %v", err)
		}
		device = h
	}

	poll := 2 * time.Second
	if s := os.Getenv("WAKEN_POLL_INTERVAL"); s != "" {
		d, err := time.ParseDuration(s)
		if err != nil {
			log.Fatalf("WAKEN_POLL_INTERVAL: %v", err)
		}
		poll = d
	}

	meta := map[string]any{"source": "waken-wa"}
	if s := os.Getenv("WAKEN_METADATA"); s != "" {
		var extra map[string]any
		if err := json.Unmarshal([]byte(s), &extra); err != nil {
			log.Fatalf("WAKEN_METADATA: %v", err)
		}
		for k, v := range extra {
			meta[k] = v
		}
	}

	deviceType := strings.TrimSpace(os.Getenv("WAKEN_DEVICE_TYPE"))
	if deviceType == "" {
		deviceType = "desktop"
	}
	if deviceType != "desktop" && deviceType != "tablet" && deviceType != "mobile" {
		log.Fatalf("WAKEN_DEVICE_TYPE must be desktop/tablet/mobile, got: %s", deviceType)
	}

	pushMode := strings.TrimSpace(os.Getenv("WAKEN_PUSH_MODE"))
	if pushMode == "" {
		pushMode = "realtime"
	}
	if pushMode != "realtime" && pushMode != "active" {
		log.Fatalf("WAKEN_PUSH_MODE must be realtime/active, got: %s", pushMode)
	}

	var batteryLevel *int
	if s := strings.TrimSpace(os.Getenv("WAKEN_BATTERY_LEVEL")); s != "" {
		v, err := strconv.Atoi(s)
		if err != nil {
			log.Fatalf("WAKEN_BATTERY_LEVEL: %v", err)
		}
		if v < 0 || v > 100 {
			log.Fatalf("WAKEN_BATTERY_LEVEL must be in [0,100], got: %d", v)
		}
		batteryLevel = &v
	}

	client := &activity.Client{BaseURL: baseURL, Token: token}

	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	ticker := time.NewTicker(poll)
	defer ticker.Stop()

	var last *foreground.Snapshot

	report := func(snap foreground.Snapshot) {
		err := client.Post(ctx, activity.ReportRequest{
			Device:       device,
			DeviceName:   device,
			DeviceType:   deviceType,
			ProcessName:  snap.ProcessName,
			ProcessTitle: snap.ProcessTitle,
			BatteryLevel: batteryLevel,
			PushMode:     pushMode,
			Metadata:     meta,
		})
		if err != nil {
			log.Printf("report failed: %v", err)
			return
		}
		log.Printf("activity reported: %s", snap.ProcessName)
	}

	if snap, err := foreground.GetSnapshot(); err != nil {
		log.Printf("foreground: %v", err)
	} else {
		cp := snap
		last = &cp
		report(snap)
	}

	for {
		select {
		case <-ctx.Done():
			log.Println("shutting down")
			return
		case <-ticker.C:
			snap, err := foreground.GetSnapshot()
			if err != nil {
				log.Printf("foreground: %v", err)
				continue
			}
			if last != nil && last.ProcessName == snap.ProcessName && last.ProcessTitle == snap.ProcessTitle {
				continue
			}
			cp := snap
			last = &cp
			report(snap)
		}
	}
}
