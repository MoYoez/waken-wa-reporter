<script setup lang="ts">
import { computed, ref, watch } from "vue";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import ToggleSwitch from "primevue/toggleswitch";
import { useI18n } from "vue-i18n";

import {
  formatReporterTimingIssueBrief,
  formatReporterTimingIssue,
  HEARTBEAT_SOFT_CEILING_MS,
  validateReporterTimingInput,
  type ReporterTimingIssue,
} from "@/lib/reporterTimingValidation";
import {
  detectPerformanceMode,
  PERFORMANCE_PRESETS,
  type ClientConfig,
  type PerformanceMode,
} from "@/types";

interface ReporterContentOption {
  key:
    | "reportForegroundApp"
    | "reportWindowTitle"
    | "reportMedia"
    | "reportPlaySource"
    | "reportMediaArtwork"
    | "reportPlaybackAppIcon"
    | "reportMediaGenre";
  label: string;
  description: string;
  inputId: string;
}

const props = defineProps<{
  modelValue: ClientConfig;
  variant: "default" | "onboarding";
  titleKey: string;
  helpKey: string;
  reporterContentOptions: ReporterContentOption[];
  showHeading?: boolean;
  headClass?: string;
}>();

const emit = defineEmits<{
  updateField: [key: keyof ClientConfig, value: ClientConfig[keyof ClientConfig]];
  updateFields: [patch: Partial<ClientConfig>];
}>();

const { t } = useI18n();

const pollIntervalInput = ref(String(props.modelValue.pollIntervalMs));
const heartbeatIntervalInput = ref(String(props.modelValue.heartbeatIntervalMs));

const pollIntervalIssue = computed(() =>
  validateReporterTimingInput(
    "pollIntervalMs",
    pollIntervalInput.value,
    props.modelValue.pollIntervalMs,
  ).issue,
);
const heartbeatIntervalIssue = computed(() =>
  validateReporterTimingInput(
    "heartbeatIntervalMs",
    heartbeatIntervalInput.value,
    props.modelValue.heartbeatIntervalMs,
  ).issue,
);

watch(
  () => props.modelValue.pollIntervalMs,
  (value) => {
    if (!pollIntervalIssue.value) {
      pollIntervalInput.value = String(value);
    }
  },
);

watch(
  () => props.modelValue.heartbeatIntervalMs,
  (value) => {
    if (!heartbeatIntervalIssue.value) {
      heartbeatIntervalInput.value = String(value);
    }
  },
);

function updateTimingField(
  key: "pollIntervalMs" | "heartbeatIntervalMs",
  input: unknown,
) {
  const raw = String(input ?? "");
  const fallback = props.modelValue[key];

  if (key === "pollIntervalMs") {
    pollIntervalInput.value = raw;
  } else {
    heartbeatIntervalInput.value = raw;
  }

  const result = validateReporterTimingInput(key, raw, fallback);
  if (result.value !== undefined) {
    emit("updateField", key, result.value);
  }
}

function timingIssueMessage(issue?: ReporterTimingIssue) {
  return issue ? formatReporterTimingIssue(issue, t) : "";
}

function timingIssueBrief(issue?: ReporterTimingIssue) {
  return issue ? formatReporterTimingIssueBrief(issue, t) : "";
}

const performanceMode = computed<PerformanceMode>(() => props.modelValue.performanceMode);

const performanceOptions = computed(() =>
  (["realtime", "balanced", "powersave", "custom"] as PerformanceMode[]).map((value) => ({
    value,
    label: t(`connectionPanel.performance.options.${value}.label`),
    description: t(`connectionPanel.performance.options.${value}.description`),
  })),
);

const showAdvancedTimings = ref(performanceMode.value === "custom");
watch(performanceMode, (mode) => {
  if (mode === "custom") {
    showAdvancedTimings.value = true;
  }
});

function applyPerformanceMode(next: PerformanceMode) {
  if (next === "custom") {
    emit("updateField", "performanceMode", next);
    showAdvancedTimings.value = true;
    return;
  }
  const preset = PERFORMANCE_PRESETS[next];
  emit("updateFields", {
    pollIntervalMs: preset.pollIntervalMs,
    heartbeatIntervalMs: preset.heartbeatIntervalMs,
    performanceMode: next,
  });
  pollIntervalInput.value = String(preset.pollIntervalMs);
  heartbeatIntervalInput.value = String(preset.heartbeatIntervalMs);
}

watch(
  () => [props.modelValue.pollIntervalMs, props.modelValue.heartbeatIntervalMs] as const,
  ([poll, heartbeat]) => {
    const detected = detectPerformanceMode(poll, heartbeat);
    if (detected !== props.modelValue.performanceMode) {
      emit("updateField", "performanceMode", detected);
    }
  },
);

const heartbeatCeilingHint = computed(() => {
  if (props.modelValue.heartbeatIntervalMs > HEARTBEAT_SOFT_CEILING_MS) {
    return t("connectionPanel.performance.heartbeatCeilingHint", {
      ceiling: HEARTBEAT_SOFT_CEILING_MS / 1000,
    });
  }
  return "";
});

