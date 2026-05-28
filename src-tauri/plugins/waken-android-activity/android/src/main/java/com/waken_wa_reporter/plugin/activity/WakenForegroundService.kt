package com.waken_wa_reporter.plugin.activity

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat

class WakenForegroundService : Service() {

    companion object {
        const val CHANNEL_ID = "waken-wa-reporter-status-v2"
        const val NOTIFICATION_ID = 408201
        const val ACTION_START = "com.waken_wa_reporter.plugin.activity.START_FOREGROUND"
        const val ACTION_STOP = "com.waken_wa_reporter.plugin.activity.STOP_FOREGROUND"
        const val ACTION_PAUSE_REPORTER =
            "com.waken_wa_reporter.plugin.activity.PAUSE_REPORTER"
        const val ACTION_RESUME_REPORTER =
            "com.waken_wa_reporter.plugin.activity.RESUME_REPORTER"
        const val EXTRA_TITLE = "title"
        const val EXTRA_BODY = "body"
        const val EXTRA_SUMMARY = "summary"
        const val EXTRA_RUNNING = "running"
        const val EXTRA_PAUSE_LABEL = "pauseLabel"
        const val EXTRA_RESUME_LABEL = "resumeLabel"

        fun startService(
            context: Context,
            title: String,
            body: String,
            summary: String,
            running: Boolean,
            pauseLabel: String,
            resumeLabel: String,
        ) {
            val intent = Intent(context, WakenForegroundService::class.java).apply {
                action = ACTION_START
                putExtra(EXTRA_TITLE, title)
                putExtra(EXTRA_BODY, body)
                putExtra(EXTRA_SUMMARY, summary)
                putExtra(EXTRA_RUNNING, running)
                putExtra(EXTRA_PAUSE_LABEL, pauseLabel)
                putExtra(EXTRA_RESUME_LABEL, resumeLabel)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stopService(context: Context) {
            val intent = Intent(context, WakenForegroundService::class.java).apply {
                action = ACTION_STOP
            }
            context.startService(intent)
        }
    }

    private var isRunning = false

    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> {
                val title = intent.getStringExtra(EXTRA_TITLE) ?: "Waken-Wa Reporter"
                val body = intent.getStringExtra(EXTRA_BODY) ?: ""
                val summary = intent.getStringExtra(EXTRA_SUMMARY) ?: ""
                val running = intent.getBooleanExtra(EXTRA_RUNNING, false)
                val pauseLabel = intent.getStringExtra(EXTRA_PAUSE_LABEL) ?: "Pause"
                val resumeLabel = intent.getStringExtra(EXTRA_RESUME_LABEL) ?: "Start"
                startForegroundNotification(title, body, summary, running, pauseLabel, resumeLabel)
            }
            ACTION_STOP -> {
                stopForegroundService()
            }
        }
        return START_STICKY
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Waken-Wa Reporter Status",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Shows the background sync status of Waken-Wa Reporter"
                setShowBadge(false)
                enableLights(false)
                enableVibration(false)
            }

            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager.createNotificationChannel(channel)
        }
    }

    private fun startForegroundNotification(
        title: String,
        body: String,
        summary: String,
        running: Boolean,
        pauseLabel: String,
        resumeLabel: String,
    ) {
        val notification = buildNotification(title, body, summary, running, pauseLabel, resumeLabel)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC,
            )
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
        isRunning = true
    }

    private fun stopForegroundService() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            stopForeground(STOP_FOREGROUND_REMOVE)
        } else {
            @Suppress("DEPRECATION")
            stopForeground(true)
        }
        stopSelf()
        isRunning = false
    }

    private fun buildNotification(
        title: String,
        body: String,
        summary: String,
        running: Boolean,
        pauseLabel: String,
        resumeLabel: String,
    ): Notification {
        val intent = packageManager.getLaunchIntentForPackage(packageName)?.apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
        }
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val actionIntent = Intent(
            if (running) ACTION_PAUSE_REPORTER else ACTION_RESUME_REPORTER,
        ).apply {
            setPackage(packageName)
        }
        val actionPendingIntent = PendingIntent.getBroadcast(
            this,
            if (running) 1 else 2,
            actionIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val actionIcon = if (running) {
            android.R.drawable.ic_media_pause
        } else {
            android.R.drawable.ic_media_play
        }
        val actionLabel = if (running) pauseLabel else resumeLabel

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(title)
            .setContentText(body)
            .setSubText(summary)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setOngoing(true)
            .setAutoCancel(false)
            .setSilent(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            .setContentIntent(pendingIntent)
            .addAction(actionIcon, actionLabel, actionPendingIntent)
            .build()
    }

    override fun onDestroy() {
        super.onDestroy()
        isRunning = false
    }
}
