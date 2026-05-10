package com.waken_wa_reporter_rustc.app

import android.app.AppOpsManager
import android.app.usage.UsageEvents
import android.app.usage.UsageStatsManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.Drawable
import android.media.MediaMetadata
import android.media.session.MediaController
import android.media.session.MediaSessionManager
import android.media.session.PlaybackState
import android.net.Uri
import android.os.BatteryManager
import android.os.Build
import android.os.Process
import android.os.SystemClock
import android.provider.Settings
import android.util.Base64
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import kotlin.math.roundToLong

object AndroidActivityCollector {
  private const val USAGE_LOOKBACK_MS = 10 * 60 * 1000L
  private const val MAX_COVER_SIZE_PX = 384
  private const val MAX_ICON_SIZE_PX = 96

  @Volatile
  private var appContext: Context? = null

  fun initialize(context: Context) {
    appContext = context.applicationContext
  }

  @JvmStatic
  fun getPermissionStatus(): String = withContextJson {
    JSONObject()
      .put("usageAccessGranted", hasUsageAccess(it))
      .put("notificationListenerGranted", hasNotificationListenerAccess(it))
      .toString()
  }

  @JvmStatic
  fun getForegroundSnapshot(includeProcessName: Boolean, includeProcessTitle: Boolean): String =
    withContextJson { context ->
      val result = JSONObject()
        .put("usageAccessGranted", hasUsageAccess(context))

      if (!hasUsageAccess(context)) {
        return@withContextJson result.toString()
      }

      val packageName = findForegroundPackageName(context).orEmpty()
      val appName = if (packageName.isNotBlank()) appLabel(context, packageName) else ""
      result
        .put("processName", if (includeProcessName) packageName else "")
        .put("processTitle", if (includeProcessTitle) appName else "")
        .toString()
    }

  @JvmStatic
  fun getNowPlaying(
    includeMedia: Boolean,
    includePlaySource: Boolean,
    includeArtwork: Boolean,
    includeSourceIcon: Boolean,
  ): String = withContextJson { context ->
    val result = JSONObject()
      .put("notificationListenerGranted", hasNotificationListenerAccess(context))

    if (!hasNotificationListenerAccess(context)) {
      return@withContextJson result.toString()
    }

    val controller = findBestMediaController(context)
      ?: return@withContextJson result.toString()

    val metadata = controller.metadata
    val playbackState = controller.playbackState
    val sourcePackage = controller.packageName.orEmpty()

    if (includeMedia) {
      result
        .put("title", metadataText(metadata, MediaMetadata.METADATA_KEY_TITLE)
          ?: metadataText(metadata, MediaMetadata.METADATA_KEY_DISPLAY_TITLE)
          ?: "")
        .put("artist", metadataText(metadata, MediaMetadata.METADATA_KEY_ARTIST)
          ?: metadataText(metadata, MediaMetadata.METADATA_KEY_ALBUM_ARTIST)
          ?: metadataText(metadata, MediaMetadata.METADATA_KEY_AUTHOR)
          ?: "")
        .put("album", metadataText(metadata, MediaMetadata.METADATA_KEY_ALBUM).orEmpty())
        .put("playbackState", playbackStateName(playbackState))
        .put("reportedAtMs", System.currentTimeMillis())

      playbackPosition(playbackState)?.let { result.put("positionMs", it) }
      metadata?.getLong(MediaMetadata.METADATA_KEY_DURATION)
        ?.takeIf { it >= 0 }
        ?.let { result.put("durationMs", it) }

      if (includeArtwork) {
        val artwork = metadataBitmap(metadata, MediaMetadata.METADATA_KEY_ART)
          ?: metadataBitmap(metadata, MediaMetadata.METADATA_KEY_ALBUM_ART)
          ?: metadataBitmap(metadata, MediaMetadata.METADATA_KEY_DISPLAY_ICON)
        artwork?.let { bitmap ->
          bitmapToDataUrl(bitmap, MAX_COVER_SIZE_PX)?.let { result.put("coverDataUrl", it) }
        }
      }
    }

    if (includePlaySource || includeSourceIcon) {
      result
        .put("sourceAppId", sourcePackage)
        .put("sourceAppName", appLabel(context, sourcePackage))
    }

    if (includeSourceIcon && sourcePackage.isNotBlank()) {
      appIcon(context, sourcePackage)
        ?.let { drawableToBitmap(it, MAX_ICON_SIZE_PX) }
        ?.let { bitmapToDataUrl(it, MAX_ICON_SIZE_PX) }
        ?.let { result.put("appIconDataUrl", it) }
    }

    result.toString()
  }

