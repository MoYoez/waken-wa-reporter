package main

import (
	"context"
	"encoding/json"
	"errors"
	"flag"
	"log"
	"maps"
	"os"
	"os/exec"
	"os/signal"
	"runtime"
	"strconv"
	"strings"
	"syscall"
	"time"

	"github.com/MoYoez/waken-wa-reporter/internal/activity"
	"github.com/MoYoez/waken-wa-reporter/internal/cliutil"
	"github.com/MoYoez/waken-wa-reporter/internal/config"
	"github.com/MoYoez/waken-wa-reporter/internal/platform/foreground"
	"github.com/MoYoez/waken-wa-reporter/internal/platform/media"
	"golang.org/x/term"
)

// formatMediaForLog returns a short single-line summary for logs, or "" if nothing to show.
func formatMediaForLog(m media.Info) string {
	if m.IsEmpty() {
		return ""
	}
	var parts []string
	if t := strings.TrimSpace(m.Title); t != "" {
		parts = append(parts, t)
	}
	if a := strings.TrimSpace(m.Artist); a != "" {
		parts = append(parts, a)
	}
	if al := strings.TrimSpace(m.Album); al != "" {
		parts = append(parts, al)
	}
	if len(parts) == 0 {
		return ""
	}
	return strings.Join(parts, " — ")
}

// openBrowser opens url in the system default handler (best-effort).
func openBrowser(url string) error {
	if strings.TrimSpace(url) == "" {
		return errors.New("open browser: empty url")
	}
	switch runtime.GOOS {
	case "darwin":
		return exec.Command("open", url).Start()
	case "windows":
		return exec.Command("rundll32", "url.dll,FileProtocolHandler", url).Start()
	default:
		return exec.Command("xdg-open", url).Start()
	}
}

func resolveApprovalRetryInterval() time.Duration {
	s := strings.TrimSpace(os.Getenv("WAKEN_APPROVAL_RETRY_INTERVAL"))
	if s == "" {
		return 45 * time.Second
	}
	d, err := time.ParseDuration(s)
	if err != nil || d < 5*time.Second {
		return 45 * time.Second
	}
	return d
}

func maybeWriteApprovalURLFile(url string) {
	path := strings.TrimSpace(os.Getenv("WAKEN_APPROVAL_URL_FILE"))
	if path == "" || url == "" {
		return
	}
	if err := os.WriteFile(path, []byte(url+"\n"), 0o600); err != nil {
		log.Printf("approval: could not write %s: %v", path, err)
	}
}

