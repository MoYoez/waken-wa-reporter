import type { ClientConfig, InspirationEntryCreateInput } from "@/types";

interface InspirationComposeState {
  title: string;
  content: string;
  contentLexical: string;
}

interface InspirationSubmissionPayloadOptions {
  attachCurrentStatus: boolean;
  attachStatusIncludeDeviceInfo: boolean;
  buildManualSnapshot: (input: string, includeDeviceInfo: boolean) => string;
  compose: InspirationComposeState;
  config: ClientConfig;
  inlineImageDataUrl: string;
  mobileRuntime: boolean;
  selectedActivityKey: string;
  selectedActivityOption: unknown | null;
  selectedSnapshotPreview: string;
  statusSnapshotInput: string;
}

export function shouldAttachStatusPayload(options: InspirationSubmissionPayloadOptions) {
  return options.attachCurrentStatus
    && (options.mobileRuntime
      ? options.statusSnapshotInput.trim().length > 0
      : Boolean(options.selectedActivityOption));
}

export function buildInspirationEntryPayload(
  options: InspirationSubmissionPayloadOptions,
): InspirationEntryCreateInput {
  const attachPayloadEnabled = shouldAttachStatusPayload(options);

  return {
    title: options.compose.title,
    content: options.compose.content.trim(),
    contentLexical: options.compose.contentLexical || undefined,
    imageDataUrl: options.inlineImageDataUrl || undefined,
    generatedHashKey: options.config.generatedHashKey.trim(),
    attachCurrentStatus: attachPayloadEnabled || undefined,
    preComputedStatusSnapshot: attachPayloadEnabled
      ? (options.mobileRuntime
          ? options.buildManualSnapshot(
              options.statusSnapshotInput,
              options.attachStatusIncludeDeviceInfo,
            )
          : options.selectedSnapshotPreview)
      : undefined,
    attachStatusDeviceHash: attachPayloadEnabled
      ? options.config.generatedHashKey.trim()
      : undefined,
    attachStatusActivityKey: attachPayloadEnabled && !options.mobileRuntime
      ? options.selectedActivityKey
      : undefined,
    attachStatusIncludeDeviceInfo: attachPayloadEnabled
      ? options.attachStatusIncludeDeviceInfo
      : undefined,
  };
}
