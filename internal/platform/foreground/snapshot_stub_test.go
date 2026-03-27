//go:build !windows && !darwin

package foreground

import (
	"errors"
	"testing"
)

func TestGetSnapshot_UnsupportedPlatform(t *testing.T) {
	_, err := GetSnapshot()
	if !errors.Is(err, ErrUnsupported) {
		t.Fatalf("expected ErrUnsupported, got %v", err)
	}
}
