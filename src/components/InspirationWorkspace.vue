<script setup lang="ts">
import { computed, onErrorCaptured, onMounted, reactive, ref, watch } from "vue";
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
  getPublicActivityFeed,
  listInspirationEntries,
  uploadInspirationAsset,
  validateConfig,
} from "../lib/api";
import {
  appendParagraphTextToLexical,
  previewInspirationContent,
  renderInspirationContentHtml,
  sanitizeEntryContent as sanitizeRichTextContent,
} from "../lib/inspirationRichText";
import { useInspirationDraftStore } from "../stores/inspirationDraft";
import type { ActivityFeedItem, ClientConfig, InspirationEntry } from "../types";

interface ActivitySelectOption {
  value: string;
  label: string;
  snapshot: string;
  group: "active" | "recent";
  item: ActivityFeedItem;
  deviceName: string;
}

const props = defineProps<{
  config: ClientConfig;
}>();

const toast = useToast();
const draftStore = useInspirationDraftStore();

const compose = reactive({
  title: draftStore.title,
  content: draftStore.content,
  contentLexical: draftStore.contentLexical,
});

const entries = ref<InspirationEntry[]>([]);
const selectedEntry = ref<InspirationEntry | null>(null);
const loading = ref(false);
const submitting = ref(false);
const uploadPending = ref(false);
const inlineUploadPending = ref(false);
const loadError = ref("");
const activityLoadError = ref("");
const inlineImageDataUrl = ref(draftStore.coverImageDataUrl);
const bodyImageInput = ref<HTMLInputElement | null>(null);
const composeTab = ref<"edit" | "preview">(draftStore.composeTab);
const selectedActivityKey = ref(draftStore.selectedActivityKey);
const attachCurrentStatus = ref(draftStore.attachCurrentStatus);
const attachStatusIncludeDeviceInfo = ref(draftStore.attachStatusIncludeDeviceInfo);
const activityOptions = ref<ActivitySelectOption[]>([]);
const activityLoading = ref(false);
const editorFaulted = ref(false);

const configIssues = computed(() => validateConfig(props.config));
const selectedActivityOption = computed(() =>
  activityOptions.value.find((item) => item.value === selectedActivityKey.value) ?? null,
);
const selectedSnapshotPreview = computed(() => {
  if (!attachCurrentStatus.value || !selectedActivityOption.value) return "";
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
  return processName || processTitle || "未命名活动";
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
  const suffix = typeof battery === "number" ? `（${deviceName} · ${battery}%）` : `（${deviceName}）`;
  return `${base} ${suffix}`.trim();
}

function toActivityOptions(items: ActivityFeedItem[], group: "active" | "recent") {
  return items
    .map((item, index) => {
      const idPart = String(item.id ?? `${item.processName ?? "item"}-${index}`);
      const snapshot = buildSnapshotText(item, attachStatusIncludeDeviceInfo.value);
      const device = String(item.device ?? "").trim();
      const prefix = group === "active" ? "当前" : "最近";
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
    return value || "-";
  }
  return date.toLocaleString();
}

async function loadEntries() {
  if (!props.config.baseUrl.trim()) {
    loadError.value = "请先填写站点地址，再加载灵感内容列表。";
    return;
  }

  loading.value = true;
  loadError.value = "";

  const result = await listInspirationEntries(props.config);
  loading.value = false;

  if (!result.success) {
    loadError.value = result.error?.message ?? "内容列表加载失败。";
    return;
  }

  const normalized = (result.data ?? []).map((entry) => ({
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
  entries.value = normalized;
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
    activityLoadError.value = result.error?.message ?? "活动列表加载失败。";
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
    void loadEntries();
    void loadActivityOptions();
  }
});

watch(
  () => props.config.baseUrl.trim(),
  (nextBaseUrl, previousBaseUrl) => {
    if (!nextBaseUrl) {
      entries.value = [];
      loadError.value = "";
      activityOptions.value = [];
      activityLoadError.value = "";
      return;
    }

    if (nextBaseUrl !== previousBaseUrl) {
      void loadEntries();
      void loadActivityOptions();
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
  if (props.config.baseUrl.trim()) {
    void loadActivityOptions();
  }
});

async function onFileSelected(event: Event) {
  const target = event.target as HTMLInputElement | null;
  const file = target?.files?.[0];
  if (!file) return;

  uploadPending.value = true;

  const dataUrl = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ""));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });

  inlineImageDataUrl.value = dataUrl;
  uploadPending.value = false;
  if (target) target.value = "";

  toast.add({
    severity: "success",
    summary: "头图已准备",
    detail: "这张图片会作为本条灵感的头图一并发布。",
    life: 3000,
  });
}

