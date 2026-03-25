//go:build windows

package win32

import (
	"syscall"
	"unsafe"
)

var ForeGroundCaller *syscall.LazyProc

var GetWindowHeadLineText *syscall.LazyProc
var GetWindowThreadProcessID *syscall.LazyProc

var ProcessID uintptr

func init() {
	ForeGroundCaller = syscall.NewLazyDLL("user32.dll").NewProc("GetForegroundWindow")
	GetWindowThreadProcessID = syscall.NewLazyDLL("user32.dll").NewProc("GetWindowThreadProcessId")
	GetWindowHeadLineText = syscall.NewLazyDLL("user32.dll").NewProc("GetWindowTextW")
}

func GetHWND() (hwnd uintptr, errorBool bool) {
	r, _, _ := ForeGroundCaller.Call()
	if r == 0 {
		return 0, true
	}
	return r, false
}

// GetForegroundWindowID Windows Platform Only.
func GetForegroundWindowID() (CallerID uintptr, errorBool bool) {
	getHWND, errorBool := GetHWND()
	if errorBool {
		return 0, true
	}
	GetWindowThreadProcessID.Call(getHWND, uintptr(unsafe.Pointer(&ProcessID)))
	return ProcessID, errorBool
}

func GetForegroundWindowApplicationName() (CallerName string, errorBool bool) {
	CallerID, errorBool := GetForegroundWindowID()
	if errorBool {
		return "", true
	}
	name, err := exeBaseNameFromPID(CallerID)
	if err != nil {
		return "", true
	}
	return name, false
}

// GetWindowTitle reads the caption text of the window identified by hwnd (not a process ID).
func GetWindowTitle(hwnd uintptr) (string, bool) {
	if hwnd == 0 {
		return "", true
	}
	var buf [512]uint16
	_, _, _ = GetWindowHeadLineText.Call(hwnd, uintptr(unsafe.Pointer(&buf[0])), uintptr(len(buf)))
	return syscall.UTF16ToString(buf[:]), false
}

// GetForegroundWindowTitle returns the title bar text of the current foreground window.
func GetForegroundWindowTitle() (title string, errorBool bool) {
	hwnd, err := GetHWND()
	if err {
		return "", true
	}
	title, err = GetWindowTitle(hwnd)
	if err {
		return "", true
	}
	return title, false
}
