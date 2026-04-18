use tauri::AppHandle;

use crate::state_store;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendLocale {
    ZhCn,
    EnUs,
}

impl BackendLocale {
    pub fn from_preference(value: &str) -> Self {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized.starts_with("en") {
            Self::EnUs
        } else {
            Self::ZhCn
        }
    }

    pub fn is_en(self) -> bool {
        matches!(self, Self::EnUs)
    }
}

pub fn load_locale(app: &AppHandle) -> BackendLocale {
    state_store::load_app_state(app)
        .map(|state| BackendLocale::from_preference(&state.locale))
        .unwrap_or(BackendLocale::ZhCn)
}
