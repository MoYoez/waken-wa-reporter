package com.waken_wa_reporter_rustc.app

import android.service.notification.NotificationListenerService

class WakenMediaNotificationListener : NotificationListenerService() {
  override fun onListenerConnected() {
    super.onListenerConnected()
    AndroidActivityCollector.initialize(applicationContext)
  }
}
