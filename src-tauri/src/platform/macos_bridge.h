#pragma once

#include <stdbool.h>

char *waken_frontmost_app_name(void);
char *waken_frontmost_window_title(void);
char *waken_bundle_app_name(const char *bundle_identifier);
char *waken_bundle_app_icon_data_url(const char *bundle_identifier);
bool waken_accessibility_is_trusted(void);
bool waken_request_accessibility_permission(void);
void waken_string_free(char *value);