async function onBodyImageSelected(event: Event) {
  const target = event.target as HTMLInputElement | null;
  const file = target?.files?.[0];
  if (!file) return;

  inlineUploadPending.value = true;

  try {
    const dataUrl = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result ?? ""));
      reader.onerror = () => reject(reader.error);
      reader.readAsDataURL(file);
    });

    const result = await uploadInspirationAsset(props.config, dataUrl);
    if (!result.success || !result.data?.url) {
      toast.add({
        severity: "error",
        summary: "附图上传失败",
        detail: result.error?.message ?? "正文附图上传失败。",
        life: 4000,
      });
      return;
    }

    compose.contentLexical = appendParagraphTextToLexical(
      compose.contentLexical,
      `![](${result.data.url})`,
    );

    toast.add({
      severity: "success",
      summary: "附图已插入",
      detail: "图片已经作为正文附图插入到当前灵感中。",
      life: 3000,
    });
  } finally {
    inlineUploadPending.value = false;
    if (target) target.value = "";
  }
}

async function submitEntry() {
  if (configIssues.value.length > 0) {
    toast.add({
      severity: "warn",
      summary: "请先完成连接设置",
      detail: "发布内容前，需要先填写站点地址和 API Token。",
      life: 4000,
    });
    return;
  }

  if (!compose.content.trim()) {
    toast.add({
      severity: "warn",
      summary: "请填写正文内容",
      detail: "发布内容前，正文不能为空。",
      life: 3000,
    });
    return;
  }

  if (attachCurrentStatus.value && !selectedActivityOption.value) {
    toast.add({
      severity: "warn",
      summary: "请选择要附带的活动",
      detail: "开启附带当前状态后，需要选择一条活动记录。",
      life: 3000,
    });
    return;
  }

  const contentToSubmit = compose.content.trim();
  const lexicalToSubmit = compose.contentLexical;
  const attachPayloadEnabled = attachCurrentStatus.value && Boolean(selectedActivityOption.value);

  submitting.value = true;
  const result = await createInspirationEntry(props.config, {
    title: compose.title,
    content: contentToSubmit,
    contentLexical: lexicalToSubmit || undefined,
    imageDataUrl: inlineImageDataUrl.value || undefined,
    generatedHashKey: props.config.generatedHashKey.trim(),
    attachCurrentStatus: attachPayloadEnabled || undefined,
    preComputedStatusSnapshot: attachPayloadEnabled ? selectedSnapshotPreview.value : undefined,
    attachStatusDeviceHash: attachPayloadEnabled ? props.config.generatedHashKey.trim() : undefined,
    attachStatusActivityKey: attachPayloadEnabled ? selectedActivityKey.value : undefined,
    attachStatusIncludeDeviceInfo: attachPayloadEnabled
      ? attachStatusIncludeDeviceInfo.value
      : undefined,
  });
  submitting.value = false;

  if (!result.success) {
    toast.add({
      severity: "error",
      summary: `发布失败（${result.status || "网络"}）`,
      detail: result.error?.message ?? "内容保存失败。",
      life: 4500,
    });
    return;
  }

  toast.add({
    severity: "success",
    summary: "内容已发布",
    detail: "新的内容已同步到 Waken-Wa。",
    life: 3000,
  });

  draftStore.resetDraft();
  compose.title = draftStore.title;
  compose.content = draftStore.content;
  compose.contentLexical = draftStore.contentLexical;
  inlineImageDataUrl.value = draftStore.coverImageDataUrl;
  composeTab.value = draftStore.composeTab;
  selectedActivityKey.value = draftStore.selectedActivityKey;
  attachCurrentStatus.value = draftStore.attachCurrentStatus;
  attachStatusIncludeDeviceInfo.value = draftStore.attachStatusIncludeDeviceInfo;

  void loadEntries();
}
</script>

