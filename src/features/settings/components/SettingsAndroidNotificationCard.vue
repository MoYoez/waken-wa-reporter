<script setup lang="ts">
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";
import ToggleSwitch from "primevue/toggleswitch";
import { useI18n } from "vue-i18n";

defineProps<{
  enabled: boolean;
  permissionGranted: boolean;
  permissionLoading: boolean;
}>();

const emit = defineEmits<{
  updateEnabled: [value: boolean];
  requestPermission: [];
}>();

const { t } = useI18n();
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <h3>{{ t("settings.mobile.androidNotificationSettings") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div class="settings-section">
        <div class="settings-section-head">
          <strong>{{ t("settings.reporter.androidNotification") }}</strong>
          <span>{{ t("settings.mobile.androidNotificationSettingsHint") }}</span>
        </div>

        <div class="reporter-enabled-card field-span-2">
          <div class="reporter-enabled-copy">
            <span class="field-label">{{ t("settings.reporter.androidNotification") }}</span>
            <strong>
              {{
                enabled
                  ? t("settings.tags.enabled")
                  : t("settings.tags.disabled")
              }}
            </strong>
            <span>{{ t("settings.reporter.androidNotificationHint") }}</span>
          </div>
          <ToggleSwitch
            :model-value="enabled"
            input-id="android-reporter-notification"
            @update:model-value="emit('updateEnabled', Boolean($event))"
          />
        </div>

        <div class="reporter-enabled-card field-span-2">
          <div class="reporter-enabled-copy">
            <span class="field-label">{{ t("settings.mobile.androidNotificationPermission") }}</span>
            <strong>
              {{
                permissionGranted
                  ? t("settings.mobile.permissionGranted")
                  : t("settings.mobile.permissionRequired")
              }}
            </strong>
            <span>{{ t("settings.mobile.androidNotificationPermissionHint") }}</span>
          </div>
          <Button
            v-if="!permissionGranted"
            :label="t('settings.mobile.requestAndroidNotificationPermission')"
            icon="pi pi-bell"
            severity="secondary"
            outlined
            :loading="permissionLoading"
            @click="emit('requestPermission')"
          />
        </div>
      </div>

      <div class="message-stack">
        <Message
          v-if="enabled && !permissionGranted"
          severity="warn"
          :closable="false"
        >
          {{ t("settings.notify.androidNotificationPermissionRequired") }}
        </Message>
      </div>
    </template>
  </Card>
</template>
