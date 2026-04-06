<script setup lang="ts">
import { computed, reactive, ref } from "vue";
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
import { readBatterySnapshot } from "../lib/battery";
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
  batteryLevel: number | null;
  isCharging: boolean;
  persistMinutes: number;
}

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
  batteryLevel: null,
  isCharging: false,
  persistMinutes: 30,
});

const submitting = ref(false);

const configIssues = computed(() => validateConfig(props.config, props.capabilities));

async function detectBattery() {
  try {
    const battery = await readBatterySnapshot();
    form.batteryLevel = battery.levelPercent;
    form.isCharging = battery.charging;
    notify({
      severity: "success",
      summary: "电量已读取",
      detail: `当前电量 ${battery.levelPercent}%${battery.charging ? "（充电中）" : ""}`,
      life: 2500,
    });
  } catch (error) {
    notify({
      severity: "warn",
      summary: "无法获取电池信息",
      detail: error instanceof Error ? error.message : "当前运行环境不支持读取电量。",
      life: 3500,
    });
  }
}

function applyPreset(preset: RecentPreset) {
  form.processName = preset.process_name;
  form.processTitle = preset.process_title ?? "";
}

async function buildRequestPayload(): Promise<ActivityPayload> {
  let batteryLevel = form.batteryLevel;
  let isCharging = form.isCharging;

  if (mobileRuntime.value) {
    try {
      const battery = await readBatterySnapshot();
      batteryLevel = battery.levelPercent;
      isCharging = battery.charging;
    } catch {
      batteryLevel = null;
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
    metadata: {
      source: "waken-wa-client",
    },
  };
}

async function submitReport() {
  if (configIssues.value.length > 0) {
    notify({
      severity: "warn",
      summary: "请先完成连接设置",
      detail: "请先填写站点地址和 API Token。",
      life: 3500,
    });
    return;
  }

  if (!form.processName.trim()) {
    notify({
      severity: "warn",
      summary: "请填写进程名称",
      detail: "添加活动前，需要提供进程名称。",
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
      summary: "设备待审核",
      detail: formatPendingApprovalDetail(pendingApproval),
      life: 6000,
    });
    emit("pendingApproval", pendingApproval);
    return;
  }

  if (!result.success) {
    notify({
      severity: "error",
      summary: `添加失败（${result.status || "网络"}）`,
      detail: result.error?.message ?? "请求未成功完成。",
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
    summary: "活动已添加",
    detail: "Waken-Wa 已成功接收这条活动记录。",
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
            <p class="eyebrow">活动同步</p>
            <h3>快速添加活动</h3>
          </div>
        </div>
      </template>
      <template #content>
        <div class="activity-form-stack">
          <div class="panel-grid">
            <label class="field-block">
              <span class="field-label">进程名称</span>
              <InputText v-model="form.processName" placeholder="例如：VS Code" />
            </label>
          </div>

          <label class="field-block field-span-2">
            <span class="field-label">进程标题（可选）</span>
            <InputText v-model="form.processTitle" placeholder="例如：编辑 index.tsx" />
          </label>

          <div v-if="!mobileRuntime" class="panel-grid">
            <label class="field-block">
              <span class="field-label">电量（可选，0-100）</span>
              <InputNumber
                v-model="form.batteryLevel"
                :min="0"
                :max="100"
                placeholder="留空则不上报"
                fluid
              />
            </label>
            <div class="field-block charging-field">
              <span class="field-label">&nbsp;</span>
              <div class="charging-row">
                <ToggleSwitch v-model="form.isCharging" input-id="manual-is-charging" />
                <label for="manual-is-charging">充电中</label>
                <Button
                  label="读取电量信息"
                  icon="pi pi-bolt"
                  severity="secondary"
                  text
                  size="small"
                  @click="detectBattery"
                />
              </div>
            </div>
          </div>

          <label class="field-block field-span-2">
            <span class="field-label">常驻时长（分钟）</span>
            <InputNumber v-model="form.persistMinutes" :min="1" :max="1440" fluid />
            <small class="field-help">
              无客户端上报时，超过该时间后活动会从首页「当前状态」自动结束（1-1440 分钟）。
            </small>
          </label>

          <div class="actions-row">
            <Button
              label="添加活动"
              icon="pi pi-plus"
              :loading="submitting"
              @click="submitReport"
            />
          </div>
        </div>

        <div class="message-stack">
          <Message v-if="configIssues.length" severity="warn" :closable="false">
            请先到“设置”里补齐站点地址和 API Token。
          </Message>
        </div>
      </template>
    </Card>

    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">最近使用</p>
            <h3>快速填入最近同步过的常用活动信息</h3>
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
            <span>{{ preset.process_title || "未填写窗口标题" }}</span>
            <small>{{ new Date(preset.lastUsedAt).toLocaleString() }}</small>
          </button>
        </div>
        <Message v-else severity="secondary" :closable="false">
          首次同步成功后，最近使用的活动信息会出现在这里。
        </Message>
      </template>
    </Card>
  </div>
</template>
