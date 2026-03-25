//go:build windows

package foreground

import (
	"errors"

	"github.com/MoYoez/waken-wa/internal/platform/win32"
)

// GetSnapshot returns the foreground window's process executable name and window title.
func GetSnapshot() (Snapshot, error) {
	name, fail := win32.GetForegroundWindowApplicationName()
	if fail {
		return Snapshot{}, errors.New("foreground process name unavailable")
	}
	title := ""
	if t, failTitle := win32.GetForegroundWindowTitle(); !failTitle {
		title = t
	}
	return Snapshot{ProcessName: name, ProcessTitle: title}, nil
}