  @JvmStatic
  fun getPowerInfo(): String = withContextJson { context ->
    val battery = context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
    val level = battery?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
    val scale = battery?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1
    val status = battery?.getIntExtra(BatteryManager.EXTRA_STATUS, -1) ?: -1
    val charging = status == BatteryManager.BATTERY_STATUS_CHARGING
      || status == BatteryManager.BATTERY_STATUS_FULL

    JSONObject()
      .put("batteryLevel", if (level >= 0 && scale > 0) (level * 100L / scale) else JSONObject.NULL)
      .put("isCharging", charging)
      .toString()
  }

  @JvmStatic
  fun openRequiredPermissionSettings(): String = withContextJson { context ->
    val notificationGranted = hasNotificationListenerAccess(context)
    val usageGranted = hasUsageAccess(context)
    val intent = when {
      !notificationGranted -> Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS)
      !usageGranted -> Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS)
      else -> Intent(
        Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
        Uri.parse("package:${context.packageName}"),
      )
    }.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

    context.startActivity(intent)
    JSONObject()
      .put("granted", notificationGranted && usageGranted)
      .toString()
  }

  private fun withContextJson(block: (Context) -> String): String {
    val context = appContext ?: return JSONObject()
      .put("error", "Android context is not initialized.")
      .toString()

    return try {
      block(context)
    } catch (error: Throwable) {
      JSONObject()
        .put("error", error.message ?: error.javaClass.simpleName)
        .toString()
    }
  }

  private fun hasUsageAccess(context: Context): Boolean {
    val appOps = context.getSystemService(Context.APP_OPS_SERVICE) as AppOpsManager
    val mode = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
      appOps.unsafeCheckOpNoThrow(
        AppOpsManager.OPSTR_GET_USAGE_STATS,
        Process.myUid(),
        context.packageName,
      )
    } else {
      @Suppress("DEPRECATION")
      appOps.checkOpNoThrow(
        AppOpsManager.OPSTR_GET_USAGE_STATS,
        Process.myUid(),
        context.packageName,
      )
    }

    return mode == AppOpsManager.MODE_ALLOWED
  }

  private fun hasNotificationListenerAccess(context: Context): Boolean {
    val enabled = Settings.Secure.getString(
      context.contentResolver,
      "enabled_notification_listeners",
    ) ?: return false

    return enabled
      .split(':')
      .mapNotNull { ComponentName.unflattenFromString(it) }
      .any {
        it.packageName == context.packageName
          && it.className == WakenMediaNotificationListener::class.java.name
      }
  }

  private fun findForegroundPackageName(context: Context): String? {
    val usageStats = context.getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
    val end = System.currentTimeMillis()
    val events = usageStats.queryEvents(end - USAGE_LOOKBACK_MS, end)
    val event = UsageEvents.Event()
    var packageName: String? = null

    while (events.hasNextEvent()) {
      events.getNextEvent(event)
      if (isForegroundEvent(event.eventType) && !event.packageName.isNullOrBlank()) {
        packageName = event.packageName
      }
    }

    return packageName
  }

  private fun isForegroundEvent(eventType: Int): Boolean {
    return eventType == UsageEvents.Event.MOVE_TO_FOREGROUND
      || (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q
        && eventType == UsageEvents.Event.ACTIVITY_RESUMED)
  }

  private fun findBestMediaController(context: Context): MediaController? {
    val manager = context.getSystemService(Context.MEDIA_SESSION_SERVICE) as MediaSessionManager
    val listener = ComponentName(context, WakenMediaNotificationListener::class.java)
    return manager
      .getActiveSessions(listener)
      .sortedByDescending { controller ->
        val stateScore = when (controller.playbackState?.state) {
          PlaybackState.STATE_PLAYING -> 3
          PlaybackState.STATE_BUFFERING,
          PlaybackState.STATE_CONNECTING,
          PlaybackState.STATE_FAST_FORWARDING,
          PlaybackState.STATE_REWINDING,
          PlaybackState.STATE_SKIPPING_TO_NEXT,
          PlaybackState.STATE_SKIPPING_TO_PREVIOUS,
          PlaybackState.STATE_SKIPPING_TO_QUEUE_ITEM -> 2
          PlaybackState.STATE_PAUSED -> 1
          else -> 0
        }
        val metadataScore = if (controller.metadata != null) 1 else 0
        stateScore * 10 + metadataScore
      }
      .firstOrNull { it.metadata != null || it.playbackState != null }
  }

  private fun metadataText(metadata: MediaMetadata?, key: String): String? {
    return metadata?.getString(key)?.trim()?.takeIf { it.isNotEmpty() }
  }

  private fun metadataBitmap(metadata: MediaMetadata?, key: String): Bitmap? {
    return metadata?.getBitmap(key)
  }

  private fun playbackStateName(state: PlaybackState?): String {
    return when (state?.state) {
      PlaybackState.STATE_PLAYING,
      PlaybackState.STATE_BUFFERING,
      PlaybackState.STATE_CONNECTING,
      PlaybackState.STATE_FAST_FORWARDING,
      PlaybackState.STATE_REWINDING,
      PlaybackState.STATE_SKIPPING_TO_NEXT,
      PlaybackState.STATE_SKIPPING_TO_PREVIOUS,
      PlaybackState.STATE_SKIPPING_TO_QUEUE_ITEM -> "playing"
      PlaybackState.STATE_PAUSED -> "paused"
      PlaybackState.STATE_STOPPED,
      PlaybackState.STATE_NONE -> "stopped"
      else -> ""
    }
  }

  private fun playbackPosition(state: PlaybackState?): Long? {
    val basePosition = state?.position?.takeIf { it >= 0 } ?: return null
    if (state.state != PlaybackState.STATE_PLAYING || state.lastPositionUpdateTime <= 0) {
      return basePosition
    }

    val elapsed = SystemClock.elapsedRealtime() - state.lastPositionUpdateTime
    if (elapsed <= 0) {
      return basePosition
    }

    return (basePosition + elapsed * state.playbackSpeed).roundToLong().coerceAtLeast(0)
  }

  private fun appLabel(context: Context, packageName: String): String {
    if (packageName.isBlank()) {
      return ""
    }

    return runCatching {
      val info = applicationInfo(context, packageName)
      context.packageManager.getApplicationLabel(info).toString()
    }.getOrDefault(packageName)
  }

  private fun appIcon(context: Context, packageName: String): Drawable? {
    if (packageName.isBlank()) {
      return null
    }

    return runCatching {
      context.packageManager.getApplicationIcon(packageName)
    }.getOrNull()
  }

  private fun applicationInfo(context: Context, packageName: String): ApplicationInfo {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      context.packageManager.getApplicationInfo(
        packageName,
        PackageManager.ApplicationInfoFlags.of(0L),
      )
    } else {
      @Suppress("DEPRECATION")
      context.packageManager.getApplicationInfo(packageName, 0)
    }
  }

  private fun drawableToBitmap(drawable: Drawable, maxSize: Int): Bitmap {
    if (drawable is BitmapDrawable && drawable.bitmap != null) {
      return scaleBitmap(drawable.bitmap, maxSize)
    }

    val width = drawable.intrinsicWidth.takeIf { it > 0 } ?: maxSize
    val height = drawable.intrinsicHeight.takeIf { it > 0 } ?: maxSize
    val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bitmap)
    drawable.setBounds(0, 0, canvas.width, canvas.height)
    drawable.draw(canvas)
    return scaleBitmap(bitmap, maxSize)
  }

  private fun bitmapToDataUrl(bitmap: Bitmap, maxSize: Int): String? {
    val scaled = scaleBitmap(bitmap, maxSize)
    val output = ByteArrayOutputStream()
    if (!scaled.compress(Bitmap.CompressFormat.PNG, 100, output)) {
      return null
    }

    return "data:image/png;base64,${Base64.encodeToString(output.toByteArray(), Base64.NO_WRAP)}"
  }

  private fun scaleBitmap(bitmap: Bitmap, maxSize: Int): Bitmap {
    val longestSide = maxOf(bitmap.width, bitmap.height)
    if (longestSide <= maxSize) {
      return bitmap
    }

    val scale = maxSize.toFloat() / longestSide.toFloat()
    val width = (bitmap.width * scale).roundToLong().toInt().coerceAtLeast(1)
    val height = (bitmap.height * scale).roundToLong().toInt().coerceAtLeast(1)
    return Bitmap.createScaledBitmap(bitmap, width, height, true)
  }
}
