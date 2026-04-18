<script setup lang="ts">
import { computed, onErrorCaptured, onMounted, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import Dialog from "primevue/dialog";
import Image from "primevue/image";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Select from "primevue/select";
import Tag from "primevue/tag";
import Textarea from "primevue/textarea";
import ToggleSwitch from "primevue/toggleswitch";
import { useToast } from "primevue/usetoast";

import LexicalEditor from "./LexicalEditor.vue";
import {
  createInspirationEntry,
  extractPendingApprovalInfo,
  formatPendingApprovalDetail,
  getPublicActivityFeed,
  listInspirationEntries,
  uploadInspirationAsset,
  validateConfig,
} from "../lib/api";
import { readBatterySnapshot } from "../lib/deviceInfo";
import {
  appendParagraphTextToLexical,
  previewInspirationContent,
  renderInspirationContentHtml,
  sanitizeEntryContent as sanitizeRichTextContent,
} from "../lib/inspirationRichText";
import { resolveApiErrorMessage } from "../lib/localizedText";
import { createNotifier } from "../lib/notify";
import { useInspirationDraftStore } from "../stores/inspirationDraft";
import type {
  ActivityFeedItem,
  ClientCapabilities,
  ClientConfig,
  InspirationEntry,
  PendingApprovalInfo,
} from "../types";

interface ActivitySelectOption {
  value: string;
  label: string;
  snapshot: string;
  group: "active" | "recent";
  item: ActivityFeedItem;
  deviceName: string;
}

const MAX_IMAGE_UPLOAD_BYTES = 5 * 1024 * 1024;
const ENTRY_PAGE_SIZE = 10;
const SUPPORTED_IMAGE_TYPES = new Set([
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/gif",
]);

const { t, locale } = useI18n();

const props = defineProps<{
  config: ClientConfig;
  capabilities: ClientCapabilities;
}>();

const emit = defineEmits<{
  pendingApproval: [info: PendingApprovalInfo];
  keyVerified: [generatedHashKey: string];
}>();

const toast = useToast();
const isNativeNotice = computed(() => !props.capabilities.realtimeReporter);
const { notify } = createNotifier(toast, () => isNativeNotice.value);
const draftStore = useInspirationDraftStore();

const compose = reactive({
  title: draftStore.title,
  content: draftStore.content,
  contentLexical: draftStore.contentLexical,
});

const entries = ref<InspirationEntry[]>([]);
const selectedEntry = ref<InspirationEntry | null>(null);
const loading = ref(false);
const loadingMore = ref(false);
const submitting = ref(false);
const uploadPending = ref(false);
const inlineUploadPending = ref(false);
const loadError = ref("");
const activityLoadError = ref("");
const inlineImageDataUrl = ref(draftStore.coverImageDataUrl);
const bodyImageInput = ref<HTMLInputElement | null>(null);
const composeTab = ref<"edit" | "preview">(draftStore.composeTab);
const statusSnapshotInput = ref(draftStore.statusSnapshotInput);
const statusSnapshotDeviceName = ref(draftStore.statusSnapshotDeviceName);
const selectedActivityKey = ref(draftStore.selectedActivityKey);
const attachCurrentStatus = ref(draftStore.attachCurrentStatus);
const attachStatusIncludeDeviceInfo = ref(draftStore.attachStatusIncludeDeviceInfo);
const statusBatteryPercent = ref<number | null>(null);
const activityOptions = ref<ActivitySelectOption[]>([]);
const activityLoading = ref(false);
const editorFaulted = ref(false);
const entryTotal = ref(0);

const mobileRuntime = computed(() => !props.capabilities.realtimeReporter);
const configIssues = computed(() => validateConfig(props.config, props.capabilities));
const selectedActivityOption = computed(() =>
  activityOptions.value.find((item) => item.value === selectedActivityKey.value) ?? null,
);

function snapshotWithDeviceAndBattery(base: string, device: string, battery: string) {
  return t("inspiration.snapshot.withDeviceAndBattery", {
    base,
    device,
    battery,
  });
}

function snapshotWithDevice(base: string, device: string) {
  return t("inspiration.snapshot.withDevice", { base, device });
}

function snapshotWithBattery(base: string, battery: string) {
  return t("inspiration.snapshot.withBattery", { base, battery });
}

function buildManualSnapshot(input: string, includeDeviceInfo: boolean) {
  const base = input.trim();
  if (!base) return "";
  if (!includeDeviceInfo) return base;

  const deviceName = statusSnapshotDeviceName.value.trim() || props.config.device.trim();
  const batteryPart =
    typeof statusBatteryPercent.value === "number"
      ? `${statusBatteryPercent.value}%`
      : "";

  if (deviceName && batteryPart) {
    return snapshotWithDeviceAndBattery(base, deviceName, batteryPart);
  }
  if (deviceName) {
    return snapshotWithDevice(base, deviceName);
  }
  if (batteryPart) {
    return snapshotWithBattery(base, batteryPart);
  }
  return base;
}

const selectedSnapshotPreview = computed(() => {
  if (!attachCurrentStatus.value) return "";
  if (mobileRuntime.value) {
    return buildManualSnapshot(statusSnapshotInput.value, attachStatusIncludeDeviceInfo.value);
  }
  if (!selectedActivityOption.value) return "";
  return selectedActivityOption.value.snapshot;
});
const selectedEntryVisible = computed({
  get: () => Boolean(selectedEntry.value),
  set: (visible: boolean) => {
    if (!visible) {
      selectedEntry.value = null;
    }
  },
});

const hasMoreEntries = computed(() => entryTotal.value > entries.value.length);
const entryCountLabel = computed(() =>
  entryTotal.value > 0
    ? t("inspiration.count.withTotal", {
        current: entries.value.length,
        total: entryTotal.value,
      })
    : t("inspiration.count.withoutTotal", { current: entries.value.length }),
);

onErrorCaptured((error, instance, info) => {
  console.error("[InspirationWorkspace] render error:", error, info, instance);
  editorFaulted.value = true;
  return false;
});

function openEntry(entry: InspirationEntry) {
  selectedEntry.value = entry;
}

function contentOf(entry: InspirationEntry | null | undefined) {
  return typeof entry?.content === "string" ? entry.content : "";
}

function lexicalOf(entry: InspirationEntry | null | undefined) {
  return typeof entry?.contentLexical === "string" ? entry.contentLexical : "";
}

function extractPreviewImage(entry: InspirationEntry) {
  return resolveAssetUrl(entry.imageDataUrl?.trim() ?? "");
}

function resolveAssetUrl(rawUrl: string) {
  const value = rawUrl.trim();
  if (!value) return "";

  try {
    return new URL(value).toString();
  } catch {
    try {
      return new URL(value, props.config.baseUrl.trim()).toString();
    } catch {
      return value;
    }
  }
}

function toLine(item: ActivityFeedItem) {
  const statusText = String(item.statusText ?? "").trim();
  if (statusText) return statusText;

  const processName = String(item.processName ?? "").trim();
  const processTitle = String(item.processTitle ?? "").trim();
  if (processTitle && processName) return `${processTitle} | ${processName}`;
  return processName || processTitle || t("inspiration.common.unnamedActivity");
}

function toBatteryPercent(item: ActivityFeedItem) {
  const metadata = item.metadata;
  if (!metadata || typeof metadata !== "object" || Array.isArray(metadata)) return null;
  const raw = (metadata as Record<string, unknown>).deviceBatteryPercent;
  if (typeof raw !== "number" || !Number.isFinite(raw)) return null;
  return Math.round(raw);
}

function buildSnapshotText(item: ActivityFeedItem, includeDeviceInfo: boolean) {
  const base = toLine(item).trim();
  if (!base) return "";
  if (!includeDeviceInfo) return base;

  const deviceName = String(item.device ?? "").trim();
  if (!deviceName) return base;

  const battery = toBatteryPercent(item);
  if (typeof battery === "number") {
    return snapshotWithDeviceAndBattery(base, deviceName, `${battery}%`);
  }

  return snapshotWithDevice(base, deviceName);
}

function toActivityOptions(items: ActivityFeedItem[], group: "active" | "recent") {
  return items
    .map((item, index) => {
      const idPart = String(item.id ?? `${item.processName ?? "item"}-${index}`);
      const snapshot = buildSnapshotText(item, attachStatusIncludeDeviceInfo.value);
      const device = String(item.device ?? "").trim();
      const prefix = t(`inspiration.activityGroup.${group}`);
      return {
        value: `${group}:${idPart}`,
        label: device ? `${prefix} · ${snapshot} · ${device}` : `${prefix} · ${snapshot}`,
        snapshot,
        group,
        item,
        deviceName: device,
      } satisfies ActivitySelectOption;
    })
    .filter((item) => item.snapshot.trim().length > 0);
}

function formatTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value || t("inspiration.common.unknownTime");
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

function validateImageFile(file: File) {
  if (!SUPPORTED_IMAGE_TYPES.has(file.type)) {
    throw new Error(t("inspiration.notify.invalidImageType"));
  }

  if (file.size > MAX_IMAGE_UPLOAD_BYTES) {
    throw new Error(t("inspiration.notify.invalidImageSize"));
  }
}

function readFileAsDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ""));
    reader.onerror = () => reject(reader.error ?? new Error(t("inspiration.notify.fileReadFailed")));
    reader.readAsDataURL(file);
  });
}

