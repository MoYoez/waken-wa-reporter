package com.waken_wa_reporter.plugin.activity

import android.app.Activity
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
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
  var running: Boolean = false
  var pauseLabel: String? = null
  var resumeLabel: String? = null
}

@TauriPlugin
class WakenAndroidActivityPlugin(private val activity: Activity) : Plugin(activity) {
  private val reporterActionReceiver = object : BroadcastReceiver() {
    override fun onReceive(context: Context?, intent: Intent?) {
      val action = when (intent?.action) {
        WakenForegroundService.ACTION_PAUSE_REPORTER -> "pause"
        WakenForegroundService.ACTION_RESUME_REPORTER -> "resume"
        else -> return
      }
      val payload = JSObject().put("action", action)
      trigger("reporter-action", payload)
    }
  }

  init {
    AndroidActivityCollector.initialize(activity.applicationContext)
    registerReporterActionReceiver()
  }

  private fun registerReporterActionReceiver() {
    val filter = IntentFilter().apply {
      addAction(WakenForegroundService.ACTION_PAUSE_REPORTER)
      addAction(WakenForegroundService.ACTION_RESUME_REPORTER)
    }
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      activity.registerReceiver(
        reporterActionReceiver,
        filter,
        Context.RECEIVER_NOT_EXPORTED,
      )
    } else {
      @Suppress("UnspecifiedRegisterReceiverFlag")
      activity.registerReceiver(reporterActionReceiver, filter)
    }
  }

  @Command
  fun startForegroundService(invoke: Invoke) {
    val args = invoke.parseArgs(StartForegroundServiceArgs::class.java)
    val title = args.title ?: "Waken-Wa Reporter"
    val body = args.body ?: ""
    val summary = args.summary ?: ""
    val running = args.running
    val pauseLabel = args.pauseLabel ?: "Pause"
    val resumeLabel = args.resumeLabel ?: "Start"

    try {
      WakenForegroundService.startService(
        activity.applicationContext,
        title,
        body,
        summary,
        running,
        pauseLabel,
        resumeLabel,
      )
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
