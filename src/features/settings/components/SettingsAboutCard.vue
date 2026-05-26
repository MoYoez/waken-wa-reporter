<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import Button from "primevue/button";
import Card from "primevue/card";
import Message from "primevue/message";

import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { check } from "@tauri-apps/plugin-updater";

import { restartApp } from "@/lib/api";

defineProps<{
  updaterSupported: boolean;
}>();

const { t } = useI18n();

const version = ref("");
const checking = ref(false);
const checkResult = ref<
  | { kind: "idle" }
  | { kind: "uptodate" }
  | { kind: "available"; nextVersion: string }
  | { kind: "installing"; nextVersion: string }
  | { kind: "error"; message: string }
>({ kind: "idle" });

const CLIENT_REPO = "https://github.com/MoYoez/waken-wa-reporter";
const SERVER_REPO = "https://github.com/MoYoez/Waken-Wa";

onMounted(async () => {
  try {
    version.value = await getVersion();
  } catch {
    version.value = "";
  }
});

async function openExternal(url: string) {
  try {
    await openUrl(url);
  } catch {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

async function handleCheckForUpdate() {
  if (checking.value) return;
  checking.value = true;
  checkResult.value = { kind: "idle" };
  try {
    const update = await check();
    if (!update) {
      checkResult.value = { kind: "uptodate" };
      return;
    }
    checkResult.value = { kind: "installing", nextVersion: update.version };
    await update.downloadAndInstall();
    await restartApp();
  } catch (error) {
    checkResult.value = {
      kind: "error",
      message: error instanceof Error ? error.message : String(error),
    };
  } finally {
    checking.value = false;
  }
}
</script>

<template>
  <Card class="glass-card">
    <template #title>
      <div class="panel-heading">
        <div>
          <h3>{{ t("settings.about.title") }}</h3>
        </div>
      </div>
    </template>
    <template #content>
      <div class="about-grid">
        <div class="about-row">
          <span class="about-label">{{ t("settings.about.version") }}</span>
          <strong class="about-value about-value--mono">
            {{ version || "—" }}
          </strong>
        </div>
        <div class="about-row">
          <span class="about-label">{{ t("settings.about.client") }}</span>
          <button
            type="button"
            class="about-link"
            @click="openExternal(CLIENT_REPO)"
          >
            <span>MoYoez/waken-wa-reporter</span>
            <i class="pi pi-external-link" aria-hidden="true" />
          </button>
        </div>
        <div class="about-row">
          <span class="about-label">{{ t("settings.about.server") }}</span>
          <button
            type="button"
            class="about-link"
            @click="openExternal(SERVER_REPO)"
          >
            <span>MoYoez/Waken-Wa</span>
            <i class="pi pi-external-link" aria-hidden="true" />
          </button>
        </div>
      </div>

      <div v-if="updaterSupported" class="actions-row">
        <Button
          :label="t('settings.about.checkUpdate')"
          icon="pi pi-refresh"
          severity="secondary"
          outlined
          :loading="checking"
          @click="handleCheckForUpdate"
        />
      </div>

      <div v-if="updaterSupported" class="message-stack">
        <Message
          v-if="checkResult.kind === 'uptodate'"
          severity="success"
          :closable="false"
        >
          {{ t("settings.about.upToDate") }}
        </Message>
        <Message
          v-else-if="checkResult.kind === 'installing'"
          severity="info"
          :closable="false"
        >
          {{ t("settings.about.installing", { version: checkResult.nextVersion }) }}
        </Message>
        <Message
          v-else-if="checkResult.kind === 'error'"
          severity="warn"
          :closable="false"
        >
          {{ checkResult.message }}
        </Message>
      </div>
    </template>
  </Card>
</template>