function normalizeEntries(payload: unknown) {
  const rows = Array.isArray(payload) ? payload : [];
  return rows.map((entry) => ({
    ...entry,
    title: typeof entry.title === "string" ? entry.title : null,
    content: typeof entry.content === "string" ? entry.content : "",
    contentLexical: typeof entry.contentLexical === "string" ? entry.contentLexical : null,
    imageDataUrl: typeof entry.imageDataUrl === "string" ? entry.imageDataUrl : null,
    statusSnapshot: typeof entry.statusSnapshot === "string" ? entry.statusSnapshot : null,
    createdAt:
      typeof entry.createdAt === "string" && entry.createdAt.trim()
        ? entry.createdAt
        : new Date().toISOString(),
  }));
}

async function loadEntries(options?: { reset?: boolean }) {
  if (!props.config.baseUrl.trim()) {
    loadError.value = t("inspiration.notify.baseUrlRequiredForList");
    return;
  }

  const reset = options?.reset !== false;
  const offset = reset ? 0 : entries.value.length;

  if (reset) {
    loading.value = true;
    loadError.value = "";
  } else {
    if (loadingMore.value || !hasMoreEntries.value) return;
    loadingMore.value = true;
  }

  const result = await listInspirationEntries(props.config, {
    limit: ENTRY_PAGE_SIZE,
    offset,
  });
  loading.value = false;
  loadingMore.value = false;

  if (!result.success) {
    loadError.value = apiErrorDetail(result.error, t("inspiration.notify.listLoadFailed"));
    return;
  }

  const normalized = normalizeEntries(result.data?.data);
  entryTotal.value = Math.max(
    0,
    Number(result.data?.pagination?.total ?? (reset ? normalized.length : entries.value.length)),
  );

  if (reset) {
    entries.value = normalized;
    return;
  }

  const merged = [...entries.value, ...normalized];
  entries.value = merged.filter(
    (entry, index, all) =>
      index === all.findIndex((candidate) => candidate.id === entry.id),
  );
}

