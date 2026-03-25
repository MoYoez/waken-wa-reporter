package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/MoYoez/waken-wa/internal/activity"
	"github.com/MoYoez/waken-wa/internal/platform/foreground"
)

func main() {
	token := os.Getenv("WAKEN_API_TOKEN")
	if token == "" {
		log.Fatal("WAKEN_API_TOKEN is required")
	}

	baseURL := os.Getenv("WAKEN_BASE_URL")
	if baseURL == "" {
		baseURL = "http://localhost:3000"
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

	client := &activity.Client{BaseURL: baseURL, Token: token}

	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	ticker := time.NewTicker(poll)
	defer ticker.Stop()

	var last *foreground.Snapshot

	report := func(snap foreground.Snapshot) {
		err := client.Post(ctx, activity.ReportRequest{
			Device:       device,
			ProcessName:  snap.ProcessName,
			ProcessTitle: snap.ProcessTitle,
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
