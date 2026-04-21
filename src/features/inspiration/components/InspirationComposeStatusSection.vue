<script setup lang="ts">
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import ToggleSwitch from "primevue/toggleswitch";
import { useI18n } from "vue-i18n";

import type { ActivitySelectOption } from "@/features/inspiration/types";

const attachCurrentStatus = defineModel<boolean>("attachCurrentStatus", { required: true });
const attachStatusIncludeDeviceInfo = defineModel<boolean>("attachStatusIncludeDeviceInfo", {
  required: true,
});
const statusSnapshotInput = defineModel<string>("statusSnapshotInput", { required: true });
const statusSnapshotDeviceName = defineModel<string>("statusSnapshotDeviceName", { required: true });
const selectedActivityKey = defineModel<string>("selectedActivityKey", { required: true });

defineProps<{
  mobileRuntime: boolean;
  activityOptions: ActivitySelectOption[];
  activityLoading: boolean;
  selectedSnapshotPreview: string;
}>();

const emit = defineEmits<{
  refreshActivities: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="field-block field-span-2">
    <span class="field-label">
      {{ mobileRuntime ? t("inspiration.fields.statusMobile") : t("inspiration.fields.statusDesktop") }}
    </span>
    <div class="activity-toggle-row">
      <ToggleSwitch v-model="attachCurrentStatus" input-id="attach-current-status" />
      <label for="attach-current-status">
        {{
          mobileRuntime
            ? t("inspiration.toggles.attachStatusMobile")
            : t("inspiration.toggles.attachStatusDesktop")
        }}
      </label>
      <ToggleSwitch
        v-model="attachStatusIncludeDeviceInfo"
        input-id="attach-device-info"
        :disabled="!attachCurrentStatus"
      />
      <label for="attach-device-info">{{ t("inspiration.toggles.attachDeviceInfo") }}</label>
    </div>
    <div v-if="mobileRuntime" class="activity-select-row">
      <InputText
        v-model="statusSnapshotInput"
        :disabled="!attachCurrentStatus"
        :placeholder="t('inspiration.placeholders.statusInput')"
      />
    </div>
    <div v-if="mobileRuntime" class="activity-select-row">
      <InputText
        v-model="statusSnapshotDeviceName"
        :disabled="!attachCurrentStatus"
        :placeholder="t('inspiration.placeholders.deviceName')"
      />
    </div>
    <div v-else class="activity-select-row">
      <Select
        v-model="selectedActivityKey"
        :options="activityOptions"
        option-label="label"
        option-value="value"
        show-clear
        filter
        :loading="activityLoading"
        :disabled="!attachCurrentStatus"
        :placeholder="t('inspiration.placeholders.activitySelect')"
      />
      <Button
        icon="pi pi-refresh"
        severity="secondary"
        text
        :loading="activityLoading"
        :aria-label="t('inspiration.buttons.refreshActivities')"
        :title="t('inspiration.buttons.refreshActivities')"
        @click="emit('refreshActivities')"
      />
    </div>
    <small class="field-help">
      {{
        mobileRuntime
          ? t("inspiration.help.statusMobile")
          : t("inspiration.help.statusDesktop")
      }}
    </small>
    <div v-if="attachCurrentStatus && selectedSnapshotPreview" class="snapshot-preview">
      <strong>{{ t("inspiration.help.snapshotPreview") }}</strong>
      <span>{{ selectedSnapshotPreview }}</span>
    </div>
  </div>
</template>
