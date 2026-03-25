package foreground

import "errors"

// ErrUnsupported is returned by GetSnapshot on unsupported platforms.
var ErrUnsupported = errors.New("foreground: snapshot not supported on this OS")

// Snapshot holds the active foreground app identity for activity reporting.
type Snapshot struct {
	ProcessName  string
	ProcessTitle string
}
