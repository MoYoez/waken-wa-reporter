package com.waken_wa_reporter.plugin.activity

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class StartForegroundServiceArgs {
  var title: String? = null
  var body: String? = null
  var summary: String? = null
}

@TauriPlugin
class WakenAndroidActivityPlugin(private val activity: Activity) : Plugin(activity) {
  init {
    AndroidActivityCollector.initialize(activity.applicationContext)
  }

  @Command
  fun startForegroundService(invoke: Invoke) {
    val args = invoke.parseArgs(StartForegroundServiceArgs::class.java)
    val title = args.title ?: "Waken-Wa Reporter"
    val body = args.body ?: ""
    val summary = args.summary ?: ""

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
