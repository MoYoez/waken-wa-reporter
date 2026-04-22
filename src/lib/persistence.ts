import { invoke } from "@tauri-apps/api/core";

import type { ApiError, ApiResult, AppStatePayload, ClientConfig, RecentPreset } from "../types";

export interface AppStateSaveValidationIssue {
  field?: string;
  path?: string;
  reason?: string;
  received?: unknown;
  expected?: string;
  min?: number;
  suggestedValue?: number;
}

export interface AppStateSaveValidationDetails {
  issues?: AppStateSaveValidationIssue[];
  values?: Record<string, number>;
}

function unknownErrorMessage(error: unknown, fallback: string) {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  if (typeof error === "string" && error.trim()) {
    return error;
  }

  return fallback;
}

export class AppStateSaveError extends Error {
  apiError?: ApiError;
  issues: AppStateSaveValidationIssue[];
  values: Record<string, number>;

  constructor(apiError: ApiError | undefined, fallback: string) {
    super(apiError?.message || fallback);
    this.name = "AppStateSaveError";
    this.apiError = apiError;
    const details = apiError?.details as AppStateSaveValidationDetails | undefined;
    this.issues = Array.isArray(details?.issues) ? details.issues : [];
    this.values = details?.values ?? {};
  }
}

export class AppStateSaveTransportError extends Error {
  details: unknown;

  constructor(error: unknown, fallback: string) {
    super(unknownErrorMessage(error, fallback));
    this.name = "AppStateSaveTransportError";
    this.details = error;
  }
}

export const defaultClientConfig = (): ClientConfig => ({
  baseUrl: "",
  apiToken: "",
  generatedHashKey: "",
  useSystemProxy: true,
  device: "",
  deviceType: "desktop",
  pushMode: "realtime",
  pollIntervalMs: 5000,
  heartbeatIntervalMs: 60000,
  reporterMetadataJson: "",
  reporterEnabled: false,
  reportForegroundApp: true,
  reportWindowTitle: true,
  reportMedia: true,
  reportPlaySource: true,
  discordEnabled: false,
  discordApplicationId: "",
  discordSourceId: "",
  launchOnStartup: false,
});

export async function loadAppState(): Promise<AppStatePayload> {
  try {
    return await invoke<AppStatePayload>("load_app_state");
  } catch (error) {
    console.error("[persistence] failed to load app state, fallback to defaults", error);
    return {
      config: defaultClientConfig(),
      recentPresets: [],
      onboardingDismissed: false,
      locale: "",
      reporterConfigPromptHandled: false,
      verifiedGeneratedHashKey: "",
    };
  }
}

export async function saveAppState(
  config: ClientConfig,
  recentPresets: RecentPreset[],
  onboardingDismissed: boolean,
  locale: string,
  reporterConfigPromptHandled: boolean,
  verifiedGeneratedHashKey: string,
) {
  let result: ApiResult<AppStatePayload>;
  try {
    result = await invoke<ApiResult<AppStatePayload>>("save_app_state", {
      payload: {
        config,
        recentPresets,
        onboardingDismissed,
        locale,
        reporterConfigPromptHandled,
        verifiedGeneratedHashKey,
      },
    });
  } catch (error) {
    throw new AppStateSaveTransportError(error, "Tauri save_app_state command failed.");
  }

  if (!result.success) {
    throw new AppStateSaveError(result.error, "Failed to save app state locally.");
  }
}
