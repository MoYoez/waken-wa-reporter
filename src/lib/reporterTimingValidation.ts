import type { TranslateFn } from "@/lib/localizedText";
import type { ClientConfig } from "@/types";

export type ReporterTimingField = "pollIntervalMs" | "heartbeatIntervalMs";
export type ReporterTimingIssueReason = "empty" | "notInteger" | "tooLarge" | "tooSmall";

interface ReporterTimingMeta {
  labelKey: string;
  min: number;
  defaultValue: number;
}

export interface ReporterTimingIssue {
  field: ReporterTimingField;
  path: string;
  reason: ReporterTimingIssueReason;
  received: string | number;
  min: number;
  suggestedValue: number;
}

export interface ReporterTimingInputResult {
  value?: number;
  issue?: ReporterTimingIssue;
}

export const REPORTER_TIMING_FIELDS: Record<ReporterTimingField, ReporterTimingMeta> = {
  pollIntervalMs: {
    labelKey: "connectionPanel.fields.pollInterval",
    min: 1_000,
    defaultValue: 5_000,
  },
  heartbeatIntervalMs: {
    labelKey: "connectionPanel.fields.heartbeatInterval",
    min: 0,
    defaultValue: 60_000,
  },
};

function timingPath(field: ReporterTimingField) {
  return `config.${field}`;
}

function validFallback(field: ReporterTimingField, fallback: unknown) {
  const meta = REPORTER_TIMING_FIELDS[field];
  const parsed = Number(fallback);
  if (Number.isSafeInteger(parsed) && parsed >= meta.min) {
    return parsed;
  }
  return meta.defaultValue;
}

function createIssue(
  field: ReporterTimingField,
  reason: ReporterTimingIssueReason,
  received: string | number,
  suggestedValue: number,
): ReporterTimingIssue {
  return {
    field,
    path: timingPath(field),
    reason,
    received,
    min: REPORTER_TIMING_FIELDS[field].min,
    suggestedValue,
  };
}

export function validateReporterTimingInput(
  field: ReporterTimingField,
  input: unknown,
  fallback: unknown,
): ReporterTimingInputResult {
  const raw = String(input ?? "");
  const trimmed = raw.trim();
  const fallbackValue = validFallback(field, fallback);

  if (!trimmed) {
    return { issue: createIssue(field, "empty", raw, fallbackValue) };
  }

  if (!/^\+?\d+$/.test(trimmed)) {
    return { issue: createIssue(field, "notInteger", raw, fallbackValue) };
  }

  const parsed = Number(trimmed);
  if (!Number.isSafeInteger(parsed)) {
    return { issue: createIssue(field, "tooLarge", raw, fallbackValue) };
  }

  const min = REPORTER_TIMING_FIELDS[field].min;
  if (parsed < min) {
    return { issue: createIssue(field, "tooSmall", raw, min) };
  }

  return { value: parsed };
}

export function validateReporterTimingConfig(config: ClientConfig) {
  const issues: ReporterTimingIssue[] = [];

  (Object.keys(REPORTER_TIMING_FIELDS) as ReporterTimingField[]).forEach((field) => {
    const value = config[field];
    const fallback = validFallback(field, REPORTER_TIMING_FIELDS[field].defaultValue);

    if (!Number.isFinite(value) || !Number.isInteger(value)) {
      issues.push(createIssue(field, "notInteger", String(value), fallback));
      return;
    }

    if (!Number.isSafeInteger(value)) {
      issues.push(createIssue(field, "tooLarge", value, fallback));
      return;
    }

    const min = REPORTER_TIMING_FIELDS[field].min;
    if (value < min) {
      issues.push(createIssue(field, "tooSmall", value, min));
    }
  });

  return issues;
}

export function formatReporterTimingIssue(issue: ReporterTimingIssue, t: TranslateFn) {
  const field = t(REPORTER_TIMING_FIELDS[issue.field].labelKey);
  const messageKey = {
    empty: "validation.integerMsRequired",
    notInteger: "validation.integerMsInvalid",
    tooLarge: "validation.integerMsTooLarge",
    tooSmall: "validation.integerMsTooSmall",
  }[issue.reason];
  const received = String(issue.received).trim() || t("validation.emptyValue");

  return `${t(messageKey, { field, min: issue.min })} ${t("validation.valueSuggestion", {
    received,
    suggested: issue.suggestedValue,
  })}`;
}
