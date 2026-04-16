import { invoke } from "@tauri-apps/api/core";

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
};

export function validateConfig(
  config: ClientConfig,
  capabilities: ClientCapabilities = DEFAULT_CAPABILITIES,
) {
  const issues: string[] = [];

  if (!config.baseUrl.trim()) {
    issues.push("Base URL 为必填项。");
  } else {
    try {
      const url = new URL(config.baseUrl.trim());
      if (!["http:", "https:"].includes(url.protocol)) {
        issues.push("Base URL 必须以 http:// 或 https:// 开头。");
      }
    } catch {
      issues.push("Base URL 格式不正确。");
    }
  }

  if (!config.apiToken.trim()) {
    issues.push("API Token 为必填项。");
  }

  if (capabilities.realtimeReporter) {
    if (!Number.isFinite(config.pollIntervalMs) || config.pollIntervalMs < 1000) {
      issues.push("实时上报轮询间隔至少为 1000 毫秒。");
    }

    if (!Number.isFinite(config.heartbeatIntervalMs) || config.heartbeatIntervalMs < 0) {
      issues.push("心跳间隔不能小于 0。");
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
    issues.push("Base URL 为必填项。");
  } else {
    try {
      const url = new URL(config.baseUrl.trim());
      if (!["http:", "https:"].includes(url.protocol)) {
        issues.push("Base URL 必须以 http:// 或 https:// 开头。");
      }
    } catch {
      issues.push("Base URL 格式不正确。");
    }
  }

  if (!config.discordApplicationId.trim()) {
    issues.push("Discord Application ID 为必填项。");
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
    message: payload.message || result.error?.message || "设备待后台审核后可用",
    approvalUrl: payload.approvalUrl || null,
  };
}

export function formatPendingApprovalDetail(info: PendingApprovalInfo) {
  return info.approvalUrl
    ? `设备待后台审核后可用，请前往设备管理完成审核：${info.approvalUrl}`
    : "设备待后台审核后可用，请前往 Waken-Wa 后台的设备管理完成审核。";
}

async function invokeApi<T>(command: string, args?: Record<string, unknown>): Promise<ApiResult<T>> {
  try {
    return await invoke<ApiResult<T>>(command, args);
  } catch (error) {
    return toInvokeError(error instanceof Error ? error.message : "Tauri 调用失败", error);
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
  return invoke<ImportedIntegrationConfig>("parse_imported_integration_config", { input });
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