function loadMoreEntries() {
  void loadEntries({ reset: false });
}

async function loadActivityOptions() {
  if (!props.config.baseUrl.trim()) {
    activityOptions.value = [];
    activityLoadError.value = "";
    return;
  }

  activityLoading.value = true;
  activityLoadError.value = "";

  const result = await getPublicActivityFeed(props.config);
  activityLoading.value = false;

  if (!result.success || !result.data) {
    activityLoadError.value = apiErrorDetail(
      result.error,
      t("inspiration.notify.activityLoadFailed"),
    );
    activityOptions.value = [];
    return;
  }

  const activeStatuses = Array.isArray(result.data.activeStatuses)
    ? result.data.activeStatuses
    : [];
  const recentActivities = Array.isArray(result.data.recentActivities)
    ? result.data.recentActivities
    : [];

  const options = [
    ...toActivityOptions(activeStatuses as ActivityFeedItem[], "active"),
    ...toActivityOptions((recentActivities as ActivityFeedItem[]).slice(0, 20), "recent"),
  ];

  const deduped = options.filter(
    (item, index, all) => index === all.findIndex((candidate) => candidate.value === item.value),
  );

  activityOptions.value = deduped;
  if (
    selectedActivityKey.value &&
    !activityOptions.value.some((item) => item.value === selectedActivityKey.value)
  ) {
    selectedActivityKey.value = "";
  }
  pickDefaultActivity();
}

function pickDefaultActivity() {
  if (!attachCurrentStatus.value) return;
  if (selectedActivityKey.value) return;
  if (!activityOptions.value.length) return;

  const preferred = activityOptions.value.find((item) => item.group === "active") ?? activityOptions.value[0];
  selectedActivityKey.value = preferred?.value ?? "";
}

