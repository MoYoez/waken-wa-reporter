import { invoke } from "@tauri-apps/api/core";

import { translate } from "../../i18n";
import { resolveApiErrorMessage } from "../localizedText";
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
  InspirationEntryCreateInput,
  InspirationEntryListResponse,
  PlatformSelfTestResult,
  RealtimeReporterSnapshot,
} from "../../types";
import { invokeApi } from "./invoke";

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
