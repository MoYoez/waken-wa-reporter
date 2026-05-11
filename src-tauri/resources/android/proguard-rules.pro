# Keep Android JNI entrypoints used by Rust platform self-test.
-keep class com.waken_wa_reporter_rustc.app.AndroidActivityCollector {
    public static void initialize(android.content.Context);
    public static java.lang.String getPermissionStatus();
    public static java.lang.String getForegroundSnapshot(boolean, boolean);
    public static java.lang.String getNowPlaying(boolean, boolean, boolean, boolean);
    public static java.lang.String getPowerInfo();
    public static java.lang.String openRequiredPermissionSettings();
}
