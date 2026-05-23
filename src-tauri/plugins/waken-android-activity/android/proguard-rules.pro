# Keep the static collector entry points called from Rust through JNI.
-keep class com.waken_wa_reporter.plugin.activity.AndroidActivityCollector {
    public static java.lang.String getPermissionStatus();
    public static java.lang.String getForegroundSnapshot(boolean, boolean);
    public static java.lang.String getNowPlaying(boolean, boolean, boolean, boolean);
    public static java.lang.String getPowerInfo();
}
