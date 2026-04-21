<script setup lang="ts">
import ActivityFormCard from "@/features/activity/components/ActivityFormCard.vue";
import ActivityRecentPresetsCard from "@/features/activity/components/ActivityRecentPresetsCard.vue";
import { useActivityWorkspace } from "@/features/activity/composables/useActivityWorkspace";
import type {
  ClientCapabilities,
  ClientConfig,
  PendingApprovalInfo,
  RecentPreset,
} from "@/types";

const props = defineProps<{
  config: ClientConfig;
  capabilities: ClientCapabilities;
  recentPresets: RecentPreset[];
}>();

const emit = defineEmits<{
  presetSaved: [preset: RecentPreset];
  pendingApproval: [info: PendingApprovalInfo];
  keyVerified: [generatedHashKey: string];
}>();

const { applyPreset, form, hasConfigIssues, mobileRuntime, submitReport, submitting } =
  useActivityWorkspace(props, {
    presetSaved: (preset) => emit("presetSaved", preset),
    pendingApproval: (info) => emit("pendingApproval", info),
    keyVerified: (generatedHashKey) => emit("keyVerified", generatedHashKey),
  });
</script>

<template>
  <div class="workspace-grid">
    <ActivityFormCard
      :form="form"
      :mobile-runtime="mobileRuntime"
      :has-config-issues="hasConfigIssues"
      :submitting="submitting"
      @submit="submitReport"
    />

    <ActivityRecentPresetsCard
      :recent-presets="recentPresets"
      @select-preset="applyPreset"
    />
  </div>
</template>
