import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import { listInspirationEntries } from "@/lib/api";
import {
  previewInspirationContent,
  renderInspirationContentHtml,
  sanitizeEntryContent as sanitizeRichTextContent,
} from "@/lib/inspirationRichText";
import type { ClientConfig, InspirationEntry } from "@/types";
import type { InspirationEntryCardView } from "@/features/inspiration/types";
import {
  contentOf,
  ENTRY_PAGE_SIZE,
  formatInspirationTime,
  lexicalOf,
  normalizeInspirationEntries,
  resolveInspirationAssetUrl,
} from "@/features/inspiration/composables/inspirationWorkspaceShared";

interface InspirationEntriesOptions {
  config: ClientConfig;
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
}

export function useInspirationEntries(options: InspirationEntriesOptions) {
  const { t, locale } = useI18n();

  const entries = ref<InspirationEntry[]>([]);
  const selectedEntry = ref<InspirationEntry | null>(null);
  const loading = ref(false);
  const loadingMore = ref(false);
  const loadError = ref("");
  const entryTotal = ref(0);

  const hasMoreEntries = computed(() => entryTotal.value > entries.value.length);
  const entryCountLabel = computed(() =>
    entryTotal.value > 0
      ? t("inspiration.count.withTotal", {
          current: entries.value.length,
          total: entryTotal.value,
        })
      : t("inspiration.count.withoutTotal", { current: entries.value.length }),
  );
  const entryCards = computed<InspirationEntryCardView[]>(() =>
    entries.value.map((entry) => ({
      entry,
      createdAtLabel: formatInspirationTime(
        entry.createdAt,
        locale.value,
        t("inspiration.common.unknownTime"),
      ),
      previewImageUrl: resolveInspirationAssetUrl(entry.imageDataUrl?.trim() ?? "", options.config.baseUrl),
      previewText: previewInspirationContent(contentOf(entry), lexicalOf(entry)),
    })),
  );
  const selectedEntryVisible = computed({
    get: () => Boolean(selectedEntry.value),
    set: (visible: boolean) => {
      if (!visible) {
        selectedEntry.value = null;
      }
    },
  });
  const selectedEntryCreatedAtLabel = computed(() =>
    selectedEntry.value
      ? formatInspirationTime(
          selectedEntry.value.createdAt,
          locale.value,
          t("inspiration.common.unknownTime"),
        )
      : "",
  );
  const selectedEntryImageUrl = computed(() =>
    selectedEntry.value
      ? resolveInspirationAssetUrl(selectedEntry.value.imageDataUrl?.trim() ?? "", options.config.baseUrl)
      : "",
  );
  const selectedEntryHtml = computed(() => {
    if (!selectedEntry.value) {
      return "";
    }

    const content = contentOf(selectedEntry.value);
    const lexical = lexicalOf(selectedEntry.value);
    if (!sanitizeRichTextContent(content) && !lexical) {
      return "";
    }

    return renderInspirationContentHtml(
      content,
      lexical,
      (rawUrl) => resolveInspirationAssetUrl(rawUrl, options.config.baseUrl),
    );
  });

  async function loadEntries(params?: { reset?: boolean }) {
    if (!options.config.baseUrl.trim()) {
      loadError.value = t("inspiration.notify.baseUrlRequiredForList");
      return;
    }

    const reset = params?.reset !== false;
    const offset = reset ? 0 : entries.value.length;

    if (reset) {
      loading.value = true;
      loadError.value = "";
    } else {
      if (loadingMore.value || !hasMoreEntries.value) {
        return;
      }
      loadingMore.value = true;
    }

    const result = await listInspirationEntries(options.config, {
      limit: ENTRY_PAGE_SIZE,
      offset,
    });
    loading.value = false;
    loadingMore.value = false;

    if (!result.success) {
      loadError.value = options.apiErrorDetail(result.error, t("inspiration.notify.listLoadFailed"));
      return;
    }

    const normalized = normalizeInspirationEntries(result.data?.data);
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

  function refreshEntries() {
    void loadEntries({ reset: true });
  }

  function loadMoreEntries() {
    void loadEntries({ reset: false });
  }

  function openEntry(entry: InspirationEntry) {
    selectedEntry.value = entry;
  }

  onMounted(() => {
    if (options.config.baseUrl.trim()) {
      refreshEntries();
    }
  });

  watch(
    () => options.config.baseUrl.trim(),
    (nextBaseUrl, previousBaseUrl) => {
      if (!nextBaseUrl) {
        entries.value = [];
        entryTotal.value = 0;
        loadError.value = "";
        return;
      }

      if (nextBaseUrl !== previousBaseUrl) {
        refreshEntries();
      }
    },
  );

  return {
    entryCards,
    entryCountLabel,
    hasMoreEntries,
    loadError,
    loading,
    loadingMore,
    loadMoreEntries,
    openEntry,
    refreshEntries,
    selectedEntry,
    selectedEntryCreatedAtLabel,
    selectedEntryHtml,
    selectedEntryImageUrl,
    selectedEntryVisible,
  };
}
