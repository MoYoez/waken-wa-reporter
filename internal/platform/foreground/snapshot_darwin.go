//go:build darwin

package foreground

import (
	"errors"

	"github.com/MoYoez/waken-wa/internal/platform/darwin"
)

// GetSnapshot returns the frontmost application name. ProcessTitle is empty until a window-title source is added.
func GetSnapshot() (Snapshot, error) {
	name, fail := darwin.GetForegroundApplicationName()
	if fail {
		return Snapshot{}, errors.New("foreground application name unavailable")
	}
	return Snapshot{ProcessName: name, ProcessTitle: ""}, nil
}
