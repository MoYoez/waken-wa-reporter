<script setup lang="ts">
import { computed, reactive } from "vue";
import Button from "primevue/button";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Password from "primevue/password";
import Select from "primevue/select";
import Tag from "primevue/tag";
import Textarea from "primevue/textarea";
import { useToast } from "primevue/usetoast";

import { parseImportedIntegrationConfig, validateConfig } from "../lib/api";
import { createNotifier } from "../lib/notify";
import type { ClientCapabilities, ClientConfig, DeviceType } from "../types";

const props = defineProps<{
  modelValue: ClientConfig;
  capabilities: ClientCapabilities;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: ClientConfig];
  imported: [message: string];
}>();

const toast = useToast();
const importPayload = reactive({ text: "" });
const isNativeNotice = computed(() => !props.capabilities.realtimeReporter);
const { notify } = createNotifier(toast, () => isNativeNotice.value);

const issues = computed(() => validateConfig(props.modelValue, props.capabilities));
const reporterSupported = computed(() => props.capabilities.realtimeReporter);

function updateField<K extends keyof ClientConfig>(key: K, value: ClientConfig[K]) {
  emit("update:modelValue", {
    ...props.modelValue,
    [key]: value,
  });
}

function inferMobileDeviceType(): DeviceType {
  if (typeof window === "undefined") return "mobile";
  return window.matchMedia("(max-width: 899px)").matches ? "mobile" : "tablet";
}

function toBaseUrl(reportEndpoint?: string) {
  if (!reportEndpoint) return undefined;
  return reportEndpoint.replace(/\/api\/activity\/?$/i, "").replace(/\/$/, "");
}

function importConfig() {
  parseImportedIntegrationConfig(importPayload.text)
    .then((parsed) => {
      emit("update:modelValue", {
        ...props.modelValue,
        baseUrl: toBaseUrl(parsed.reportEndpoint) ?? props.modelValue.baseUrl,
        apiToken: parsed.token ?? props.modelValue.apiToken,
        deviceType: reporterSupported.value ? "desktop" : inferMobileDeviceType(),
      });
      emit("imported", parsed.tokenName ? `已导入 Token：${parsed.tokenName}` : "已导入接入配置。");
      notify({
        severity: "success",
        summary: "配置导入成功",
        detail: toBaseUrl(parsed.reportEndpoint) ?? "已自动填入地址与 Token。",
        life: 3000,
      });
      importPayload.text = "";
    })
    .catch((error) => {
      notify({
        severity: "error",
        summary: "导入失败",
        detail: error instanceof Error ? error.message : "配置内容无效。",
        life: 4000,
      });
    });
}
</script>

<template>
  <Card class="glass-card connection-panel">
    <template #title>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">连接设置</p>
          <h3>完成连接后，这台设备就可以稳定同步到你的 Waken-Wa</h3>
        </div>
        <Tag :severity="issues.length ? 'warn' : 'success'" :value="issues.length ? '待完善' : '已就绪'" rounded />
      </div>
    </template>

    <template #content>
      <div class="settings-section">
        <div class="settings-section-head">
          <strong>连接信息</strong>
          <span>用于建立与站点的连接，也是内容发布和状态同步共用的核心凭证。设备标识会由客户端自动生成并长期保持稳定。</span>
        </div>
        <div class="panel-grid">
          <label class="field-block field-span-2">
            <span class="field-label">站点地址</span>
            <InputText
              :model-value="modelValue.baseUrl"
              placeholder="https://waken-wa.example.com"
              @update:model-value="updateField('baseUrl', $event ?? '')"
            />
          </label>

          <label class="field-block field-span-2">
            <span class="field-label">API Token</span>
            <Password
              :model-value="modelValue.apiToken"
              placeholder="粘贴后台生成的完整 Token，无需手动添加 Bearer"
              fluid
              toggle-mask
              :feedback="false"
              @update:model-value="updateField('apiToken', $event)"
            />
          </label>

          <label class="field-block">
            <span class="field-label">设备名称（可选）</span>
            <InputText
              :model-value="modelValue.device"
              placeholder="留空则使用默认设备名"
              @update:model-value="updateField('device', $event ?? '')"
            />
          </label>
        </div>
      </div>

      <div v-if="reporterSupported" class="settings-section">
        <div class="settings-section-head">
          <strong>后台同步</strong>
          <span>控制后台同步的轮询节奏、心跳频率，以及默认附带的扩展信息。</span>
        </div>
        <div class="panel-grid">
          <label class="field-block">
            <span class="field-label">轮询间隔</span>
            <InputText
              :model-value="String(modelValue.pollIntervalMs)"
              placeholder="2000"
              @update:model-value="updateField('pollIntervalMs', Number($event ?? 0))"
            />
          </label>

          <label class="field-block">
            <span class="field-label">心跳间隔</span>
            <InputText
              :model-value="String(modelValue.heartbeatIntervalMs)"
              placeholder="60000"
              @update:model-value="updateField('heartbeatIntervalMs', Number($event ?? 0))"
            />
          </label>

          <label class="field-block field-span-2">
            <span class="field-label">启动后自动开启后台同步</span>
            <Select
              :model-value="modelValue.reporterEnabled"
              :options="[
                { label: '关闭', value: false },
                { label: '开启', value: true },
              ]"
              option-label="label"
              option-value="value"
              @update:model-value="updateField('reporterEnabled', Boolean($event))"
            />
          </label>

          <label class="field-block field-span-2">
            <span class="field-label">后台同步附加信息 JSON</span>
            <Textarea
              :model-value="modelValue.reporterMetadataJson"
              rows="4"
              auto-resize
              placeholder="{&quot;source&quot;:&quot;waken-wa-client&quot;}"
              @update:model-value="updateField('reporterMetadataJson', $event ?? '')"
            />
          </label>
        </div>
      </div>
      <div v-else class="settings-section">
        <div class="settings-section-head">
          <strong>移动端说明</strong>
          <span>当前运行在移动端模式：后台实时同步相关参数已停用，仅保留手动活动提交与内容发布。</span>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-section-head">
          <strong>快速导入</strong>
          <span>如果你已经从后台复制过接入配置，可以直接粘贴到这里，一次性填写地址和 Token。</span>
        </div>
        <div class="panel-grid">
          <label class="field-block field-span-2">
            <span class="field-label">粘贴接入配置</span>
            <Textarea
              v-model="importPayload.text"
              rows="4"
              auto-resize
              placeholder="粘贴从 Waken-Wa 后台复制的一键接入配置。"
            />
          </label>
        </div>
      </div>

      <div class="actions-row">
        <Button label="导入接入配置" icon="pi pi-upload" @click="importConfig" />
      </div>

      <div class="message-stack">
        <Message v-if="issues.length === 0" severity="success" :closable="false">
          当前设置已可用，Wink~
        </Message>
        <Message v-for="issue in issues" :key="issue" severity="warn" :closable="false">
          {{ issue }}
        </Message>
      </div>
    </template>
  </Card>
</template>