onMounted(() => {
  if (props.config.baseUrl.trim()) {
    void loadEntries({ reset: true });
    if (!mobileRuntime.value) {
      void loadActivityOptions();
    }
  }
});

watch(
  () => props.config.baseUrl.trim(),
  (nextBaseUrl, previousBaseUrl) => {
    if (!nextBaseUrl) {
      entries.value = [];
      entryTotal.value = 0;
      loadError.value = "";
      activityOptions.value = [];
      activityLoadError.value = "";
      return;
    }

    if (nextBaseUrl !== previousBaseUrl) {
      void loadEntries({ reset: true });
      if (!mobileRuntime.value) {
        void loadActivityOptions();
      }
    }
  },
);

watch(
  compose,
  (next) => {
    draftStore.patchDraft({
      title: next.title,
      content: next.content,
      contentLexical: next.contentLexical,
    });
  },
  { deep: true },
);

watch(inlineImageDataUrl, (value) => {
  draftStore.patchDraft({ coverImageDataUrl: value });
});

watch(composeTab, (value) => {
  draftStore.patchDraft({ composeTab: value });
});

watch(statusSnapshotInput, (value) => {
  draftStore.patchDraft({ statusSnapshotInput: value });
});

watch(statusSnapshotDeviceName, (value) => {
  draftStore.patchDraft({ statusSnapshotDeviceName: value });
});

watch(selectedActivityKey, (value) => {
  draftStore.patchDraft({ selectedActivityKey: value });
});

watch(attachCurrentStatus, (value) => {
  draftStore.patchDraft({ attachCurrentStatus: value });
  if (value) {
    pickDefaultActivity();
  }
});

watch(attachStatusIncludeDeviceInfo, (value) => {
  draftStore.patchDraft({ attachStatusIncludeDeviceInfo: value });
  if (!mobileRuntime.value && props.config.baseUrl.trim()) {
    void loadActivityOptions();
  }
});

async function onFileSelected(event: Event) {
  const target = event.target as HTMLInputElement | null;
  const file = target?.files?.[0];
  if (!file) return;

  uploadPending.value = true;
  try {
    validateImageFile(file);
    const dataUrl = await readFileAsDataUrl(file);
    inlineImageDataUrl.value = dataUrl;

    notify({
      severity: "success",
      summary: t("inspiration.notify.coverReady"),
      detail: t("inspiration.notify.coverReadyDetail"),
      life: 3000,
    });
  } catch (error) {
    notify({
      severity: "error",
      summary: t("inspiration.notify.coverReadFailed"),
      detail: error instanceof Error ? error.message : t("inspiration.notify.coverReadFailedDetail"),
      life: 4000,
    });
  } finally {
    uploadPending.value = false;
    if (target) target.value = "";
  }
}

async function onBodyImageSelected(event: Event) {
  const target = event.target as HTMLInputElement | null;
  const file = target?.files?.[0];
  if (!file) return;

  inlineUploadPending.value = true;

  try {
    validateImageFile(file);
    const dataUrl = await readFileAsDataUrl(file);

    const result = await uploadInspirationAsset(props.config, dataUrl);
    const pendingApproval = extractPendingApprovalInfo(result);
    if (pendingApproval) {
      notify({
        severity: "warn",
        summary: t("inspiration.notify.pendingApproval"),
        detail: formatPendingApprovalDetail(pendingApproval),
        life: 6000,
      });
      emit("pendingApproval", pendingApproval);
      return;
    }

    if (!result.success || !result.data?.url) {
      notify({
        severity: "error",
        summary: t("inspiration.notify.inlineImageUploadFailed"),
        detail: apiErrorDetail(
          result.error,
          t("inspiration.notify.inlineImageUploadFailedDetail"),
        ),
        life: 4000,
      });
      return;
    }

    compose.contentLexical = appendParagraphTextToLexical(
      compose.contentLexical,
      `![](${result.data.url})`,
    );
    emit("keyVerified", props.config.generatedHashKey.trim());

    notify({
      severity: "success",
      summary: t("inspiration.notify.inlineImageInserted"),
      detail: t("inspiration.notify.inlineImageInsertedDetail"),
      life: 3000,
    });
  } finally {
    inlineUploadPending.value = false;
    if (target) target.value = "";
  }
}

