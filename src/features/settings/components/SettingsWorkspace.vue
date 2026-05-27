<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";

import ConnectionPanel from "@/features/settings/components/ConnectionPanel.vue";
import SettingsAboutCard from "@/features/settings/components/SettingsAboutCard.vue";
import SettingsAndroidNotificationCard from "@/features/settings/components/SettingsAndroidNotificationCard.vue";
import SettingsDiscordCard from "@/features/settings/components/SettingsDiscordCard.vue";
import SettingsHeaderPanel from "@/features/settings/components/SettingsHeaderPanel.vue";
import SettingsLanguageCard from "@/features/settings/components/SettingsLanguageCard.vue";
import SettingsMobileCard from "@/features/settings/components/SettingsMobileCard.vue";
import SettingsReporterCard from "@/features/settings/components/SettingsReporterCard.vue";
import SettingsStartupCard from "@/features/settings/components/SettingsStartupCard.vue";
import { useSettingsWorkspace } from "@/features/settings/composables/useSettingsWorkspace";
import { readDevicePlatform } from "@/lib/deviceInfo";
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

const isAndroid = ref(false);

// Check device platform on initialization
readDevicePlatform().then((platform) => {
  isAndroid.value = platform === "android";
});

let permissionRefreshTimer: number | undefined;

function refreshPermissionStateAfterReturn() {
  window.clearTimeout(permissionRefreshTimer);
  permissionRefreshTimer = window.setTimeout(() => {
    void refreshAndroidReporterNotificationPermission({ silent: true });
    void refreshAndroidPermissionStatus({ silent: true });
    void handleSelfTest({ silent: true });
  }, 250);
}

function handleVisibilityChange() {
  if (document.visibilityState === "visible" && isAndroid.value) {
    refreshPermissionStateAfterReturn();
  }
}

function handleWindowFocus() {
  if (isAndroid.value) {
    refreshPermissionStateAfterReturn();
  }
}

onMounted(() => {
  document.addEventListener("visibilitychange", handleVisibilityChange);
  window.addEventListener("focus", handleWindowFocus);
});

// Trigger the initial Android-only permission refresh as soon as the platform
// detection promise resolves. Using onMounted alone races against the async
// readDevicePlatform() IPC and would skip the initial refresh.
watch(
  isAndroid,
  (value) => {
    if (value) {
      void refreshAndroidReporterNotificationPermission({ silent: true });
    }
  },
  { immediate: true },
);

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

    <div class="settings-document">
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
        v-if="isAndroid"
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

      <SettingsAboutCard :updater-supported="capabilities.updater" />
    </div>
  </div>
</template>
