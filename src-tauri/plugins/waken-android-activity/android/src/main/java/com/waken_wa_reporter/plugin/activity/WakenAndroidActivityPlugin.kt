package com.waken_wa_reporter.plugin.activity

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@TauriPlugin
class WakenAndroidActivityPlugin(private val activity: Activity) : Plugin(activity) {
  init {
    AndroidActivityCollector.initialize(activity.applicationContext)
  }

  @Command
  fun startForegroundService(invoke: Invoke) {
    val title = invoke.getString("title") ?: "Waken-Wa Reporter"
    val body = invoke.getString("body") ?: ""
    val summary = invoke.getString("summary") ?: ""

    try {
      WakenForegroundService.startService(activity.applicationContext, title, body, summary)
      val result = JSObject()
      result.put("success", true)
      invoke.resolve(result)
    } catch (e: Exception) {
      invoke.reject("Failed to start foreground service", e)
    }
  }

  @Command
  fun stopForegroundService(invoke: Invoke) {
    try {
      WakenForegroundService.stopService(activity.applicationContext)
      val result = JSObject()
      result.put("success", true)
      invoke.resolve(result)
    } catch (e: Exception) {
      invoke.reject("Failed to stop foreground service", e)
    }
  }
}
