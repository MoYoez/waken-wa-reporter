use std::{
    env, fs,
    path::{Path, PathBuf},
};

const ANDROID_PROGUARD_SOURCE: &str = "resources/android/proguard-rules.pro";
const ANDROID_PROGUARD_TARGET: &str = "gen/android/app/proguard-rules.pro";

fn main() {
    println!("cargo:rerun-if-changed=src/platform/macos_bridge.m");
    println!("cargo:rerun-if-changed=src/platform/macos_bridge.h");
    println!("cargo:rerun-if-changed={ANDROID_PROGUARD_SOURCE}");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "android" {
        sync_android_proguard_rules();
    }

    if target_os == "macos" {
        cc::Build::new()
            .file("src/platform/macos_bridge.m")
            .flag("-fobjc-arc")
            .compile("waken_wa_macos_bridge");

        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
    }

    tauri_build::build()
}

fn sync_android_proguard_rules() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let source = manifest_dir.join(ANDROID_PROGUARD_SOURCE);
    let target = manifest_dir.join(ANDROID_PROGUARD_TARGET);

    let rules = fs::read_to_string(&source).unwrap_or_else(|error| {
        panic!(
            "failed to read Android proguard source {}: {error}",
            source.display()
        )
    });

    let merged = merge_android_proguard_rules(&target, &rules);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|error| {
            panic!(
                "failed to create Android proguard directory {}: {error}",
                parent.display()
            )
        });
    }

    fs::write(&target, merged).unwrap_or_else(|error| {
        panic!(
            "failed to write Android proguard target {}: {error}",
            target.display()
        )
    });
}

fn merge_android_proguard_rules(target: &Path, managed_rules: &str) -> String {
    let normalized_managed = managed_rules.trim();
    if normalized_managed.is_empty() {
        return fs::read_to_string(target).unwrap_or_default();
    }

    let existing = fs::read_to_string(target).unwrap_or_default();
    if existing.contains(normalized_managed) {
        return existing;
    }

    let mut merged = existing.trim_end().to_string();
    if !merged.is_empty() {
        merged.push_str("\n\n");
    }
    merged.push_str(normalized_managed);
    merged.push('\n');
    merged
}
