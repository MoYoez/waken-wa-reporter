<script setup lang="ts">
import { computed, reactive } from "vue";
import Button from "primevue/button";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Password from "primevue/password";
import Tag from "primevue/tag";
import Textarea from "primevue/textarea";
import ToggleSwitch from "primevue/toggleswitch";
import { useToast } from "primevue/usetoast";

import { parseImportedIntegrationConfig, validateConfig } from "../lib/api";
import { createNotifier } from "../lib/notify";
import type { ClientCapabilities, ClientConfig, DeviceType } from "../types";

const props = defineProps<{
  modelValue: ClientConfig;
  capabilities: ClientCapabilities;
  variant?: "default" | "onboarding";
  verifiedGeneratedHashKey?: string;
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
const isOnboarding = computed(() => props.variant === "onboarding");
const currentGeneratedHashKey = computed(() => props.modelValue.generatedHashKey.trim());
const reporterContentOptions = [
  {
    key: "reportForegroundApp" as const,
    label: "当前应用",
    description: "当前正在使用的应用",
    inputId: "report-foreground-app",
  },
  {
    key: "reportWindowTitle" as const,
    label: "窗口名称",
    description: "当前窗口标题或名称",
    inputId: "report-window-title",
  },
  {
    key: "reportMedia" as const,
    label: "播放内容",
    description: "正在播放的媒体内容",
    inputId: "report-media",
  },
  {
    key: "reportPlaySource" as const,
    label: "播放来源",
    description: "媒体来自哪个应用",
    inputId: "report-play-source",
  },
];

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
        device: parsed.deviceName?.trim() || props.modelValue.device,
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
      <template v-if="isOnboarding">
        <div class="settings-section">
          <div class="settings-section-head">
            <strong>Base64 快速导入</strong>
            <span>首次引导里优先推荐这种方式。把后台复制的一键接入配置贴进来，会自动填好站点地址和 Token。</span>
          </div>
          <div class="panel-grid">
            <label class="field-block field-span-2">
              <span class="field-label">Base64 接入配置</span>
              <Textarea
                v-model="importPayload.text"
                rows="4"
                auto-resize
                placeholder="粘贴从 Waken-Wa 后台复制的一键接入配置。"
              />
            </label>
          </div>
          <div class="actions-row">
            <Button label="导入接入配置" icon="pi pi-upload" @click="importConfig" />
          </div>
        </div>

        <div class="settings-section">
          <details class="settings-disclosure">
            <summary class="settings-disclosure-summary">
              <div>
                <strong>附加配置</strong>
                <span>需要手动填写或微调时，再展开设置站点地址、Token 和同步参数。</span>
              </div>
              <i class="pi pi-angle-down" aria-hidden="true" />
            </summary>
            <div class="settings-disclosure-body">
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

                <label class="field-block field-span-2">
                  <span class="field-label">设备 Key（自动生成）</span>
                  <InputText
                    :model-value="currentGeneratedHashKey"
                    readonly
                    placeholder="首次启动后自动生成"
                  />
                  <small class="field-help">这个 Key 用于标识当前设备。首次启用或更换后，服务端可能需要重新审核。</small>
                </label>

                <div v-if="reporterSupported" class="reporter-enabled-card field-span-2">
                  <div class="reporter-enabled-copy">
                    <span class="field-label">使用系统代理</span>
                    <strong>{{ modelValue.useSystemProxy ? "已开启" : "已关闭" }}</strong>
                    <span>
                      开启后会按系统与运行环境中的代理配置发起请求，常见包括 `HTTP_PROXY`、`HTTPS_PROXY` 和 `ALL_PROXY`。
                    </span>
                  </div>
                  <ToggleSwitch
                    :model-value="modelValue.useSystemProxy"
                    input-id="onboarding-use-system-proxy"
                    @update:model-value="updateField('useSystemProxy', Boolean($event))"
                  />
                </div>
              </div>

              <template v-if="reporterSupported">
                <div class="settings-section-head settings-disclosure-subhead">
                  <strong>后台同步附加配置</strong>
                  <span>控制后台同步的轮询节奏、心跳频率、自动启动，以及自动同步包含哪些内容。</span>
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

                  <div class="reporter-enabled-card field-span-2">
                    <div class="reporter-enabled-copy">
                      <span class="field-label">启动后自动开启后台同步</span>
                      <strong>{{ modelValue.reporterEnabled ? "已开启" : "未开启" }}</strong>
                      <span>
                        开启后，这台客户端下次启动时会在连接配置就绪后自动开始后台同步。
                      </span>
                    </div>
                    <ToggleSwitch
                      :model-value="modelValue.reporterEnabled"
                      input-id="onboarding-reporter-enabled"
                      @update:model-value="updateField('reporterEnabled', Boolean($event))"
                    />
                  </div>

                  <div class="settings-section-head settings-disclosure-subhead field-span-2">
                    <strong>自动同步包含内容</strong>
                    <span>选择自动同步时要包含的内容。</span>
                  </div>

                  <div class="compact-toggle-grid field-span-2">
                    <div
                      v-for="option in reporterContentOptions"
                      :key="`onboarding-${option.key}`"
                      class="compact-toggle-card"
                    >
                      <div class="compact-toggle-copy">
                        <strong>{{ option.label }}</strong>
                        <span>{{ option.description }}</span>
                      </div>
                      <ToggleSwitch
                        :model-value="modelValue[option.key]"
                        :input-id="`onboarding-${option.inputId}`"
                        @update:model-value="updateField(option.key, Boolean($event))"
                      />
                    </div>
                  </div>
                </div>
              </template>
              <div v-else class="settings-section-head settings-disclosure-subhead">
                <strong>移动端说明</strong>
                <span>当前运行在移动端模式：后台实时同步相关参数已停用，仅保留手动活动提交与内容发布。</span>
              </div>
            </div>
          </details>
        </div>

        <div class="settings-section">
          <div class="settings-section-head">
            <strong>设备名称（可选）</strong>
            <span>如果你想让这台设备在后台里更容易辨认，可以在这里补充一个名字；留空也能正常使用。</span>
          </div>
          <div class="panel-grid">
            <label class="field-block field-span-2">
              <span class="field-label">设备名称（可选）</span>
              <InputText
                :model-value="modelValue.device"
                placeholder="留空则使用默认设备名"
                @update:model-value="updateField('device', $event ?? '')"
              />
            </label>
          </div>
        </div>
      </template>

      <template v-else>
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

            <label class="field-block field-span-2">
              <span class="field-label">设备 Key（自动生成）</span>
              <InputText
                :model-value="currentGeneratedHashKey"
                readonly
                placeholder="首次启动后自动生成"
              />
              <small class="field-help">这个 Key 用于标识当前设备。首次启用或更换后，服务端可能需要重新审核。</small>
            </label>

            <div v-if="reporterSupported" class="reporter-enabled-card field-span-2">
              <div class="reporter-enabled-copy">
                <span class="field-label">使用系统代理</span>
                <strong>{{ modelValue.useSystemProxy ? "已开启" : "已关闭" }}</strong>
                <span>
                  开启后会按系统与运行环境中的代理配置发起请求，常见包括 `HTTP_PROXY`、`HTTPS_PROXY` 和 `ALL_PROXY`。
                </span>
              </div>
              <ToggleSwitch
                :model-value="modelValue.useSystemProxy"
                input-id="settings-use-system-proxy"
                @update:model-value="updateField('useSystemProxy', Boolean($event))"
              />
            </div>
          </div>
        </div>

        <div v-if="reporterSupported" class="settings-section">
          <div class="settings-section-head">
            <strong>后台同步</strong>
            <span>控制后台同步的轮询节奏、心跳频率、自动启动，以及自动上报包含哪些内容。</span>
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

            <div class="reporter-enabled-card field-span-2">
              <div class="reporter-enabled-copy">
                <span class="field-label">启动后自动开启后台同步</span>
                <strong>{{ modelValue.reporterEnabled ? "已开启" : "未开启" }}</strong>
                <span>
                  开启后，这台客户端在下次启动时会自动开始后台同步，适合长期常驻使用。
                </span>
              </div>
              <ToggleSwitch
                :model-value="modelValue.reporterEnabled"
                input-id="settings-reporter-enabled"
                @update:model-value="updateField('reporterEnabled', Boolean($event))"
              />
            </div>

            <div class="settings-section-head field-span-2">
              <strong>自动同步包含内容</strong>
              <span>选择自动同步时要包含的内容。</span>
            </div>

            <div class="compact-toggle-grid field-span-2">
              <div
                v-for="option in reporterContentOptions"
                :key="option.key"
                class="compact-toggle-card"
              >
                <div class="compact-toggle-copy">
                  <strong>{{ option.label }}</strong>
                  <span>{{ option.description }}</span>
                </div>
                <ToggleSwitch
                  :model-value="modelValue[option.key]"
                  :input-id="option.inputId"
                  @update:model-value="updateField(option.key, Boolean($event))"
                />
              </div>
            </div>
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
      </template>

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
