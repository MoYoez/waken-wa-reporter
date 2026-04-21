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

pub(super) fn public_feed_status_error(locale: BackendLocale, status: u16, body: &str) -> String {
    if body.is_empty() {
        if locale.is_en() {
            format!("Public activity returned HTTP {status}")
        } else {
            format!("公开活动接口返回状态码 {status}")
        }
    } else if locale.is_en() {
        format!("Public activity returned HTTP {status}: {body}")
    } else {
        format!("公开活动接口返回状态码 {status}：{body}")
    }
}

pub(super) fn default_public_feed_failure(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Public activity returned failure".into()
    } else {
        "公开活动接口返回失败".into()
    }
}

pub(super) fn discord_worker_stopping(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord sync is still stopping. Try again shortly.".into()
    } else {
        "Discord 同步仍在退出，请稍后重试。".into()
    }
}

pub(super) fn discord_ipc_unavailable(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord IPC is unavailable. Make sure Discord Desktop is running.".into()
    } else {
        "Discord IPC 不可用，请确认桌面端 Discord 已运行。".into()
    }
}

pub(super) fn discord_config_base_url_missing(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Site URL is required before Discord sync can start.".into()
    } else {
        "缺少站点地址，无法启动 Discord 同步。".into()
    }
}

pub(super) fn discord_config_app_id_missing(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord Application ID is required before Discord sync can start.".into()
    } else {
        "缺少 Discord Application ID，无法启动 Discord 同步。".into()
    }
}

pub(super) fn discord_config_source_id_missing(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord source ID is required before Discord sync can start.".into()
    } else {
        "缺少 Discord 来源标识，无法启动 Discord 同步。".into()
    }
}
