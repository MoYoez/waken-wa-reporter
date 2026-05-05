use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use serde_json::json;

use crate::{
    models::{ApiResult, PlatformSelfTestResult},
    platform,
};

const PLATFORM_SELF_TEST_TIMEOUT_MS: u64 = 4_000;
static PLATFORM_SELF_TEST_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

struct PlatformSelfTestRunGuard;

impl Drop for PlatformSelfTestRunGuard {
    fn drop(&mut self) {
        PLATFORM_SELF_TEST_IN_FLIGHT.store(false, Ordering::Release);
    }
}

pub async fn run_platform_self_test() -> Result<ApiResult<PlatformSelfTestResult>, String> {
    if PLATFORM_SELF_TEST_IN_FLIGHT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Ok(ApiResult::failure_localized(
            409,
            Some("backendErrors.platformSelfTestAlreadyRunning".to_string()),
            "上一次平台检测仍在等待 Windows API 返回，请稍后重试。",
            None,
            None,
        ));
    }

    let timeout = Duration::from_millis(PLATFORM_SELF_TEST_TIMEOUT_MS);
    let (sender, receiver) = mpsc::channel();

    std::thread::Builder::new()
        .name("platform-self-test".to_string())
        .spawn(move || {
            let _guard = PlatformSelfTestRunGuard;
            let _ = sender.send(platform::run_self_test());
        })
        .map_err(|error| {
            PLATFORM_SELF_TEST_IN_FLIGHT.store(false, Ordering::Release);
            format!("启动平台检测线程失败：{error}")
        })?;

    match tauri::async_runtime::spawn_blocking(move || receiver.recv_timeout(timeout)).await {
        Ok(Ok(result)) => Ok(ApiResult::success(200, result)),
        Ok(Err(mpsc::RecvTimeoutError::Timeout)) => Ok(ApiResult::failure_localized(
            408,
            Some("backendErrors.platformSelfTestTimedOut".to_string()),
            "平台检测超时：某个 Windows 媒体或窗口 API 没有及时返回。",
            None,
            Some(json!({ "timeoutMs": PLATFORM_SELF_TEST_TIMEOUT_MS })),
        )),
        Ok(Err(mpsc::RecvTimeoutError::Disconnected)) => Ok(ApiResult::failure_localized(
            500,
            Some("backendErrors.platformSelfTestFailed".to_string()),
            "平台检测线程在返回结果前结束。",
            None,
            None,
        )),
        Err(error) => Ok(ApiResult::failure_localized(
            500,
            Some("backendErrors.platformSelfTestFailed".to_string()),
            format!("平台检测线程执行失败：{error}"),
            None,
            None,
        )),
    }
}

pub fn request_accessibility_permission() -> Result<ApiResult<bool>, String> {
    match platform::request_accessibility_permission() {
        Ok(granted) => Ok(ApiResult::success(200, granted)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            Some("backendErrors.accessibilityPermissionUnsupported".to_string()),
            error,
            None,
            None,
        )),
    }
}
