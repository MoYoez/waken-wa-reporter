package com.waken_wa_reporter.plugin.activity

import android.app.Activity
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin

@TauriPlugin
class WakenAndroidActivityPlugin(private val activity: Activity) : Plugin(activity) {
  init {
    AndroidActivityCollector.initialize(activity.applicationContext)
  }
}