async function submitEntry() {
  if (configIssues.value.length > 0) {
    notify({
      severity: "warn",
      summary: t("inspiration.notify.settingsRequired"),
      detail: t("inspiration.notify.settingsRequiredDetail"),
      life: 4000,
    });
    return;
  }

  if (!compose.content.trim()) {
    notify({
      severity: "warn",
      summary: t("inspiration.notify.contentRequired"),
      detail: t("inspiration.notify.contentRequiredDetail"),
      life: 3000,
    });
    return;
  }

  if (attachCurrentStatus.value) {
    if (mobileRuntime.value && !statusSnapshotInput.value.trim()) {
      notify({
        severity: "warn",
        summary: t("inspiration.notify.statusInputRequired"),
        detail: t("inspiration.notify.statusInputRequiredDetail"),
        life: 3000,
      });
      return;
    }

    if (!mobileRuntime.value && !selectedActivityOption.value) {
      notify({
        severity: "warn",
        summary: t("inspiration.notify.activityRequired"),
        detail: t("inspiration.notify.activityRequiredDetail"),
        life: 3000,
      });
      return;
    }
  }

  const contentToSubmit = compose.content.trim();
  const lexicalToSubmit = compose.contentLexical;
  const attachPayloadEnabled = attachCurrentStatus.value
    && (mobileRuntime.value ? statusSnapshotInput.value.trim().length > 0 : Boolean(selectedActivityOption.value));

  if (attachPayloadEnabled && mobileRuntime.value && attachStatusIncludeDeviceInfo.value && statusBatteryPercent.value === null) {
    try {
      const battery = await readBatterySnapshot();
      statusBatteryPercent.value = battery.levelPercent;
    } catch {
      // ignore battery read failure; snapshot will fallback to device only
    }
  }

  submitting.value = true;
  const result = await createInspirationEntry(props.config, {
    title: compose.title,
    content: contentToSubmit,
    contentLexical: lexicalToSubmit || undefined,
    imageDataUrl: inlineImageDataUrl.value || undefined,
    generatedHashKey: props.config.generatedHashKey.trim(),
    attachCurrentStatus: attachPayloadEnabled || undefined,
    preComputedStatusSnapshot: attachPayloadEnabled
      ? (mobileRuntime.value
          ? buildManualSnapshot(statusSnapshotInput.value, attachStatusIncludeDeviceInfo.value)
          : selectedSnapshotPreview.value)
      : undefined,
    attachStatusDeviceHash: attachPayloadEnabled ? props.config.generatedHashKey.trim() : undefined,
    attachStatusActivityKey: attachPayloadEnabled && !mobileRuntime.value ? selectedActivityKey.value : undefined,
    attachStatusIncludeDeviceInfo: attachPayloadEnabled
      ? attachStatusIncludeDeviceInfo.value
      : undefined,
  });
  submitting.value = false;

  const pendingApproval = extractPendingApprovalInfo(result);
  if (pendingApproval) {
    notify({
      severity: "warn",
      summary: t("inspiration.notify.pendingApproval"),
      detail: formatPendingApprovalDetail(pendingApproval),
      life: 6000,
    });
    emit("pendingApproval", pendingApproval);
    return;
  }

  if (!result.success) {
    notify({
      severity: "error",
      summary: t("inspiration.notify.submitFailed", {
        status: result.status || t("inspiration.common.network"),
      }),
      detail: apiErrorDetail(result.error, t("inspiration.notify.submitFailedDetail")),
      life: 4500,
    });
    return;
  }

  emit("keyVerified", props.config.generatedHashKey.trim());

  notify({
    severity: "success",
    summary: t("inspiration.notify.submitSuccess"),
    detail: t("inspiration.notify.submitSuccessDetail"),
    life: 3000,
  });

  draftStore.resetDraft();
  compose.title = draftStore.title;
  compose.content = draftStore.content;
  compose.contentLexical = draftStore.contentLexical;
  inlineImageDataUrl.value = draftStore.coverImageDataUrl;
  composeTab.value = draftStore.composeTab;
  statusSnapshotInput.value = draftStore.statusSnapshotInput;
  statusSnapshotDeviceName.value = draftStore.statusSnapshotDeviceName;
  selectedActivityKey.value = draftStore.selectedActivityKey;
  attachCurrentStatus.value = draftStore.attachCurrentStatus;
  attachStatusIncludeDeviceInfo.value = draftStore.attachStatusIncludeDeviceInfo;
  statusBatteryPercent.value = null;

  void loadEntries({ reset: true });
}
</script>

