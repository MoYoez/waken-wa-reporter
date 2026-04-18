import { createI18n } from "vue-i18n";

import { messages } from "./messages";

export const SUPPORTED_LOCALES = ["zh-CN", "en-US"] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

export const DEFAULT_LOCALE: SupportedLocale = "zh-CN";

export function normalizeLocale(value?: string | null): SupportedLocale {
  const normalized = String(value ?? "").trim().toLowerCase();
  if (normalized.startsWith("zh")) {
    return "zh-CN";
  }
  if (normalized.startsWith("en")) {
    return "en-US";
  }
  return DEFAULT_LOCALE;
}

export function detectPreferredLocale(): SupportedLocale {
  if (typeof navigator === "undefined") {
    return DEFAULT_LOCALE;
  }

  const candidates = [...navigator.languages, navigator.language];
  for (const candidate of candidates) {
    const locale = normalizeLocale(candidate);
    if (locale) {
      return locale;
    }
  }

  return DEFAULT_LOCALE;
}

export const localeOptions = SUPPORTED_LOCALES.map((locale) => ({
  value: locale,
  label: messages[locale].localeNames[locale],
}));

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: detectPreferredLocale(),
  fallbackLocale: DEFAULT_LOCALE,
  messages,
});

export function setI18nLocale(value?: string | null): SupportedLocale {
  const locale = normalizeLocale(value);
  i18n.global.locale.value = locale;
  return locale;
}

export function translate(
  key: string,
  values?: Record<string, string | number | boolean | null | undefined>,
) {
  return values ? i18n.global.t(key, values) : i18n.global.t(key);
}
