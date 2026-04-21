import {
  resolveLocalizedEntry,
  resolveLocalizedText,
} from "@/lib/localizedText";
import type { PlatformProbeResult, PlatformSelfTestResult } from "@/types";

export interface SelfTestCardView {
  key: "foreground" | "windowTitle" | "media";
  titleKey: string;
  success: boolean;
  primaryText: string;
  secondaryText: string;
  showAccessibilityAction?: boolean;
}

type TranslateFn = (key: string, params?: Record<string, unknown>) => string;

export function buildSelfTestCardViews(
  result: PlatformSelfTestResult | null,
  t: TranslateFn,
) {
  if (!result) {
    return [];
  }

  return [
    {
      key: "foreground",
      titleKey: "settings.selfTest.foreground",
      success: result.foreground.success,
      primaryText: primaryProbeText(result.foreground, t),
      secondaryText: secondaryProbeText(result.foreground, t),
    },
    {
      key: "windowTitle",
      titleKey: "settings.selfTest.windowTitle",
      success: result.windowTitle.success,
      primaryText: primaryProbeText(result.windowTitle, t),
      secondaryText: secondaryProbeText(result.windowTitle, t),
      showAccessibilityAction:
        result.platform === "macos"
        && !result.windowTitle.success,
    },
    {
      key: "media",
      titleKey: "settings.selfTest.media",
      success: result.media.success,
      primaryText: primaryProbeText(result.media, t),
      secondaryText: secondaryProbeText(result.media, t),
    },
  ] satisfies SelfTestCardView[];
}

export function resolveSelfTestPlatformHintKey(result: PlatformSelfTestResult | null) {
  if (result?.platform === "macos") {
    return "settings.selfTest.macosHint";
  }
  if (result?.platform === "linux") {
    return "settings.selfTest.linuxHint";
  }
  return "";
}

function firstGuidance(probe: PlatformProbeResult, t: TranslateFn) {
  const localized = probe.guidanceEntries
    ?.map((entry) => resolveLocalizedEntry(entry, t))
    .find((item) => item.trim());
  if (localized) {
    return localized;
  }

  return probe.guidance?.find((item) => item.trim()) ?? "";
}

function probeSummary(probe: PlatformProbeResult, t: TranslateFn) {
  return resolveLocalizedText(
    t,
    probe.summaryKey,
    probe.summaryParams,
    probe.summary,
  );
}

function probeDetail(probe: PlatformProbeResult, t: TranslateFn) {
  return resolveLocalizedText(
    t,
    probe.detailKey,
    probe.detailParams,
    probe.detail,
  );
}

function compactDetail(value: string | null | undefined, t: TranslateFn) {
  const text = (value ?? "").trim();
  if (!text) {
    return t("settings.notify.noneResult");
  }

  const normalized = text.replace(/\s+/g, " ");
  if (normalized.length <= 88) {
    return normalized;
  }

  const firstChunk = normalized.split(/[；;]/)[0]?.trim() || normalized;
  if (firstChunk.length <= 88) {
    return firstChunk;
  }

  return `${firstChunk.slice(0, 84).trimEnd()}...`;
}

function primaryProbeText(probe: PlatformProbeResult, t: TranslateFn) {
  return probe.success ? compactDetail(probeDetail(probe, t), t) : probeSummary(probe, t);
}

function secondaryProbeText(probe: PlatformProbeResult, t: TranslateFn) {
  if (probe.success) {
    return "";
  }

  return firstGuidance(probe, t) || probeDetail(probe, t);
}
