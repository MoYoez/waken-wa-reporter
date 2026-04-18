import type { LocalizedTextEntry } from "../types";

export type TranslateFn = (key: string, params?: Record<string, any>) => string;

interface ApiErrorLike {
  message?: string;
  code?: string | null;
  params?: Record<string, unknown> | null;
}

export function resolveLocalizedText(
  t: TranslateFn,
  key: string | null | undefined,
  params: Record<string, unknown> | null | undefined,
  fallback: string,
) {
  if (!key) {
    return fallback;
  }

  const result = params ? t(key, params) : t(key);
  return result === key ? fallback : result;
}

export function resolveLocalizedEntry(
  entry: LocalizedTextEntry | null | undefined,
  t: TranslateFn,
  fallback = "",
) {
  if (!entry) {
    return fallback;
  }

  return resolveLocalizedText(t, entry.key, entry.params, entry.text || fallback);
}

export function resolveApiErrorMessage(
  error: ApiErrorLike | null | undefined,
  t: TranslateFn,
  fallback: string,
) {
  if (!error) {
    return fallback;
  }

  return resolveLocalizedText(t, error.code, error.params, error.message || fallback);
}
