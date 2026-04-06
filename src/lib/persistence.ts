import { invoke } from "@tauri-apps/api/core";

import type { AppStatePayload, ClientConfig, RecentPreset } from "../types";

export const defaultClientConfig = (): ClientConfig => ({
  baseUrl: "",
  apiToken: "",
  generatedHashKey: "",
  device: "Waken-Wa Client",
  deviceType: "desktop",
  pushMode: "realtime",
  pollIntervalMs: 2000,
  heartbeatIntervalMs: 60000,
  reporterMetadataJson: "{\n  \"source\": \"waken-wa-client\"\n}",
  reporterEnabled: false,
});

export async function loadAppState(): Promise<AppStatePayload> {
  try {
    return await invoke<AppStatePayload>("load_app_state");
  } catch {
    return {
      config: defaultClientConfig(),
      recentPresets: [],
      onboardingDismissed: false,
      reporterConfigPromptHandled: false,
    };
  }
}

export async function saveAppState(
  config: ClientConfig,
  recentPresets: RecentPreset[],
  onboardingDismissed: boolean,
  reporterConfigPromptHandled: boolean,
) {
  await invoke("save_app_state", {
    payload: {
      config,
      recentPresets,
      onboardingDismissed,
      reporterConfigPromptHandled,
    },
  });
}
