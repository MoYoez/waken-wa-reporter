#import "macos_bridge.h"

#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <dispatch/dispatch.h>
#import <stdlib.h>

void MRMediaRemoteGetNowPlayingInfo(dispatch_queue_t queue, void (^handler)(NSDictionary *info));

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

char *waken_media_now_playing_json(void) {
    dispatch_semaphore_t sem = dispatch_semaphore_create(0);
    __block char *result = NULL;

    MRMediaRemoteGetNowPlayingInfo(
        dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0),
        ^(NSDictionary *info) {
            if (!info) {
                dispatch_semaphore_signal(sem);
                return;
            }

            NSString *title = info[@"kMRMediaRemoteNowPlayingInfoTitle"] ?: @"";
            NSString *artist = info[@"kMRMediaRemoteNowPlayingInfoArtist"] ?: @"";
            NSString *album = info[@"kMRMediaRemoteNowPlayingInfoAlbum"] ?: @"";

            NSDictionary *payload = @{
                @"title": title,
                @"artist": artist,
                @"album": album,
                @"sourceAppId": @"MediaRemote"
            };

            NSError *error = nil;
            NSData *jsonData = [NSJSONSerialization dataWithJSONObject:payload options:0 error:&error];
            if (!error && jsonData) {
                NSString *jsonString = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
                result = strdup([jsonString UTF8String]);
            }
            dispatch_semaphore_signal(sem);
        }
    );

    dispatch_semaphore_wait(sem, dispatch_time(DISPATCH_TIME_NOW, (int64_t)(2 * NSEC_PER_SEC)));
    return result;
}

void waken_string_free(char *value) {
    if (value != NULL) {
        free(value);
    }
}
