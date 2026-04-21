import { computed, reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "primevue/usetoast";

import {
  extractPendingApprovalInfo,
  formatPendingApprovalDetail,
  submitActivityReport,
  validateConfig,
} from "@/lib/api";
import { readBatterySnapshot } from "@/lib/deviceInfo";
import { resolveApiErrorMessage } from "@/lib/localizedText";
import { createNotifier } from "@/lib/notify";
import type {
  ActivityPayload,
  ClientCapabilities,
  ClientConfig,
  PendingApprovalInfo,
  RecentPreset,
} from "@/types";

export interface ActivityFormState {
  processName: string;
  processTitle: string;
  includeBattery: boolean;
  persistMinutes: number;
}

interface ActivityWorkspaceProps {
  config: ClientConfig;
  capabilities: ClientCapabilities;
}

interface ActivityWorkspaceEmitters {
  presetSaved: (preset: RecentPreset) => void;
  pendingApproval: (info: PendingApprovalInfo) => void;
  keyVerified: (generatedHashKey: string) => void;
}

export function useActivityWorkspace(
  props: ActivityWorkspaceProps,
  emit: ActivityWorkspaceEmitters,
) {
  const { t } = useI18n();
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
  const hasConfigIssues = computed(() => configIssues.value.length > 0);

  function applyPreset(preset: RecentPreset) {
    form.processName = preset.process_name;
    form.processTitle = preset.process_title ?? "";
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
            detail: error instanceof Error
              ? error.message
              : t("activity.notify.batteryUnavailableDetail"),
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
    if (hasConfigIssues.value) {
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
      emit.pendingApproval(pendingApproval);
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

    emit.presetSaved({
      process_name: form.processName.trim(),
      process_title: form.processTitle.trim() || undefined,
      lastUsedAt: new Date().toISOString(),
    });
    emit.keyVerified(props.config.generatedHashKey.trim());

    notify({
      severity: "success",
      summary: t("activity.notify.submitSuccess"),
      detail: t("activity.notify.submitSuccessDetail"),
      life: 3000,
    });
  }

  return {
    applyPreset,
    form,
    hasConfigIssues,
    mobileRuntime,
    submitReport,
    submitting,
  };
}
