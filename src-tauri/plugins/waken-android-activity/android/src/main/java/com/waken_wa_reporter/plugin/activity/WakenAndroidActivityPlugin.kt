package com.waken_wa_reporter.plugin.activity

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.PluginCall

@TauriPlugin
class WakenAndroidActivityPlugin(private val activity: Activity) : Plugin(activity) {
  init {
    AndroidActivityCollector.initialize(activity.applicationContext)
  }

  @Command
  fun startForegroundService(call: PluginCall) {
    val title = call.getString("title") ?: "Waken-Wa Reporter"
    val body = call.getString("body") ?: ""
    val summary = call.getString("summary") ?: ""

    try {
      WakenForegroundService.startService(activity.applicationContext, title, body, summary)
      val result = JSObject()
      result.put("success", true)
      call.resolve(result)
    } catch (e: Exception) {
      call.reject("Failed to start foreground service", e)
    }
  }

  @Command
  fun stopForegroundService(call: PluginCall) {
    try {
      WakenForegroundService.stopService(activity.applicationContext)
      val result = JSObject()
      result.put("success", true)
      call.resolve(result)
    } catch (e: Exception) {
      call.reject("Failed to stop foreground service", e)
    }
  }
}
