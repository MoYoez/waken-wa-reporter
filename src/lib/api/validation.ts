import { translate } from "../../i18n";
import type { ClientCapabilities, ClientConfig } from "../../types";

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

  validateBaseUrl(config.baseUrl, issues);

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
  validateBaseUrl(config.baseUrl, issues);

  if (!config.discordApplicationId.trim()) {
    issues.push(translate("validation.discordAppIdRequired"));
  }

  return issues;
}

function validateBaseUrl(baseUrl: string, issues: string[]) {
  if (!baseUrl.trim()) {
    issues.push(translate("validation.baseUrlRequired"));
    return;
  }

  try {
    const url = new URL(baseUrl.trim());
    if (!["http:", "https:"].includes(url.protocol)) {
      issues.push(translate("validation.baseUrlProtocol"));
    }
  } catch {
    issues.push(translate("validation.baseUrlInvalid"));
  }
}
