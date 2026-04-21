use crate::{
    models::{ApiResult, PlatformSelfTestResult},
    platform,
};

pub fn run_platform_self_test() -> Result<ApiResult<PlatformSelfTestResult>, String> {
    Ok(ApiResult::success(200, platform::run_self_test()))
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
