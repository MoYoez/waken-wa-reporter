use serde_json::{json, Value};

use crate::{
    backend_locale::BackendLocale,
    platform::{ForegroundSnapshot, MediaInfo},
};

pub(super) struct LogTextSpec {
    pub(super) key: Option<&'static str>,
    pub(super) params: Option<Value>,
    pub(super) fallback: String,
}

pub(super) fn fallback_text(value: impl Into<String>) -> LogTextSpec {
    LogTextSpec {
        key: None,
        params: None,
        fallback: value.into(),
    }
}

pub(super) fn localized_text(
    key: &'static str,
    params: Option<Value>,
    fallback: impl Into<String>,
) -> LogTextSpec {
    LogTextSpec {
        key: Some(key),
        params,
        fallback: fallback.into(),
    }
}

pub(super) fn build_log_detail(
    snapshot: &ForegroundSnapshot,
    media: &MediaInfo,
    is_heartbeat: bool,
    locale: BackendLocale,
) -> LogTextSpec {
    let action = if locale.is_en() {
        if is_heartbeat {
            "Heartbeat"
        } else {
            "Report"
        }
    } else if is_heartbeat {
        "心跳"
    } else {
        "上报"
    };

    let base_params = json!({
        "processName": snapshot.process_name,
        "processTitle": snapshot.process_title,
    });

    if media.is_empty() {
        localized_text(
            if is_heartbeat {
                "reporterLogs.activityHeartbeat.detail"
            } else {
                "reporterLogs.activityReported.detail"
            },
            Some(base_params),
            format!(
                "{}{} / {}",
                if locale.is_en() {
                    format!("{action}: ")
                } else {
                    format!("{action}：")
                },
                snapshot.process_name,
                snapshot.process_title
            ),
        )
    } else {
        localized_text(
            if is_heartbeat {
                "reporterLogs.activityHeartbeat.detailWithMedia"
            } else {
                "reporterLogs.activityReported.detailWithMedia"
            },
            Some(json!({
                "processName": snapshot.process_name,
                "processTitle": snapshot.process_title,
                "mediaSummary": media.summary(),
            })),
            format!(
                "{}{} / {} | {}{}",
                if locale.is_en() {
                    format!("{action}: ")
                } else {
                    format!("{action}：")
                },
                snapshot.process_name,
                snapshot.process_title,
                if locale.is_en() { "" } else { "媒体：" },
                media.summary()
            ),
        )
    }
}
