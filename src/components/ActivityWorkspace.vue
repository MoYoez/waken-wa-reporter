<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import ToggleSwitch from "primevue/toggleswitch";
import { useToast } from "primevue/usetoast";

import {
  extractPendingApprovalInfo,
  formatPendingApprovalDetail,
  submitActivityReport,
  validateConfig,
} from "../lib/api";
import { readBatterySnapshot } from "../lib/deviceInfo";
import { resolveApiErrorMessage } from "../lib/localizedText";
import { createNotifier } from "../lib/notify";
import type {
  ActivityPayload,
  ClientCapabilities,
  ClientConfig,
  PendingApprovalInfo,
  RecentPreset,
} from "../types";

interface ActivityFormState {
  processName: string;
  processTitle: string;
  includeBattery: boolean;
  persistMinutes: number;
}

const { t, locale } = useI18n();

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

const toast = useToast();
const isNativeNotice = computed(() => !props.capabilities.realtimeReporter);
const { notify } = createNotifier(toast, () => isNativeNotice.value);
const mobileRuntime = computed(() => !props.capabilities.realtimeReporter);

const form = reactive<ActivityFormState>({
  processName: "",
  processTitle: "",
  includeBattery: true,
  persistMinutes: 30,
});

const submitting = ref(false);

const configIssues = computed(() => validateConfig(props.config, props.capabilities));

function applyPreset(preset: RecentPreset) {
  form.processName = preset.process_name;
  form.processTitle = preset.process_title ?? "";
}

function formatTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString(locale.value);
}

function translateText(key: string, params?: Record<string, unknown>) {
  return params ? t(key, params) : t(key);
}

function apiErrorDetail(
  error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
  fallback: string,
) {
  return resolveApiErrorMessage(error, translateText, fallback);
}

async function buildRequestPayload(): Promise<ActivityPayload> {
  let batteryLevel: number | null = null;
  let isCharging = false;

  if (mobileRuntime.value || form.includeBattery) {
    try {
      const battery = await readBatterySnapshot();
      batteryLevel = battery.levelPercent;
      isCharging = battery.charging;
    } catch (error) {
      batteryLevel = null;
      if (form.includeBattery) {
        notify({
          severity: "warn",
          summary: t("activity.notify.batteryUnavailable"),
          detail: error instanceof Error ? error.message : t("activity.notify.batteryUnavailableDetail"),
          life: 3500,
        });
      }
    }
  }

  return {
    generatedHashKey: props.config.generatedHashKey.trim(),
    process_name: form.processName.trim(),
    ...(form.processTitle.trim() ? { process_title: form.processTitle.trim() } : {}),
    device_type: props.config.deviceType,
    push_mode: "active",
    persist_minutes: Math.min(Math.max(Math.round(form.persistMinutes || 30), 1), 1440),
    ...(typeof batteryLevel === "number" ? { battery_level: batteryLevel } : {}),
    ...(typeof batteryLevel === "number" ? { is_charging: isCharging } : {}),
  };
}

async function submitReport() {
  if (configIssues.value.length > 0) {
    notify({
      severity: "warn",
      summary: t("activity.notify.settingsRequired"),
      detail: t("activity.notify.settingsRequiredDetail"),
      life: 3500,
    });
    return;
  }

  if (!form.processName.trim()) {
    notify({
      severity: "warn",
      summary: t("activity.notify.nameRequired"),
      detail: t("activity.notify.nameRequiredDetail"),
      life: 3000,
    });
    return;
  }

  submitting.value = true;
  const result = await submitActivityReport(props.config, await buildRequestPayload());
  submitting.value = false;

  const pendingApproval = extractPendingApprovalInfo(result);
  if (pendingApproval) {
    notify({
      severity: "warn",
      summary: t("activity.notify.pendingApproval"),
      detail: formatPendingApprovalDetail(pendingApproval),
      life: 6000,
    });
    emit("pendingApproval", pendingApproval);
    return;
  }

  if (!result.success) {
    notify({
      severity: "error",
      summary: t("activity.notify.submitFailed", {
        status: result.status || t("activity.common.network"),
      }),
      detail: apiErrorDetail(result.error, t("activity.notify.submitFailedDetail")),
      life: 4500,
    });
    return;
  }

  emit("presetSaved", {
    process_name: form.processName.trim(),
    process_title: form.processTitle.trim() || undefined,
    lastUsedAt: new Date().toISOString(),
  });
  emit("keyVerified", props.config.generatedHashKey.trim());

  notify({
    severity: "success",
    summary: t("activity.notify.submitSuccess"),
    detail: t("activity.notify.submitSuccessDetail"),
    life: 3000,
  });
}
</script>

<template>
  <div class="workspace-grid">
    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("activity.title.eyebrow") }}</p>
            <h3>{{ t("activity.title.title") }}</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="activity-form-stack">
          <div class="panel-grid">
            <label class="field-block">
              <span class="field-label">{{ t("activity.fields.name") }}</span>
              <InputText v-model="form.processName" :placeholder="t('activity.placeholders.name')" />
            </label>
          </div>

          <label class="field-block field-span-2">
            <span class="field-label">{{ t("activity.fields.title") }}</span>
            <InputText v-model="form.processTitle" :placeholder="t('activity.placeholders.title')" />
          </label>

          <div v-if="!mobileRuntime" class="panel-grid">
            <div class="reporter-enabled-card field-span-2">
              <div class="reporter-enabled-copy">
                <span class="field-label">{{ t("activity.fields.includeBattery") }}</span>
                <strong>{{ form.includeBattery ? t("activity.common.enabled") : t("activity.common.disabled") }}</strong>
                <span>{{ t("activity.help.includeBattery") }}</span>
              </div>
              <ToggleSwitch
                v-model="form.includeBattery"
                input-id="manual-include-battery"
              />
            </div>
          </div>

          <label class="field-block field-span-2">
            <span class="field-label">{{ t("activity.fields.persistMinutes") }}</span>
            <InputNumber v-model="form.persistMinutes" :min="1" :max="1440" fluid />
            <small class="field-help">
              {{ t("activity.help.persistMinutes") }}
            </small>
          </label>

          <div class="actions-row">
            <Button
              :label="t('activity.buttons.submit')"
              icon="pi pi-plus"
              :loading="submitting"
              @click="submitReport"
            />
          </div>
        </div>

        <div class="message-stack">
          <Message v-if="configIssues.length" severity="warn" :closable="false">
            {{ t("activity.help.settingsWarning") }}
          </Message>
        </div>
      </template>
    </Card>

    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("activity.recent.eyebrow") }}</p>
            <h3>{{ t("activity.recent.title") }}</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div v-if="recentPresets.length" class="preset-grid">
          <button
            v-for="preset in recentPresets"
            :key="`${preset.process_name}-${preset.lastUsedAt}`"
            class="preset-card"
            type="button"
            @click="applyPreset(preset)"
          >
            <strong>{{ preset.process_name }}</strong>
            <span>{{ preset.process_title || t("activity.common.windowTitleFallback") }}</span>
            <small>{{ formatTime(preset.lastUsedAt) }}</small>
          </button>
        </div>
        <Message v-else severity="secondary" :closable="false">
          {{ t("activity.recent.empty") }}
        </Message>
      </template>
    </Card>
  </div>
</template>
