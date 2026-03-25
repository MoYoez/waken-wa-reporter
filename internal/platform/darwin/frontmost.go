//go:build darwin

package darwin

/*
#cgo CFLAGS: -x objective-c
#cgo LDFLAGS: -framework CoreGraphics -framework Foundation
#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <stdlib.h>

char* getFrontmostAppName(void) {
    CFArrayRef windowList = CGWindowListCopyWindowInfo(
        kCGWindowListOptionOnScreenOnly | kCGWindowListOptionOnScreenAboveWindow,
        kCGNullWindowID
    );

    if (!windowList) return NULL;

    CFIndex count = CFArrayGetCount(windowList);
    char *result = NULL;

    for (CFIndex i = 0; i < count; i++) {
        CFDictionaryRef window = CFArrayGetValueAtIndex(windowList, i);
        CFNumberRef layer = CFDictionaryGetValue(window, kCGWindowLayer);

        int layerValue = 0;
        if (layer && CFNumberGetValue(layer, kCFNumberIntType, &layerValue)) {
            if (layerValue == 0) { // simple window
                CFStringRef ownerName = CFDictionaryGetValue(window, kCGWindowOwnerName);
                if (ownerName) {
                    NSString *name = (__bridge NSString *)ownerName;
                    result = strdup([name UTF8String]);
                    break;
                }
            }
        }
    }

    CFRelease(windowList);
    return result;
}
*/
import "C"

import (
	"unsafe"
)

// GetForegroundApplicationName returns the name of the frontmost application on macOS.
// Caller should run from main thread (e.g. main goroutine with runtime.LockOSThread) for up-to-date result.
// Returns (name, false) on success, ("", true) on failure.
func GetForegroundApplicationName() (string, bool) {
	cstr := C.getFrontmostAppName()
	if cstr == nil {
		return "", true
	}
	defer C.free(unsafe.Pointer(cstr))
	return C.GoString(cstr), false
}
