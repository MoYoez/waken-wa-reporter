#import "macos_bridge.h"

#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <stdlib.h>

char *waken_frontmost_app_name(void) {
    NSRunningApplication *frontmostApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
    if (!frontmostApp) return NULL;

    NSString *localizedName = frontmostApp.localizedName;
    if (!localizedName || localizedName.length == 0) return NULL;

    return strdup([localizedName UTF8String]);
}

char *waken_frontmost_window_title(void) {
    NSRunningApplication *frontmostApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
    if (!frontmostApp) return NULL;

    pid_t pid = frontmostApp.processIdentifier;
    if (pid <= 0) return NULL;

    AXUIElementRef appElement = AXUIElementCreateApplication(pid);
    if (!appElement) return NULL;

    CFTypeRef focusedWindow = NULL;
    AXError windowError = AXUIElementCopyAttributeValue(
        appElement,
        kAXFocusedWindowAttribute,
        &focusedWindow
    );

    char *result = NULL;
    if (windowError == kAXErrorSuccess && focusedWindow) {
        CFTypeRef titleValue = NULL;
        AXError titleError = AXUIElementCopyAttributeValue(
            (AXUIElementRef)focusedWindow,
            kAXTitleAttribute,
            &titleValue
        );

        if (titleError == kAXErrorSuccess && titleValue) {
            if (CFGetTypeID(titleValue) == CFStringGetTypeID()) {
                NSString *title = (__bridge NSString *)titleValue;
                NSString *trimmed = [title stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
                if (trimmed.length > 0) {
                    result = strdup([trimmed UTF8String]);
                }
            }
            CFRelease(titleValue);
        }

        CFRelease(focusedWindow);
    }

    CFRelease(appElement);
    return result;
}

static NSData *waken_png_data_from_image(NSImage *image) {
    if (!image) return nil;

    NSSize targetSize = NSMakeSize(256.0, 256.0);
    [image setSize:targetSize];

    NSRect proposedRect = NSMakeRect(0, 0, targetSize.width, targetSize.height);
    CGImageRef cgImage = [image CGImageForProposedRect:&proposedRect context:nil hints:nil];
    NSBitmapImageRep *representation = nil;
    if (cgImage) {
        representation = [[NSBitmapImageRep alloc] initWithCGImage:cgImage];
    } else if (image.TIFFRepresentation) {
        representation = [NSBitmapImageRep imageRepWithData:image.TIFFRepresentation];
    }

    if (!representation) return nil;
    return [representation representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
}

char *waken_bundle_app_icon_data_url(const char *bundle_identifier) {
    if (bundle_identifier == NULL) return NULL;

    NSString *bundleId = [NSString stringWithUTF8String:bundle_identifier];
    if (!bundleId || bundleId.length == 0) return NULL;

    NSWorkspace *workspace = [NSWorkspace sharedWorkspace];
    NSImage *icon = nil;

    NSURL *applicationURL = [workspace URLForApplicationWithBundleIdentifier:bundleId];
    if (applicationURL && applicationURL.path.length > 0) {
        icon = [workspace iconForFile:applicationURL.path];
    }

    if (!icon) {
        for (NSRunningApplication *runningApp in workspace.runningApplications) {
            if ([runningApp.bundleIdentifier isEqualToString:bundleId]) {
                NSURL *bundleURL = runningApp.bundleURL ?: runningApp.executableURL;
                if (bundleURL && bundleURL.path.length > 0) {
                    icon = [workspace iconForFile:bundleURL.path];
                }
                break;
            }
        }
    }

    NSData *pngData = waken_png_data_from_image(icon);
    if (!pngData || pngData.length == 0) return NULL;

    NSString *base64 = [pngData base64EncodedStringWithOptions:0];
    if (!base64 || base64.length == 0) return NULL;

    NSString *dataUrl = [NSString stringWithFormat:@"data:image/png;base64,%@", base64];
    return strdup([dataUrl UTF8String]);
}

bool waken_accessibility_is_trusted(void) {
    return AXIsProcessTrusted();
}

bool waken_request_accessibility_permission(void) {
    NSDictionary *options = @{
        (__bridge NSString *)kAXTrustedCheckOptionPrompt : @YES
    };
    return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
}

void waken_string_free(char *value) {
    if (value != NULL) {
        free(value);
    }
}
