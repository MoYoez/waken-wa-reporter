import { computed, reactive, ref, watch } from "vue";

import { renderInspirationContentHtml } from "@/lib/inspirationRichText";
import { useInspirationDraftStore } from "@/stores/inspirationDraft";
import type { ClientConfig } from "@/types";
import type { InspirationComposeTab } from "@/features/inspiration/types";
import { resolveInspirationAssetUrl } from "@/features/inspiration/composables/inspirationWorkspaceShared";

export function useInspirationDraftState(config: ClientConfig) {
  const draftStore = useInspirationDraftStore();

  const compose = reactive({
    title: draftStore.title,
    content: draftStore.content,
    contentLexical: draftStore.contentLexical,
  });
  const inlineImageDataUrl = ref(draftStore.coverImageDataUrl);
  const composeTab = ref<InspirationComposeTab>(draftStore.composeTab);

  const composePreviewHtml = computed(() => {
    const plainText = compose.content.trim();
    const lexicalText = compose.contentLexical.trim();
    if (!plainText && !lexicalText) {
      return "";
    }

    return renderInspirationContentHtml(
      compose.content,
      compose.contentLexical,
      (rawUrl) => resolveInspirationAssetUrl(rawUrl, config.baseUrl),
    );
  });

  function applyDraftStateFromStore() {
    compose.title = draftStore.title;
    compose.content = draftStore.content;
    compose.contentLexical = draftStore.contentLexical;
    inlineImageDataUrl.value = draftStore.coverImageDataUrl;
    composeTab.value = draftStore.composeTab;
  }

  function resetDraftStore() {
    draftStore.resetDraft();
  }

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

  return {
    applyDraftStateFromStore,
    compose,
    composePreviewHtml,
    composeTab,
    draftStore,
    inlineImageDataUrl,
    resetDraftStore,
  };
}
