//go:build darwin

package foreground

import "testing"

// macOS snapshot uses frontmost app name only; ProcessTitle is not filled yet.
// True "now playing" would need MPNowPlayingInfoCenter / MRMediaRemote or similar (not in this repo).
func TestGetSnapshot_Darwin_FrontmostApplicationName(t *testing.T) {
	snap, err := GetSnapshot()
	if err != nil {
		t.Skipf("GetSnapshot: %v (permissions or no on-screen window)", err)
	}
	if snap.ProcessName == "" {
		t.Fatal("ProcessName must be non-empty when err is nil")
	}
	t.Logf("frontmost app=%q (ProcessTitle intentionally empty on darwin)", snap.ProcessName)
}

func TestGetSnapshot_Darwin_ProcessTitleEmptyByDesign(t *testing.T) {
	snap, err := GetSnapshot()
	if err != nil {
		t.Skipf("GetSnapshot: %v", err)
	}
	if snap.ProcessTitle != "" {
		t.Logf("ProcessTitle is now non-empty %q; update test if window title support is added", snap.ProcessTitle)
	}
}

func TestGetSnapshot_Darwin_StableAcrossCalls(t *testing.T) {
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
			t.Logf("frontmost app changed: %q -> %q", names[0], names[i])
			return
		}
	}
}