<template>
  <div class="workspace-grid">
    <Card class="glass-card inspiration-compose">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("inspiration.title.eyebrow") }}</p>
            <h3>{{ t("inspiration.title.title") }}</h3>
          </div>
          <Button
            :label="t('inspiration.buttons.refresh')"
            icon="pi pi-refresh"
            severity="secondary"
            text
            :loading="loading"
            @click="loadEntries({ reset: true })"
          />
        </div>
      </template>
      <template #content>
        <div class="panel-grid">
          <div class="field-block field-span-2">
            <span class="field-label">{{ t("inspiration.fields.title") }}</span>
            <InputText v-model="compose.title" :placeholder="t('inspiration.placeholders.title')" />
          </div>

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
                @click="loadActivityOptions"
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

          <div class="field-block field-span-2">
            <span class="field-label">{{ t("inspiration.fields.body") }}</span>
            <input
              ref="bodyImageInput"
              type="file"
              accept="image/*"
              class="sr-only"
              @change="onBodyImageSelected"
            />
            <div class="editor-mode-tabs">
              <button
                type="button"
                class="editor-mode-tab"
                :class="{ active: composeTab === 'edit' }"
                @click="composeTab = 'edit'"
              >
                {{ t("inspiration.tabs.edit") }}
              </button>
              <button
                type="button"
                class="editor-mode-tab"
                :class="{ active: composeTab === 'preview' }"
                @click="composeTab = 'preview'"
              >
                {{ t("inspiration.tabs.preview") }}
              </button>
            </div>
            <div class="editor-asset-actions">
              <Button
                :label="t('inspiration.buttons.insertInlineImage')"
                icon="pi pi-image"
                text
                size="small"
                :loading="inlineUploadPending"
                @click="bodyImageInput?.click()"
              />
              <small>{{ t("inspiration.help.inlineImage") }}</small>
            </div>
            <div v-show="composeTab === 'edit'" class="editor-tab-panel">
              <LexicalEditor
                v-if="!editorFaulted"
                v-model="compose.content"
                v-model:lexical-value="compose.contentLexical"
                :placeholder="t('inspiration.placeholders.editor')"
              />
              <div v-else class="editor-fallback">
                <Message severity="warn" :closable="false">
                  {{ t("inspiration.help.editorFallback") }}
                </Message>
                <Textarea
                  v-model="compose.content"
                  rows="12"
                  auto-resize
                  :placeholder="t('inspiration.placeholders.fallbackEditor')"
                />
              </div>
            </div>
            <div v-show="composeTab === 'preview'" class="compose-live-preview compose-tab-preview">
              <article class="entry-card compose-preview-card">
                <div class="entry-header">
                  <div>
                    <h4>{{ compose.title.trim() || t("inspiration.common.untitledEntry") }}</h4>
                    <small>{{ t("inspiration.help.unpublished") }}</small>
                  </div>
                </div>

                <Image
                  v-if="inlineImageDataUrl.trim()"
                  :src="inlineImageDataUrl"
                  :alt="t('inspiration.imageAlt.draftCover')"
                  image-class="entry-detail-image"
                />

                <div
                  v-if="compose.content.trim() || compose.contentLexical.trim()"
                  class="entry-content markdown-content"
                  v-html="renderInspirationContentHtml(compose.content, compose.contentLexical, resolveAssetUrl)"
                />
                <Message
                  v-else
                  severity="secondary"
                  :closable="false"
                >
                  {{ t("inspiration.help.previewEmpty") }}
                </Message>
              </article>
            </div>
          </div>
        </div>

        <div class="inspiration-upload">
          <label class="upload-label">
            <input
              type="file"
              accept="image/png,image/jpeg,image/webp,image/gif"
              @change="onFileSelected"
            />
            <span><i class="pi pi-image" /> {{ t("inspiration.buttons.selectCover") }}</span>
          </label>
          <Button
            :label="t('inspiration.buttons.submit')"
            icon="pi pi-send"
            :loading="submitting || uploadPending"
            @click="submitEntry"
          />
        </div>

        <div class="message-stack">
          <Message v-if="configIssues.length" severity="warn" :closable="false">
            {{ t("inspiration.help.configIssues") }}
          </Message>
          <Message v-if="loadError" severity="error" :closable="false">
            {{ loadError }}
          </Message>
          <Message v-if="activityLoadError" severity="warn" :closable="false">
            {{ activityLoadError }}
          </Message>
          <Message severity="secondary" :closable="false">
            {{ t("inspiration.help.uploadHint") }}
          </Message>
        </div>

        <div v-if="inlineImageDataUrl" class="asset-preview">
          <div>
            <p class="field-label">{{ t("inspiration.fields.currentCover") }}</p>
            <strong>{{ t("inspiration.help.currentCoverTitle") }}</strong>
            <small>{{ t("inspiration.help.currentCoverDetail") }}</small>
          </div>
          <Image
            :src="inlineImageDataUrl"
            :alt="t('inspiration.imageAlt.coverPreview')"
            image-class="inline-preview-image"
          />
        </div>
      </template>
    </Card>

    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("inspiration.list.eyebrow") }}</p>
            <h3>{{ t("inspiration.list.title") }}</h3>
          </div>
          <Tag :value="entryCountLabel" rounded />
        </div>
      </template>
      <template #content>
        <div v-if="entries.length" class="entry-list">
          <article
            v-for="(entry, index) in entries"
            :key="`inspiration-${entry.id ?? 'na'}-${entry.createdAt}-${index}`"
            class="entry-card entry-card-button"
            @click="openEntry(entry)"
          >
            <div class="entry-header">
              <div>
                <h4>{{ entry.title || t("inspiration.common.untitledEntry") }}</h4>
                <small>{{ formatTime(entry.createdAt) }}</small>
              </div>
              <Tag
                v-if="entry.statusSnapshot"
                :value="t('inspiration.list.statusSnapshotTag')"
                severity="contrast"
                rounded
              />
            </div>

            <Image
              v-if="extractPreviewImage(entry)"
              :src="extractPreviewImage(entry)"
              :alt="t('inspiration.imageAlt.entryAttachment')"
              image-class="entry-preview-image"
            />

            <p
              v-if="previewInspirationContent(contentOf(entry), lexicalOf(entry))"
              class="entry-content entry-preview-text"
            >
              {{ previewInspirationContent(contentOf(entry), lexicalOf(entry)) }}
            </p>
            <blockquote v-if="entry.statusSnapshot" class="status-snapshot">
              {{ entry.statusSnapshot }}
            </blockquote>
            <div class="entry-card-footer">
              <span>{{ t("inspiration.list.viewFull") }}</span>
              <i class="pi pi-angle-right" />
            </div>
          </article>
        </div>
        <Message v-else-if="!loading && !loadError" severity="secondary" :closable="false">
          {{ t("inspiration.list.empty") }}
        </Message>
        <div v-if="entries.length && hasMoreEntries" class="entry-list-actions">
          <Button
            :label="t('inspiration.list.loadMore')"
            icon="pi pi-angle-down"
            severity="secondary"
            outlined
            :loading="loadingMore"
            @click="loadMoreEntries"
          />
        </div>
      </template>
    </Card>

    <Dialog
      v-model:visible="selectedEntryVisible"
      modal
      dismissable-mask
      :draggable="false"
      class="entry-detail-dialog"
    >
      <template #header>
        <div v-if="selectedEntry" class="panel-heading">
          <div>
            <p class="eyebrow">{{ t("inspiration.list.detailEyebrow") }}</p>
            <h3>{{ selectedEntry.title || t("inspiration.common.untitledEntry") }}</h3>
          </div>
          <small>{{ formatTime(selectedEntry.createdAt) }}</small>
        </div>
      </template>

      <div v-if="selectedEntry" class="entry-detail-content">
        <Image
          v-if="extractPreviewImage(selectedEntry)"
          :src="extractPreviewImage(selectedEntry)"
          :alt="t('inspiration.imageAlt.entryAttachment')"
          image-class="entry-detail-image"
        />
        <div
          v-if="sanitizeRichTextContent(contentOf(selectedEntry)) || lexicalOf(selectedEntry)"
          class="entry-content markdown-content"
          v-html="renderInspirationContentHtml(contentOf(selectedEntry), lexicalOf(selectedEntry), resolveAssetUrl)"
        />
        <blockquote v-if="selectedEntry.statusSnapshot" class="status-snapshot">
          {{ selectedEntry.statusSnapshot }}
        </blockquote>
      </div>
    </Dialog>
  </div>
</template>
