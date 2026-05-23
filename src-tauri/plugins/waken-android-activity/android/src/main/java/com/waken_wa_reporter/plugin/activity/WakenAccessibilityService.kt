package com.waken_wa_reporter.plugin.activity

import android.accessibilityservice.AccessibilityService
import android.os.Build
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo

class WakenAccessibilityService : AccessibilityService() {
  override fun onServiceConnected() {
    super.onServiceConnected()
    AndroidActivityCollector.initialize(applicationContext)
  }

  override fun onAccessibilityEvent(event: AccessibilityEvent?) {
    val packageName = event?.packageName?.toString()?.trim().orEmpty()
    if (packageName.isBlank()) {
      return
    }

    val className = event?.className?.toString()?.trim().orEmpty()
    val title = eventTitle(event, packageName, className)
      ?: rootWindowTitle(rootInActiveWindow, packageName, className)
      ?: ""

    AndroidActivityCollector.updateAccessibilityWindow(packageName, title)
  }

  override fun onInterrupt() = Unit

  private fun eventTitle(
    event: AccessibilityEvent?,
    packageName: String,
    className: String,
  ): String? {
    return event
      ?.text
      ?.asSequence()
      ?.mapNotNull { cleanTitleCandidate(it, packageName, className) }
      ?.firstOrNull()
      ?: cleanTitleCandidate(event?.contentDescription, packageName, className)
  }

  private fun rootWindowTitle(
    root: AccessibilityNodeInfo?,
    packageName: String,
    className: String,
  ): String? {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) {
      return null
    }

    return cleanTitleCandidate(root?.window?.title, packageName, className)
  }

  private fun cleanTitleCandidate(
    value: CharSequence?,
    packageName: String,
    className: String,
  ): String? {
    val title = value?.toString()?.trim().orEmpty()
    if (title.isBlank() || title == packageName || title == className) {
      return null
    }

    return title
  }
}
