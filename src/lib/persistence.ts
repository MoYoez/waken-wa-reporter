import { invoke } from "@tauri-apps/api/core";

import type { AppStatePayload, ClientConfig, RecentPreset } from "../types";

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
      reporterConfigPromptHandled: false,
      verifiedGeneratedHashKey: "",
    };
  }
}

export async function saveAppState(
  config: ClientConfig,
  recentPresets: RecentPreset[],
  onboardingDismissed: boolean,
  reporterConfigPromptHandled: boolean,
  verifiedGeneratedHashKey: string,
) {
  await invoke("save_app_state", {
    payload: {
      config,
      recentPresets,
      onboardingDismissed,
      reporterConfigPromptHandled,
      verifiedGeneratedHashKey,
    },
  });
}
