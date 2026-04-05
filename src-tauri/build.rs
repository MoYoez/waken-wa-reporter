fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("src/platform/macos_bridge.m")
            .flag("-fobjc-arc")
            .compile("waken_wa_macos_bridge");

        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
        println!("cargo:rustc-link-lib=framework=MediaRemote");
    }

    tauri_build::build()
}
