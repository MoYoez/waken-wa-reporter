#pragma once

#include <stdbool.h>

char *waken_frontmost_app_name(void);
char *waken_frontmost_window_title(void);
char *waken_media_now_playing_json(void);
bool waken_accessibility_is_trusted(void);
bool waken_request_accessibility_permission(void);
void waken_string_free(char *value);
