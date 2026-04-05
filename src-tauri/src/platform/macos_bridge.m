#import "macos_bridge.h"

#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <dispatch/dispatch.h>
#import <stdlib.h>

void MRMediaRemoteGetNowPlayingInfo(dispatch_queue_t queue, void (^handler)(NSDictionary *info));

char *waken_frontmost_app_name(void) {
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
            if (layerValue == 0) {
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