func maybeOpenApprovalURL(url string) {
	if strings.TrimSpace(os.Getenv("WAKEN_OPEN_APPROVAL")) != "1" {
		return
	}
	if err := openBrowser(url); err != nil {
		log.Printf("approval: could not open browser: %v", err)
	}
}

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

	deviceName, err := config.ResolveDeviceName()
	if err != nil {
		log.Fatalf("resolve device name: %v", err)
	}
	generatedHashKey, err := config.ResolveGeneratedHashKey()
	if err != nil {
		log.Fatalf("resolve generated hash key: %v", err)
	}

	device := strings.TrimSpace(os.Getenv("WAKEN_DEVICE"))
	if device == "" {
		if deviceName != "" {
			device = deviceName
		} else {
			h, err := os.Hostname()
			if err != nil {
				log.Fatalf("hostname: %v", err)
			}
			device = h
		}
	}

	if deviceName == "" {
		deviceName = device
	}

	poll, err := config.ResolvePollInterval()
	if err != nil {
		log.Fatalf("poll interval: %v", err)
	}

	heartbeat, heartbeatEnabled, err := config.ResolveHeartbeatInterval()
	if err != nil {
		log.Fatalf("heartbeat interval: %v", err)
	}

	meta := map[string]any{"source": "waken-wa"}
	if path, err := config.DefaultFilePath(); err == nil {
		if f, err := config.Load(path); err == nil && f.Metadata != nil {
			activity.MergeMetadata(meta, f.Metadata)
		}
	}
	if s := os.Getenv("WAKEN_METADATA"); s != "" {
		var extra map[string]any
		if err := json.Unmarshal([]byte(s), &extra); err != nil {
			log.Fatalf("WAKEN_METADATA: %v", err)
		}
		activity.MergeMetadata(meta, extra)
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
	approvalEvery := resolveApprovalRetryInterval()

	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	ticker := time.NewTicker(poll)
	defer ticker.Stop()

	type lastState struct {
		snap     foreground.Snapshot
		mediaSig string
	}
	var last *lastState
	var lastReport time.Time

	var pendingMode bool
	var lastPendingRetry time.Time
	var bannerShown bool

	enterPending := func(p *activity.PendingApprovalError) {
		pendingMode = true
		lastPendingRetry = time.Now()
		if p.Message != "" {
			log.Printf("approval: %s", p.Message)
		}
		if !bannerShown {
			if term.IsTerminal(int(os.Stdout.Fd())) {
				cliutil.PrintApprovalBanner(p.ApprovalURL)
			} else {
				log.Printf("approval URL (non-TTY): %s", p.ApprovalURL)
			}
			bannerShown = true
		}
		maybeWriteApprovalURLFile(p.ApprovalURL)
		maybeOpenApprovalURL(p.ApprovalURL)
	}

	postReport := func(snap foreground.Snapshot, minfo media.Info, merr error, heartbeat bool) error {
		reportMeta := maps.Clone(meta)
		if merr == nil && !minfo.IsEmpty() {
			activity.MergeMetadata(reportMeta, map[string]any{"media": minfo.AsMap()})
		} else if merr != nil && !errors.Is(merr, media.ErrNoMedia) && !errors.Is(merr, media.ErrUnsupported) {
			log.Printf("media: %v", merr)
		}
		err := client.Post(ctx, activity.ReportRequest{
			GeneratedHashKey: generatedHashKey,
			Device:           device,
			DeviceName:       deviceName,
			DeviceType:       deviceType,
			ProcessName:      snap.ProcessName,
			ProcessTitle:     snap.ProcessTitle,
			BatteryLevel:     batteryLevel,
			PushMode:         pushMode,
			Metadata:         reportMeta,
		})
		if err != nil {
			var p *activity.PendingApprovalError
			if errors.As(err, &p) {
				return err
			}
			log.Printf("report failed: %v", err)
			return err
		}
		mediaSuffix := ""
		if merr == nil {
			if s := formatMediaForLog(minfo); s != "" {
				mediaSuffix = " | media: " + s
			}
		}
		if heartbeat {
			log.Printf("activity heartbeat: %s%s", snap.ProcessName, mediaSuffix)
		} else {
			log.Printf("activity reported: %s%s", snap.ProcessName, mediaSuffix)
		}
		return nil
	}

	mediaSignature := func(minfo media.Info, merr error) string {
		if merr != nil || minfo.IsEmpty() {
			return ""
		}
		return minfo.Signature()
	}

	if snap, err := foreground.GetSnapshot(); err != nil {
		log.Printf("foreground: %v", err)
	} else {
		minfo, merr := media.GetNowPlaying()
		sig := mediaSignature(minfo, merr)
		last = &lastState{snap: snap, mediaSig: sig}
		err := postReport(snap, minfo, merr, false)
		var p *activity.PendingApprovalError
		if errors.As(err, &p) {
			enterPending(p)
		} else if err == nil {
			lastReport = time.Now()
		}
	}

	for {
		select {
		case <-ctx.Done():
			log.Println("shutting down")
			return
		case <-ticker.C:
			if pendingMode {
				if time.Since(lastPendingRetry) < approvalEvery {
					continue
				}
				lastPendingRetry = time.Now()
				snap, err := foreground.GetSnapshot()
				if err != nil {
					log.Printf("foreground: %v", err)
					continue
				}
				minfo, merr := media.GetNowPlaying()
				err = postReport(snap, minfo, merr, false)
				var p *activity.PendingApprovalError
				if errors.As(err, &p) {
					continue
				}
				if err != nil {
					continue
				}
				pendingMode = false
				log.Println("device approved; resuming normal reporting")
				last = &lastState{snap: snap, mediaSig: mediaSignature(minfo, merr)}
				lastReport = time.Now()
				continue
			}

			snap, err := foreground.GetSnapshot()
			if err != nil {
				log.Printf("foreground: %v", err)
				continue
			}
			minfo, merr := media.GetNowPlaying()
			sig := mediaSignature(minfo, merr)
			same := last != nil &&
				last.snap.ProcessName == snap.ProcessName &&
				last.snap.ProcessTitle == snap.ProcessTitle &&
				last.mediaSig == sig
			if same {
				if heartbeatEnabled && time.Since(lastReport) >= heartbeat {
					err := postReport(snap, minfo, merr, true)
					var p *activity.PendingApprovalError
					if errors.As(err, &p) {
						enterPending(p)
						continue
					}
					if err == nil {
						lastReport = time.Now()
					}
				}
				continue
			}
			last = &lastState{snap: snap, mediaSig: sig}
			err = postReport(snap, minfo, merr, false)
			var p *activity.PendingApprovalError
			if errors.As(err, &p) {
				enterPending(p)
				continue
			}
			if err == nil {
				lastReport = time.Now()
			}
		}
	}
}
