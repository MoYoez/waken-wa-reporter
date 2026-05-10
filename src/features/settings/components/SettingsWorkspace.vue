<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";

import ConnectionPanel from "@/features/settings/components/ConnectionPanel.vue";
import SettingsAndroidNotificationCard from "@/features/settings/components/SettingsAndroidNotificationCard.vue";
import SettingsDiscordCard from "@/features/settings/components/SettingsDiscordCard.vue";
import SettingsHeaderPanel from "@/features/settings/components/SettingsHeaderPanel.vue";
import SettingsLanguageCard from "@/features/settings/components/SettingsLanguageCard.vue";
import SettingsMobileCard from "@/features/settings/components/SettingsMobileCard.vue";
import SettingsReporterCard from "@/features/settings/components/SettingsReporterCard.vue";
import SettingsStartupCard from "@/features/settings/components/SettingsStartupCard.vue";
import { useSettingsWorkspace } from "@/features/settings/composables/useSettingsWorkspace";
import type { SupportedLocale } from "@/i18n";
import type {
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  RealtimeReporterSnapshot,
} from "@/types";

const props = defineProps<{
  modelValue: ClientConfig;
  locale: SupportedLocale;
  capabilities: ClientCapabilities;
  reporterSnapshot: RealtimeReporterSnapshot;
  discordPresenceSnapshot: DiscordPresenceSnapshot;
  reporterBusy: boolean;
  discordBusy: boolean;
  verifiedGeneratedHashKey: string;
  localeRestartRequired: boolean;
  restarting: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: ClientConfig];
  "update:locale": [value: SupportedLocale];
  imported: [message: string];
  startReporter: [];
  stopReporter: [];
  startDiscordPresence: [];
  stopDiscordPresence: [];
  restartApp: [];
}>();

const {
  accessibilityPermissionLoading,
  androidNotificationPermissionLoading,
  androidReporterNotificationPermissionGranted,
  autostartSupported,
  configReady,
  discordConfigIssues,
  discordConfigReady,
  discordSupported,
  handleRequestAndroidReporterNotificationPermission,
  handleRequestPermission,
  handleRestartApp,
  handleSelfTest,
  refreshAndroidReporterNotificationPermission,
  refreshAndroidPermissionStatus,
  reporterSupported,
  selfTestCards,
  selfTestLoading,
  selfTestPlatformHintKey,
  selfTestResult,
  selfTestSupported,
  updateField,
} = useSettingsWorkspace(props, {
  onUpdateModelValue: (value) => emit("update:modelValue", value),
  onRestartApp: () => emit("restartApp"),
});

let permissionRefreshTimer: number | undefined;

function isAndroidRuntime() {
  return typeof navigator !== "undefined" && /Android/i.test(navigator.userAgent);
}

function refreshPermissionStateAfterReturn() {
  window.clearTimeout(permissionRefreshTimer);
  permissionRefreshTimer = window.setTimeout(() => {
    void refreshAndroidReporterNotificationPermission({ silent: true });
    void refreshAndroidPermissionStatus({ silent: true });
    void handleSelfTest({ silent: true });
  }, 250);
}

function handleVisibilityChange() {
  if (document.visibilityState === "visible" && isAndroidRuntime()) {
    refreshPermissionStateAfterReturn();
  }
}

function handleWindowFocus() {
  if (isAndroidRuntime()) {
    refreshPermissionStateAfterReturn();
  }
}

onMounted(() => {
  if (isAndroidRuntime()) {
    void refreshAndroidReporterNotificationPermission({ silent: true });
  }
  document.addEventListener("visibilitychange", handleVisibilityChange);
  window.addEventListener("focus", handleWindowFocus);
});

onBeforeUnmount(() => {
  window.clearTimeout(permissionRefreshTimer);
  document.removeEventListener("visibilitychange", handleVisibilityChange);
  window.removeEventListener("focus", handleWindowFocus);
});
</script>

<template>
  <div class="workspace-grid">
    <SettingsHeaderPanel
      :reporter-supported="reporterSupported"
      :reporter-running="reporterSnapshot.running"
    />

    <SettingsLanguageCard
      :locale="props.locale"
      :locale-restart-required="localeRestartRequired"
      :restarting="restarting"
      @update-locale="emit('update:locale', $event)"
      @restart-app="handleRestartApp"
    />

    <SettingsStartupCard
      v-if="autostartSupported"
      :enabled="modelValue.launchOnStartup"
      @update-enabled="updateField('launchOnStartup', $event)"
    />

    <ConnectionPanel
      :model-value="modelValue"
      :capabilities="capabilities"
      :verified-generated-hash-key="verifiedGeneratedHashKey"
      @update:model-value="$emit('update:modelValue', $event)"
      @imported="$emit('imported', $event)"
    />

    <SettingsReporterCard
      v-if="reporterSupported"
      :config-ready="configReady"
      :snapshot="reporterSnapshot"
      :reporter-busy="reporterBusy"
      :self-test-supported="selfTestSupported"
      :self-test-loading="selfTestLoading"
      :accessibility-permission-loading="accessibilityPermissionLoading"
      :self-test-platform="selfTestResult?.platform || ''"
      :self-test-cards="selfTestCards"
      :self-test-hint-key="selfTestPlatformHintKey"
      @start="$emit('startReporter')"
      @stop="$emit('stopReporter')"
      @self-test="handleSelfTest"
      @request-permission="handleRequestPermission"
    />

    <SettingsAndroidNotificationCard
      v-if="isAndroidRuntime()"
      :enabled="modelValue.androidReporterNotificationEnabled"
      :permission-granted="androidReporterNotificationPermissionGranted"
      :permission-loading="androidNotificationPermissionLoading"
      @update-enabled="updateField('androidReporterNotificationEnabled', $event)"
      @request-permission="handleRequestAndroidReporterNotificationPermission"
    />

    <SettingsMobileCard v-if="!reporterSupported" />

    <SettingsDiscordCard
      v-if="discordSupported"
      :application-id="modelValue.discordApplicationId"
      :enabled="modelValue.discordEnabled"
      :snapshot="discordPresenceSnapshot"
      :discord-busy="discordBusy"
      :issues="discordConfigIssues"
      :can-start="discordConfigReady"
      @update-application-id="updateField('discordApplicationId', $event)"
      @update-enabled="updateField('discordEnabled', $event)"
      @start="$emit('startDiscordPresence')"
      @stop="$emit('stopDiscordPresence')"
    />
  </div>
</template>