</script>

<template>
  <div v-if="props.showHeading !== false" :class="props.headClass || 'settings-section-head'">
    <strong>{{ t(props.titleKey) }}</strong>
    <span>{{ t(props.helpKey) }}</span>
  </div>
  <div class="panel-grid">
    <label class="field-block field-span-2 performance-mode-field">
      <span class="field-label">{{ t("connectionPanel.performance.label") }}</span>
      <Select
        :model-value="performanceMode"
        :options="performanceOptions"
        option-label="label"
        option-value="value"
        @update:model-value="applyPerformanceMode($event as PerformanceMode)"
      />
      <small class="field-help">
        {{ performanceOptions.find((o) => o.value === performanceMode)?.description }}
      </small>
    </label>

    <div class="advanced-timings-card field-span-2">
      <button
        type="button"
        class="advanced-toggle-button"
        @click="showAdvancedTimings = !showAdvancedTimings"
      >
        <i :class="showAdvancedTimings ? 'pi pi-chevron-up' : 'pi pi-chevron-down'" />
        <span>
          {{
            showAdvancedTimings
              ? t("connectionPanel.performance.hideAdvanced")
              : t("connectionPanel.performance.showAdvanced")
          }}
        </span>
      </button>

      <div class="advanced-timings-content" :class="{ 'advanced-timings-content--expanded': showAdvancedTimings }">
        <label class="field-block">
          <span class="field-label">{{ t("connectionPanel.fields.pollInterval") }}</span>
          <InputText
            :model-value="pollIntervalInput"
            :invalid="Boolean(pollIntervalIssue)"
            :placeholder="t('connectionPanel.placeholders.pollInterval')"
            @update:model-value="updateTimingField('pollIntervalMs', $event)"
          />
          <small
            class="field-help field-help-error timing-field-error"
            :class="{ 'timing-field-error-empty': !pollIntervalIssue }"
            :title="timingIssueMessage(pollIntervalIssue)"
            aria-live="polite"
          >
            <i v-if="pollIntervalIssue" class="pi pi-exclamation-circle" />
            {{ timingIssueBrief(pollIntervalIssue) || "\u00A0" }}
          </small>
        </label>

        <label class="field-block">
          <span class="field-label">{{ t("connectionPanel.fields.heartbeatInterval") }}</span>
          <InputText
            :model-value="heartbeatIntervalInput"
            :invalid="Boolean(heartbeatIntervalIssue)"
            :placeholder="t('connectionPanel.placeholders.heartbeatInterval')"
            @update:model-value="updateTimingField('heartbeatIntervalMs', $event)"
          />
          <small
            class="field-help field-help-error timing-field-error"
            :class="{ 'timing-field-error-empty': !heartbeatIntervalIssue }"
            :title="timingIssueMessage(heartbeatIntervalIssue)"
            aria-live="polite"
          >
            <i v-if="heartbeatIntervalIssue" class="pi pi-exclamation-circle" />
            {{ timingIssueBrief(heartbeatIntervalIssue) || "\u00A0" }}
          </small>
          <small v-if="heartbeatCeilingHint" class="field-help field-help-warn">
            <i class="pi pi-info-circle" />
            {{ heartbeatCeilingHint }}
          </small>
        </label>
      </div>
    </div>

    <div class="reporter-enabled-card field-span-2">
      <div class="reporter-enabled-copy">
        <span class="field-label">{{ t("connectionPanel.fields.reporterEnabled") }}</span>
        <strong>{{ props.modelValue.reporterEnabled ? t("connectionPanel.toggles.enabled") : t("connectionPanel.toggles.reporterDisabled") }}</strong>
        <span>
          {{
            props.variant === "onboarding"
              ? t("connectionPanel.help.reporterEnabledOnboarding")
              : t("connectionPanel.help.reporterEnabledSettings")
          }}
        </span>
      </div>
      <ToggleSwitch
        :model-value="props.modelValue.reporterEnabled"
        :input-id="props.variant === 'onboarding' ? 'onboarding-reporter-enabled' : 'settings-reporter-enabled'"
        @update:model-value="emit('updateField', 'reporterEnabled', Boolean($event))"
      />
    </div>

    <div :class="`${props.headClass || 'settings-section-head'} field-span-2`">
      <strong>{{ t("connectionPanel.sections.reportContent") }}</strong>
      <span>{{ t("connectionPanel.help.reportContent") }}</span>
    </div>

    <div class="compact-toggle-grid field-span-2">
      <div
        v-for="option in props.reporterContentOptions"
        :key="props.variant === 'onboarding' ? `onboarding-${option.key}` : option.key"
        class="compact-toggle-card"
      >
        <div class="compact-toggle-copy">
          <strong>{{ option.label }}</strong>
          <span>{{ option.description }}</span>
        </div>
        <ToggleSwitch
          :model-value="props.modelValue[option.key]"
          :input-id="props.variant === 'onboarding' ? `onboarding-${option.inputId}` : option.inputId"
          @update:model-value="emit('updateField', option.key, Boolean($event))"
        />
      </div>
    </div>

      </div>
</template>
