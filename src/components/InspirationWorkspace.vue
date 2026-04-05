<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import Button from "primevue/button";
import Card from "primevue/card";
import Image from "primevue/image";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Tag from "primevue/tag";
import Textarea from "primevue/textarea";
import { useToast } from "primevue/usetoast";

import {
  createInspirationEntry,
  listInspirationEntries,
  uploadInspirationAsset,
  validateConfig,
} from "../lib/api";
import type {
  ClientConfig,
  InspirationAssetUploadResult,
  InspirationEntry,
} from "../types";

const props = defineProps<{
  config: ClientConfig;
}>();

const toast = useToast();

const compose = reactive({
  title: "",
  content: "",
});

const entries = ref<InspirationEntry[]>([]);
const loading = ref(false);
const submitting = ref(false);
const uploadPending = ref(false);
const loadError = ref("");
const uploadedAsset = ref<InspirationAssetUploadResult | null>(null);
const inlineImageDataUrl = ref("");

const configIssues = computed(() => validateConfig(props.config));

function extractPreviewImage(entry: InspirationEntry) {
  if (entry.imageDataUrl?.trim()) return entry.imageDataUrl;

  const match = entry.content.match(/!\[[^\]]*]\(([^)]+)\)/);
  return match?.[1] ?? "";
}

function formatTime(value: string) {
  return new Date(value).toLocaleString();
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

  entries.value = result.data ?? [];
}

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

  const result = await uploadInspirationAsset(props.config, dataUrl);
  uploadPending.value = false;
  if (target) target.value = "";

  if (!result.success || !result.data) {
    toast.add({
      severity: "error",
      summary: "图片上传失败",
      detail: result.error?.message ?? "图片上传失败。",
      life: 4000,
    });
    return;
  }

  uploadedAsset.value = result.data;

  const markdown = `\n\n![inspiration-image](${result.data.url})`;
  if (!compose.content.includes(result.data.url)) {
    compose.content += markdown;
  }

  toast.add({
    severity: "success",
    summary: "图片已上传",
    detail: "资源地址已自动插入到草稿内容中。",
    life: 3000,
  });
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

  submitting.value = true;
  const result = await createInspirationEntry(props.config, {
    title: compose.title,
    content: compose.content,
    generatedHashKey: props.config.generatedHashKey.trim(),
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

  compose.title = "";
  compose.content = "";
  uploadedAsset.value = null;
  inlineImageDataUrl.value = "";

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
          <label class="field-block field-span-2">
            <span class="field-label">标题</span>
            <InputText v-model="compose.title" placeholder="例如：今天突然冒出的一个想法" />
          </label>
          <label class="field-block field-span-2">
            <span class="field-label">正文</span>
            <Textarea
              v-model="compose.content"
              rows="10"
              auto-resize
              placeholder="写下一段随想，或粘贴你想同步到 Waken-Wa 的内容。"
            />
          </label>
        </div>

        <div class="inspiration-upload">
          <label class="upload-label">
            <input type="file" accept="image/*" @change="onFileSelected" />
            <span><i class="pi pi-image" /> 上传图片</span>
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
        </div>

        <div v-if="uploadedAsset" class="asset-preview">
          <div>
            <p class="field-label">最近上传的图片</p>
            <strong>{{ uploadedAsset.publicKey }}</strong>
            <small>{{ uploadedAsset.url }}</small>
          </div>
          <Image
            v-if="inlineImageDataUrl"
            :src="inlineImageDataUrl"
            alt="Uploaded inspiration asset"
            preview
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
          <article v-for="entry in entries" :key="entry.id" class="entry-card">
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
              preview
              image-class="entry-preview-image"
            />

            <p class="entry-content">{{ entry.content }}</p>
            <blockquote v-if="entry.statusSnapshot" class="status-snapshot">
              {{ entry.statusSnapshot }}
            </blockquote>
          </article>
        </div>
        <Message v-else-if="!loading && !loadError" severity="secondary" :closable="false">
          还没有加载到内容。可以先在左侧创作区发布第一条。
        </Message>
      </template>
    </Card>
  </div>
</template>
