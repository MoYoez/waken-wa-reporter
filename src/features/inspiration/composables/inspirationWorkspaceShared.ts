import type { ActivityFeedItem, InspirationEntry } from "@/types";

export const MAX_IMAGE_UPLOAD_BYTES = 5 * 1024 * 1024;
export const ENTRY_PAGE_SIZE = 10;
export const SUPPORTED_IMAGE_TYPES = new Set([
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/gif",
]);

export function resolveInspirationAssetUrl(rawUrl: string, baseUrl: string) {
  const value = rawUrl.trim();
  if (!value) {
    return "";
  }

  try {
    return new URL(value).toString();
  } catch {
    try {
      return new URL(value, baseUrl.trim()).toString();
    } catch {
      return value;
    }
  }
}

export function contentOf(entry: InspirationEntry | null | undefined) {
  return typeof entry?.content === "string" ? entry.content : "";
}

export function lexicalOf(entry: InspirationEntry | null | undefined) {
  return typeof entry?.contentLexical === "string" ? entry.contentLexical : "";
}

export function formatInspirationTime(value: string, locale: string, fallback: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value || fallback;
  }
  return date.toLocaleString(locale);
}

export function normalizeInspirationEntries(payload: unknown) {
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
  })) as InspirationEntry[];
}

export function validateInspirationImageFile(file: File, t: (key: string) => string) {
  if (!SUPPORTED_IMAGE_TYPES.has(file.type)) {
    throw new Error(t("inspiration.notify.invalidImageType"));
  }

  if (file.size > MAX_IMAGE_UPLOAD_BYTES) {
    throw new Error(t("inspiration.notify.invalidImageSize"));
  }
}

export function readInspirationFileAsDataUrl(file: File, t: (key: string) => string) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ""));
    reader.onerror = () => reject(reader.error ?? new Error(t("inspiration.notify.fileReadFailed")));
    reader.readAsDataURL(file);
  });
}

export function activityLineText(item: ActivityFeedItem, fallback: string) {
  const statusText = String(item.statusText ?? "").trim();
  if (statusText) {
    return statusText;
  }

  const processName = String(item.processName ?? "").trim();
  const processTitle = String(item.processTitle ?? "").trim();
  if (processTitle && processName) {
    return `${processTitle} | ${processName}`;
  }
  return processName || processTitle || fallback;
}

export function activityBatteryPercent(item: ActivityFeedItem) {
  const metadata = item.metadata;
  if (!metadata || typeof metadata !== "object" || Array.isArray(metadata)) {
    return null;
  }

  const raw = (metadata as Record<string, unknown>).deviceBatteryPercent;
  if (typeof raw !== "number" || !Number.isFinite(raw)) {
    return null;
  }
  return Math.round(raw);
}
