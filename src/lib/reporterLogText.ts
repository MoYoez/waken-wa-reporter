import type { ReporterLogEntry } from "../types";
import { resolveLocalizedText, type TranslateFn } from "./localizedText";

export function resolveReporterLogTitle(log: ReporterLogEntry, t: TranslateFn) {
  return resolveLocalizedText(t, log.titleKey, log.titleParams, log.title);
}

export function resolveReporterLogDetail(log: ReporterLogEntry, t: TranslateFn) {
  return resolveLocalizedText(t, log.detailKey, log.detailParams, log.detail);
}
