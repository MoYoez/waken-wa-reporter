import { invoke } from "@tauri-apps/api/core";
import {
  disable as disableAutostartPlugin,
  enable as enableAutostartPlugin,
  isEnabled as isAutostartEnabledPlugin,
} from "@tauri-apps/plugin-autostart";

import { translate } from "../i18n";
import { resolveApiErrorMessage } from "./localizedText";
import type {
  ActivityFeedData,
  ActivityPayload,
  ApiResult,
  ClientCapabilities,
  ClientConfig,
  DiscordPresenceSnapshot,
  ExistingReporterConfig,
  ImportedIntegrationConfig,
  InspirationAssetUploadResult,
  InspirationEntry,
  InspirationEntryListResponse,
  InspirationEntryCreateInput,
  PendingApprovalInfo,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "../types";

function toInvokeError(message: string, details?: unknown): ApiResult<never> {
  return {
    success: false,
    status: 0,
    error: {
      status: 0,
      message,
      details,
    },
  };
}

const DEFAULT_CAPABILITIES: ClientCapabilities = {
  realtimeReporter: true,
  tray: true,
  platformSelfTest: true,
  discordPresence: true,
  autostart: true,
};

export function validateConfig(
  config: ClientConfig,
  capabilities: ClientCapabilities = DEFAULT_CAPABILITIES,
) {
  const issues: string[] = [];

  if (!config.baseUrl.trim()) {
    issues.push(translate("validation.baseUrlRequired"));
  } else {
    try {
      const url = new URL(config.baseUrl.trim());
      if (!["http:", "https:"].includes(url.protocol)) {
        issues.push(translate("validation.baseUrlProtocol"));
      }
    } catch {
      issues.push(translate("validation.baseUrlInvalid"));
    }
  }

  if (!config.apiToken.trim()) {
    issues.push(translate("validation.apiTokenRequired"));
  }

  if (capabilities.realtimeReporter) {
    if (!Number.isFinite(config.pollIntervalMs) || config.pollIntervalMs < 1000) {
      issues.push(translate("validation.pollIntervalMin"));
    }

    if (!Number.isFinite(config.heartbeatIntervalMs) || config.heartbeatIntervalMs < 0) {
      issues.push(translate("validation.heartbeatNonNegative"));
    }
  }

  return issues;
}

export function validateDiscordPresenceConfig(
  config: ClientConfig,
  capabilities: ClientCapabilities = DEFAULT_CAPABILITIES,
) {
  if (!capabilities.discordPresence) {
    return [];
  }

  const issues: string[] = [];

  if (!config.baseUrl.trim()) {
    issues.push(translate("validation.baseUrlRequired"));
  } else {
    try {
      const url = new URL(config.baseUrl.trim());
      if (!["http:", "https:"].includes(url.protocol)) {
        issues.push(translate("validation.baseUrlProtocol"));
      }
    } catch {
      issues.push(translate("validation.baseUrlInvalid"));
    }
  }

  if (!config.discordApplicationId.trim()) {
    issues.push(translate("validation.discordAppIdRequired"));
  }

  return issues;
}

function pendingApprovalPayload(
  candidate: unknown,
): { pending: boolean; message: string; approvalUrl: string } | null {
  if (!candidate || typeof candidate !== "object" || Array.isArray(candidate)) {
    return null;
  }

  const payload = candidate as Record<string, unknown>;
  if (payload.pending !== true) {
    return null;
  }

  return {
    pending: true,
    message:
      typeof payload.error === "string"
        ? payload.error
        : typeof payload.message === "string"
          ? payload.message
          : "",
    approvalUrl: typeof payload.approvalUrl === "string" ? payload.approvalUrl : "",
  };
}

export function extractPendingApprovalInfo(
  result: ApiResult<unknown>,
): PendingApprovalInfo | null {
  if (result.status !== 202) {
    return null;
  }

  const payload =
    pendingApprovalPayload(result.error?.details)
    ?? pendingApprovalPayload(result.data);

  if (!payload) {
    return null;
  }

  return {
    message:
      payload.message
      || resolveApiErrorMessage(
        result.error,
        translate,
        translate("errors.pendingApprovalDefault"),
      )
      || translate("errors.pendingApprovalDefault"),
    approvalUrl: payload.approvalUrl || null,
  };
}

export function formatPendingApprovalDetail(info: PendingApprovalInfo) {
  return info.approvalUrl
    ? translate("errors.pendingApprovalWithUrl", { approvalUrl: info.approvalUrl })
    : translate("errors.pendingApprovalWithoutUrl");
}

async function invokeApi<T>(command: string, args?: Record<string, unknown>): Promise<ApiResult<T>> {
  try {
    return await invoke<ApiResult<T>>(command, args);
  } catch (error) {
    return toInvokeError(
      error instanceof Error ? error.message : translate("errors.tauriInvokeFailed"),
      error,
    );
  }
}

export async function getClientCapabilities(): Promise<ApiResult<ClientCapabilities>> {
  return invokeApi("get_client_capabilities");
}

export async function submitActivityReport(
  config: ClientConfig,
  payload: ActivityPayload,
): Promise<ApiResult<Record<string, unknown>>> {
  return invokeApi("submit_activity_report", { config, payload });
}

export async function getPublicActivityFeed(
  config: ClientConfig,
): Promise<ApiResult<ActivityFeedData>> {
  return invokeApi("get_public_activity_feed", { config });
}

export async function listInspirationEntries(
  config: ClientConfig,
  options?: { limit?: number; offset?: number },
): Promise<ApiResult<InspirationEntryListResponse>> {
  return invokeApi("list_inspiration_entries", {
    config,
    limit: options?.limit,
    offset: options?.offset,
  });
}

export async function probeConnectivity(
  config: ClientConfig,
): Promise<ApiResult<Record<string, unknown>>> {
  return invokeApi("probe_connectivity", { config });
}

export async function createInspirationEntry(
  config: ClientConfig,
  input: InspirationEntryCreateInput,
): Promise<ApiResult<InspirationEntry>> {
  return invokeApi("create_inspiration_entry", { config, input });
}

export async function uploadInspirationAsset(
  config: ClientConfig,
  imageDataUrl: string,
): Promise<ApiResult<InspirationAssetUploadResult>> {
  return invokeApi("upload_inspiration_asset", { config, imageDataUrl });
}

export async function parseImportedIntegrationConfig(
  input: string,
): Promise<ImportedIntegrationConfig> {
  const result = await invokeApi<ImportedIntegrationConfig>(
    "parse_imported_integration_config",
    { input },
  );

  if (!result.success || !result.data) {
    throw new Error(
      resolveApiErrorMessage(
        result.error,
        translate,
        translate("connectionPanel.notify.importFailedDetail"),
      ),
    );
  }

  return result.data;
}

export async function hideToTray(): Promise<void> {
  await invoke("hide_to_tray");
}

export async function startRealtimeReporter(
  config: ClientConfig,
): Promise<ApiResult<RealtimeReporterSnapshot>> {
  return invokeApi("start_realtime_reporter", { config });
}

export async function stopRealtimeReporter(): Promise<ApiResult<RealtimeReporterSnapshot>> {
  return invokeApi("stop_realtime_reporter");
}

export async function getRealtimeReporterSnapshot(): Promise<ApiResult<RealtimeReporterSnapshot>> {
  return invokeApi("get_realtime_reporter_snapshot");
}

export async function startDiscordPresenceSync(
  config: ClientConfig,
): Promise<ApiResult<DiscordPresenceSnapshot>> {
  return invokeApi("start_discord_presence_sync", { config });
}

export async function stopDiscordPresenceSync(): Promise<ApiResult<DiscordPresenceSnapshot>> {
  return invokeApi("stop_discord_presence_sync");
}

export async function getDiscordPresenceSnapshot(): Promise<ApiResult<DiscordPresenceSnapshot>> {
  return invokeApi("get_discord_presence_snapshot");
}

export async function runPlatformSelfTest(): Promise<ApiResult<PlatformSelfTestResult>> {
  return invokeApi("run_platform_self_test");
}

export async function requestAccessibilityPermission(): Promise<ApiResult<boolean>> {
  return invokeApi("request_accessibility_permission");
}

export async function discoverExistingReporterConfig(): Promise<ApiResult<ExistingReporterConfig>> {
  return invokeApi("discover_existing_reporter_config");
}

export async function restartApp(): Promise<void> {
  await invoke("restart_app");
}

export async function isAutostartEnabled(): Promise<boolean> {
  return isAutostartEnabledPlugin();
}

export async function setAutostartEnabled(enabled: boolean): Promise<void> {
  if (enabled) {
    await enableAutostartPlugin();
    return;
  }

  await disableAutostartPlugin();
}
