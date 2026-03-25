//go:build !windows && !darwin

package foreground

// GetSnapshot is not implemented on this platform.
func GetSnapshot() (Snapshot, error) {
	return Snapshot{}, ErrUnsupported
}
