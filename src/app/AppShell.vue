<script setup lang="ts">
import Toast from "primevue/toast";

import AppSidebar from "@/app/components/AppSidebar.vue";
import MobileTabbar from "@/app/components/MobileTabbar.vue";
import OnboardingDialog from "@/app/components/OnboardingDialog.vue";
import PendingApprovalDialog from "@/app/components/PendingApprovalDialog.vue";
import PendingSaveFloat from "@/app/components/PendingSaveFloat.vue";
import { useAppShell } from "@/app/composables/useAppShell";
import ActivityWorkspace from "@/features/activity/components/ActivityWorkspace.vue";
import InspirationWorkspace from "@/features/inspiration/components/InspirationWorkspace.vue";
import OverviewWorkspace from "@/features/overview/components/OverviewWorkspace.vue";
import RealtimeWorkspace from "@/features/realtime/components/RealtimeWorkspace.vue";
import SettingsWorkspace from "@/features/settings/components/SettingsWorkspace.vue";

const {
  activeSection,
  applyLocale,
  applySettingsChanges,
  capabilities,
  closeOnboarding,
  closePendingApprovalDialog,
  completeOnboardingSetup,
  config,
  currentLocale,
  discordBusy,
  discordPresenceSnapshot,
  discordSupported,
  existingReporterConfig,
  handlePendingApproval,
  handlePresetSaved,
  handleRestartApp,
  handleStartDiscordPresence,
  handleStartReporter,
  handleStopDiscordPresence,
  handleStopReporter,
  hasPendingSettingsChanges,
  importingReporterConfig,
  isNativeNotice,
  isPhone,
  localeRestartRequired,
  mobileConnectivity,
  notifyImport,
  onboardingDraftConfig,
  onboardingSetupMode,
  pendingApprovalDialogVisible,
  readiness,
  recentPresets,
  rememberVerifiedGeneratedHashKey,
  reporterBusy,
  reporterConfigPromptHandled,
  reporterSnapshot,
  reporterSupported,
  revertPendingSettings,
  runMobileConnectivityProbe,
  selectSection,
  settingsRestarting,
  shouldShowOnboarding,
  skipExistingReporterConfig,
  startSetup,
  traySupported,
  updateConfig,
  updateOnboardingDraftConfig,
  useExistingReporterConfig,
  verifiedGeneratedHashKey,
  visibleSections,
} = useAppShell();
</script>

<template>
  <div class="app-root" :class="{ 'has-pending-save': hasPendingSettingsChanges }">
    <Toast v-if="!isNativeNotice" position="top-right" />

    <OnboardingDialog
      :visible="shouldShowOnboarding"
      :setup-mode="onboardingSetupMode"
      :reporter-supported="reporterSupported"
      :existing-reporter-config="existingReporterConfig"
      :reporter-config-prompt-handled="reporterConfigPromptHandled"
      :importing-reporter-config="importingReporterConfig"
      :model-value="onboardingDraftConfig"
      :capabilities="capabilities"
      :verified-generated-hash-key="verifiedGeneratedHashKey"
      @close="closeOnboarding"
      @start-setup="startSetup"
      @skip-existing-config="skipExistingReporterConfig"
      @use-existing-config="useExistingReporterConfig"
      @complete="completeOnboardingSetup"
      @back="onboardingSetupMode = false"
      @update:model-value="updateOnboardingDraftConfig"
      @imported="notifyImport"
    />

    <PendingApprovalDialog
      :visible="pendingApprovalDialogVisible"
      :message="reporterSnapshot.lastPendingApprovalMessage"
      :approval-url="reporterSnapshot.lastPendingApprovalUrl"
      @close="closePendingApprovalDialog"
    />

    <transition name="pending-save-float">
      <PendingSaveFloat
        v-if="hasPendingSettingsChanges"
        @apply="applySettingsChanges"
        @revert="revertPendingSettings"
      />
    </transition>

    <div class="app-shell" :class="{ 'phone-nav': isPhone }">
      <AppSidebar
        v-if="!isPhone"
        :visible-sections="visibleSections"
        :active-section="activeSection"
        :readiness="readiness"
        :reporter-supported="reporterSupported"
        :reporter-running="reporterSnapshot.running"
        :discord-supported="discordSupported"
        :discord-running="discordPresenceSnapshot.running"
        :discord-connected="discordPresenceSnapshot.connected"
        :tray-supported="traySupported"
        @select="selectSection"
      />

      <main class="app-main">
        <OverviewWorkspace
          v-if="activeSection === 'overview'"
          :config="config"
          :readiness="readiness"
          :capabilities="capabilities"
          :mobile-connectivity="mobileConnectivity"
          :reporter-snapshot="reporterSnapshot"
          :discord-presence-snapshot="discordPresenceSnapshot"
          :reporter-busy="reporterBusy"
          @start-reporter="handleStartReporter"
          @stop-reporter="handleStopReporter"
          @retry-mobile-connectivity="runMobileConnectivityProbe(true)"
        />

        <SettingsWorkspace
          v-else-if="activeSection === 'settings'"
          :model-value="config"
          :capabilities="capabilities"
          :reporter-snapshot="reporterSnapshot"
          :discord-presence-snapshot="discordPresenceSnapshot"
          :reporter-busy="reporterBusy"
          :discord-busy="discordBusy"
          :verified-generated-hash-key="verifiedGeneratedHashKey"
          :locale="currentLocale"
          :locale-restart-required="localeRestartRequired"
          :restarting="settingsRestarting"
          @update:model-value="updateConfig"
          @update:locale="applyLocale($event, true)"
          @restart-app="handleRestartApp"
          @imported="notifyImport"
          @start-reporter="handleStartReporter"
          @stop-reporter="handleStopReporter"
          @start-discord-presence="handleStartDiscordPresence"
          @stop-discord-presence="handleStopDiscordPresence"
        />

        <ActivityWorkspace
          v-else-if="activeSection === 'activity'"
          :config="config"
          :capabilities="capabilities"
          :recent-presets="recentPresets"
          @preset-saved="handlePresetSaved"
          @pending-approval="handlePendingApproval"
          @key-verified="rememberVerifiedGeneratedHashKey"
        />

        <RealtimeWorkspace
          v-else-if="activeSection === 'realtime'"
          :snapshot="reporterSnapshot"
        />

        <InspirationWorkspace
          v-else-if="activeSection === 'inspiration'"
          :config="config"
          :capabilities="capabilities"
          @pending-approval="handlePendingApproval"
          @key-verified="rememberVerifiedGeneratedHashKey"
        />
      </main>
    </div>

    <MobileTabbar
      v-if="isPhone"
      :visible-sections="visibleSections"
      :active-section="activeSection"
      @select="selectSection"
    />
  </div>
</template>
