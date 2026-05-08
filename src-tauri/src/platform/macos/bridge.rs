use std::ffi::{c_char, CStr, CString};

unsafe extern "C" {
    fn waken_frontmost_app_name() -> *mut c_char;
    fn waken_frontmost_window_title() -> *mut c_char;
    fn waken_bundle_app_icon_data_url(bundle_identifier: *const c_char) -> *mut c_char;
    fn waken_accessibility_is_trusted() -> bool;
    fn waken_request_accessibility_permission() -> bool;
    fn waken_string_free(value: *mut c_char);
}

fn read_bridge_string(fetch: unsafe extern "C" fn() -> *mut c_char) -> Option<String> {
    let ptr = unsafe { fetch() };
    if ptr.is_null() {
        return None;
    }

    let value = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
    unsafe { waken_string_free(ptr) };
    Some(value)
}

fn read_bridge_string_with_arg(
    fetch: unsafe extern "C" fn(*const c_char) -> *mut c_char,
    value: &str,
) -> Option<String> {
    let value = CString::new(value).ok()?;
    let ptr = unsafe { fetch(value.as_ptr()) };
    if ptr.is_null() {
        return None;
    }

    let result = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
    unsafe { waken_string_free(ptr) };
    Some(result)
}

pub(super) fn read_frontmost_app_name() -> Option<String> {
    read_bridge_string(waken_frontmost_app_name)
}

pub(super) fn read_frontmost_window_title() -> Option<String> {
    read_bridge_string(waken_frontmost_window_title)
}

pub(super) fn read_bundle_app_icon_data_url(bundle_identifier: &str) -> Option<String> {
    let icon = read_bridge_string_with_arg(waken_bundle_app_icon_data_url, bundle_identifier)?;
    (!icon.trim().is_empty()).then_some(icon)
}

pub(super) fn accessibility_permission_granted() -> bool {
    unsafe { waken_accessibility_is_trusted() }
}

pub(super) fn request_accessibility_permission() -> bool {
    unsafe { waken_request_accessibility_permission() }
}