<template>
  <div class="workspace-grid">
    <Card class="glass-card inspiration-compose">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">灵感创作</p>
            <h3>在桌面端整理想法并发布到 Waken-Wa</h3>
          </div>
          <Button
            label="刷新列表"
            icon="pi pi-refresh"
            severity="secondary"
            text
            :loading="loading"
            @click="loadEntries"
          />
        </div>
      </template>
      <template #content>
        <div class="panel-grid">
          <div class="field-block field-span-2">
            <span class="field-label">标题</span>
            <InputText v-model="compose.title" placeholder="例如：今天突然冒出的一个想法" />
          </div>

          <div class="field-block field-span-2">
            <span class="field-label">关联活动（可选）</span>
            <div class="activity-toggle-row">
              <ToggleSwitch v-model="attachCurrentStatus" input-id="attach-current-status" />
              <label for="attach-current-status">提交时附带当前状态快照</label>
              <ToggleSwitch
                v-model="attachStatusIncludeDeviceInfo"
                input-id="attach-device-info"
                :disabled="!attachCurrentStatus"
              />
              <label for="attach-device-info">快照包含设备与电量</label>
            </div>
            <div class="activity-select-row">
              <Select
                v-model="selectedActivityKey"
                :options="activityOptions"
                option-label="label"
                option-value="value"
                show-clear
                filter
                :loading="activityLoading"
                :disabled="!attachCurrentStatus"
                placeholder="选择一条活动并附带到状态快照"
              />
              <Button
                icon="pi pi-refresh"
                severity="secondary"
                text
                :loading="activityLoading"
                @click="loadActivityOptions"
              />
            </div>
            <small class="field-help">逻辑与 Web 端一致：可选设备活动，提交时由后端生成状态快照。</small>
            <div v-if="attachCurrentStatus && selectedSnapshotPreview" class="snapshot-preview">
              <strong>快照预览：</strong>
              <span>{{ selectedSnapshotPreview }}</span>
            </div>
          </div>

          <div class="field-block field-span-2">
            <span class="field-label">正文</span>
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
                编辑
              </button>
              <button
                type="button"
                class="editor-mode-tab"
                :class="{ active: composeTab === 'preview' }"
                @click="composeTab = 'preview'"
              >
                预览
              </button>
            </div>
            <div class="editor-asset-actions">
              <Button
                label="插入附图"
                icon="pi pi-image"
                text
                size="small"
                :loading="inlineUploadPending"
                @click="bodyImageInput?.click()"
              />
              <small>头图用于封面，附图会插入正文内容里。</small>
            </div>
            <div v-show="composeTab === 'edit'" class="editor-tab-panel">
              <LexicalEditor
                v-if="!editorFaulted"
                v-model="compose.content"
                v-model:lexical-value="compose.contentLexical"
                placeholder="写下一段随想，支持标题、引用、列表、链接和代码。"
              />
              <div v-else class="editor-fallback">
                <Message severity="warn" :closable="false">
                  编辑器暂时切换到兼容模式，已避免页面卡死。
                </Message>
                <Textarea
                  v-model="compose.content"
                  rows="12"
                  auto-resize
                  placeholder="请输入正文内容"
                />
              </div>
            </div>
            <div v-show="composeTab === 'preview'" class="compose-live-preview compose-tab-preview">
              <article class="entry-card compose-preview-card">
                <div class="entry-header">
                  <div>
                    <h4>{{ compose.title.trim() || "未命名条目" }}</h4>
                    <small>尚未发布</small>
                  </div>
                </div>

                <Image
                  v-if="inlineImageDataUrl.trim()"
                  :src="inlineImageDataUrl"
                  alt="Draft cover"
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
                  这里会显示当前正文的最终效果。
                </Message>
              </article>
            </div>
          </div>
        </div>

        <div class="inspiration-upload">
          <label class="upload-label">
            <input type="file" accept="image/*" @change="onFileSelected" />
            <span><i class="pi pi-image" /> 选择头图</span>
          </label>
          <Button
            label="发布内容"
            icon="pi pi-send"
            :loading="submitting || uploadPending"
            @click="submitEntry"
          />
        </div>

        <div class="message-stack">
          <Message v-if="configIssues.length" severity="warn" :closable="false">
            发布内容需要有效的 Token；仅浏览列表时只需要站点地址。
          </Message>
          <Message v-if="loadError" severity="error" :closable="false">
            {{ loadError }}
          </Message>
          <Message v-if="activityLoadError" severity="warn" :closable="false">
            {{ activityLoadError }}
          </Message>
        </div>

        <div v-if="inlineImageDataUrl" class="asset-preview">
          <div>
            <p class="field-label">当前头图</p>
            <strong>发布时会作为灵感头图显示</strong>
            <small>不再把图片地址写进正文内容。</small>
          </div>
          <Image
            :src="inlineImageDataUrl"
            alt="Inspiration cover"
            image-class="inline-preview-image"
          />
        </div>
      </template>
    </Card>

    <Card class="glass-card">
      <template #title>
        <div class="panel-heading">
          <div>
            <p class="eyebrow">灵感列表</p>
            <h3>查看最近发布到 Waken-Wa 的内容</h3>
          </div>
          <Tag :value="`${entries.length} 条记录`" rounded />
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
                <h4>{{ entry.title || "未命名条目" }}</h4>
                <small>{{ formatTime(entry.createdAt) }}</small>
              </div>
              <Tag v-if="entry.statusSnapshot" value="附带状态快照" severity="contrast" rounded />
            </div>

            <Image
              v-if="extractPreviewImage(entry)"
              :src="extractPreviewImage(entry)"
              alt="Entry attachment"
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
              <span>点击查看完整内容</span>
              <i class="pi pi-angle-right" />
            </div>
          </article>
        </div>
        <Message v-else-if="!loading && !loadError" severity="secondary" :closable="false">
          还没有加载到内容。可以先在左侧创作区发布第一条。
        </Message>
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
            <p class="eyebrow">灵感详情</p>
            <h3>{{ selectedEntry.title || "未命名条目" }}</h3>
          </div>
          <small>{{ formatTime(selectedEntry.createdAt) }}</small>
        </div>
      </template>

      <div v-if="selectedEntry" class="entry-detail-content">
        <Image
          v-if="extractPreviewImage(selectedEntry)"
          :src="extractPreviewImage(selectedEntry)"
          alt="Entry attachment"
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
