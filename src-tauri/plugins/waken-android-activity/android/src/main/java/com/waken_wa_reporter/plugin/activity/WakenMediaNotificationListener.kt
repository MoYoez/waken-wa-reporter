package com.waken_wa_reporter.plugin.activity

import android.service.notification.NotificationListenerService

open class WakenMediaNotificationListener : NotificationListenerService() {
  override fun onListenerConnected() {
    super.onListenerConnected()
    AndroidActivityCollector.initialize(applicationContext)
  }
}
