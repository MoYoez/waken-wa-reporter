//go:build windows

package win32

import (
	"errors"
	"path/filepath"
	"syscall"
	"unsafe"
)

const (
	processQueryLimitedInformation = 0x1000
	imageNameBufChars              = 32768
)

var (
	kernel32                       = syscall.NewLazyDLL("kernel32.dll")
	procOpenProcess                = kernel32.NewProc("OpenProcess")
	procCloseHandle                = kernel32.NewProc("CloseHandle")
	procQueryFullProcessImageNameW = kernel32.NewProc("QueryFullProcessImageNameW")
)

// exeBaseNameFromPID returns the executable file base name for the process ID (e.g. "Code.exe").
func exeBaseNameFromPID(pid uintptr) (string, error) {
	if pid == 0 {
		return "", errors.New("invalid pid")
	}
	h, _, _ := procOpenProcess.Call(processQueryLimitedInformation, 0, pid)
	if h == 0 {
		return "", errors.New("OpenProcess failed")
	}
	defer procCloseHandle.Call(h)

	var buf [imageNameBufChars]uint16
	n := uint32(len(buf))
	r, _, _ := procQueryFullProcessImageNameW.Call(h, 0, uintptr(unsafe.Pointer(&buf[0])), uintptr(unsafe.Pointer(&n)))
	if r == 0 {
		return "", errors.New("QueryFullProcessImageNameW failed")
	}
	full := syscall.UTF16ToString(buf[:n])
	if full == "" {
		return "", errors.New("empty image path")
	}
	return filepath.Base(full), nil
}
