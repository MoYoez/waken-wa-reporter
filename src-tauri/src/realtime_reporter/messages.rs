use crate::backend_locale::BackendLocale;

pub(super) fn format_error(
    locale: BackendLocale,
    zh_prefix: &str,
    en_prefix: &str,
    error: impl std::fmt::Display,
) -> String {
    if locale.is_en() {
        format!("{en_prefix}: {error}")
    } else {
        format!("{zh_prefix}：{error}")
    }
}

pub(super) fn reporter_worker_stopping(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Background sync is still stopping. Try again shortly.".into()
    } else {
        "后台同步仍在退出，请稍后重试。".into()
    }
}

pub(super) fn reporter_config_base_url_missing(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Site URL is required before background sync can start.".into()
    } else {
        "缺少站点地址，无法启动后台同步。".into()
    }
}

pub(super) fn reporter_config_api_token_missing(locale: BackendLocale) -> String {
    if locale.is_en() {
        "API Token is required before background sync can start.".into()
    } else {
        "缺少 API Token，无法启动后台同步。".into()
    }
}

pub(super) fn reporter_config_generated_hash_key_missing(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Device key is required before background sync can start.".into()
    } else {
        "缺少 GeneratedHashKey，无法启动后台同步。".into()
    }
}

pub(super) fn server_status_error(locale: BackendLocale, status: u16, body: &str) -> String {
    if body.is_empty() {
        if locale.is_en() {
            format!("Server returned HTTP {status}")
        } else {
            format!("服务端返回状态码 {status}")
        }
    } else if locale.is_en() {
        format!("Server returned HTTP {status}: {body}")
    } else {
        format!("服务端返回状态码 {status}：{body}")
    }
}

pub(super) fn pending_approval_default(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Device is pending approval".into()
    } else {
        "设备待后台审核后可用".into()
    }
}

pub(super) fn server_failure_default(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Server returned failure".into()
    } else {
        "服务端返回失败".into()
    }
}
