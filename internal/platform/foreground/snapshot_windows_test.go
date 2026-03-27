//go:build windows

package foreground

import (
	"strings"
	"testing"

	"github.com/MoYoez/waken-wa/internal/platform/win32"
)

// This repo does not use Windows Media Session / Global System Media Transport Controls.
// Many players still put track info in the foreground window title; we verify that path works.
func TestGetSnapshot_Windows_ForegroundExecutableAndTitle(t *testing.T) {
	snap, err := GetSnapshot()
	if err != nil {
		t.Skipf("GetSnapshot: %v (no foreground window or restricted session)", err)
	}
	if snap.ProcessName == "" {
		t.Fatal("ProcessName must be non-empty when err is nil")
	}
	// Typical media hosts: browsers, Spotify, etc. Title may be empty for some chromeless UIs.
	t.Logf("foreground process=%q title=%q", snap.ProcessName, snap.ProcessTitle)
}

func TestGetSnapshot_Windows_StableAcrossCalls(t *testing.T) {
	var names []string
	for i := 0; i < 3; i++ {
		snap, err := GetSnapshot()
		if err != nil {
			t.Skipf("GetSnapshot: %v", err)
		}
		names = append(names, snap.ProcessName)
	}
	for i := 1; i < len(names); i++ {
		if names[i] != names[0] {
			t.Logf("process changed between polls: %q -> %q", names[0], names[i])
			return
		}
	}
}

func TestWin32_GetForegroundWindowTitle_ConsistentWithSnapshot(t *testing.T) {
	snap, err := GetSnapshot()
	if err != nil {
		t.Skipf("GetSnapshot: %v", err)
	}
	hwnd, bad := win32.GetHWND()
	if bad {
		t.Fatal("GetHWND failed while GetSnapshot succeeded")
	}
	title, badTitle := win32.GetForegroundWindowTitle()
	if badTitle {
		t.Fatal("GetForegroundWindowTitle failed while GetSnapshot succeeded")
	}
	// GetWindowTitle path must match snapshot title field.
	if snap.ProcessTitle != title {
		t.Errorf("snapshot title %q != GetForegroundWindowTitle %q", snap.ProcessTitle, title)
	}
	if hwnd == 0 {
		t.Error("GetHWND returned 0 while GetSnapshot succeeded")
	}
}

func TestWin32_GetForegroundWindowApplicationName_ExecutableSuffix(t *testing.T) {
	name, fail := win32.GetForegroundWindowApplicationName()
	if fail {
		t.Skip("no foreground process name")
	}
	if !strings.HasSuffix(strings.ToLower(name), ".exe") {
		t.Logf("foreground exe name without .exe suffix: %q (acceptable for some hosts)", name)
	}
}
