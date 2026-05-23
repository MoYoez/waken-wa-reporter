use std::ffi::{c_char, CStr, CString};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

unsafe extern "C" {
    fn waken_frontmost_app_name() -> *mut c_char;
    fn waken_frontmost_window_title() -> *mut c_char;
    fn waken_bundle_app_name(bundle_identifier: *const c_char) -> *mut c_char;
    fn waken_bundle_app_icon_data_url(bundle_identifier: *const c_char) -> *mut c_char;
    fn waken_accessibility_is_trusted() -> bool;
    fn waken_request_accessibility_permission() -> bool;
    fn waken_register_foreground_change_observer(callback: extern "C" fn());
    fn waken_string_free(value: *mut c_char);
}

static FOREGROUND_SUBSCRIBERS: OnceLock<Mutex<Vec<Sender<()>>>> = OnceLock::new();

extern "C" fn foreground_change_trampoline() {
    if let Some(lock) = FOREGROUND_SUBSCRIBERS.get() {
        if let Ok(mut subscribers) = lock.lock() {
            subscribers.retain(|sender| sender.send(()).is_ok());
        }
    }
}

pub(super) fn subscribe_foreground_changes() -> Receiver<()> {
    let lock = FOREGROUND_SUBSCRIBERS.get_or_init(|| {
        unsafe { waken_register_foreground_change_observer(foreground_change_trampoline) };
        Mutex::new(Vec::new())
    });
    let (tx, rx) = channel();
    if let Ok(mut subscribers) = lock.lock() {
        subscribers.push(tx);
    }
    rx
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

pub(super) fn read_bundle_app_name(bundle_identifier: &str) -> Option<String> {
    let name = read_bridge_string_with_arg(waken_bundle_app_name, bundle_identifier)?;
    (!name.trim().is_empty()).then_some(name)
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
