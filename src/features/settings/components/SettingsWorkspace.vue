<script setup lang="ts">
import ConnectionPanel from "@/features/settings/components/ConnectionPanel.vue";
import SettingsDiscordCard from "@/features/settings/components/SettingsDiscordCard.vue";
import SettingsHeroPanel from "@/features/settings/components/SettingsHeroPanel.vue";
import SettingsLanguageCard from "@/features/settings/components/SettingsLanguageCard.vue";
import SettingsMobileCard from "@/features/settings/components/SettingsMobileCard.vue";
import SettingsReporterCard from "@/features/settings/components/SettingsReporterCard.vue";
import SettingsSelfTestCard from "@/features/settings/components/SettingsSelfTestCard.vue";
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
  autostartSupported,
  canRequestAccessibilityPermission,
  configReady,
  discordConfigIssues,
  discordConfigReady,
  discordSupported,
  handleRequestAccessibilityPermission,
  handleRestartApp,
  handleSelfTest,
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
</script>

<template>
  <div class="workspace-grid">
    <SettingsHeroPanel
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
      :can-request-accessibility-permission="canRequestAccessibilityPermission"
      :self-test-loading="selfTestLoading"
      :accessibility-permission-loading="accessibilityPermissionLoading"
      @start="$emit('startReporter')"
      @stop="$emit('stopReporter')"
      @self-test="handleSelfTest"
      @request-accessibility-permission="handleRequestAccessibilityPermission"
    />

    <SettingsMobileCard v-else />

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

    <SettingsSelfTestCard
      v-if="selfTestSupported && selfTestResult"
      :platform="selfTestResult.platform"
      :cards="selfTestCards"
      :hint-key="selfTestPlatformHintKey"
      :accessibility-permission-loading="accessibilityPermissionLoading"
      @request-accessibility-permission="handleRequestAccessibilityPermission"
    />
  </div>
</template>
