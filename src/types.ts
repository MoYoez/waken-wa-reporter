export type DeviceType = "desktop" | "tablet" | "mobile";

export type PushMode = "realtime" | "active";

export interface ClientConfig {
  baseUrl: string;
  apiToken: string;
  generatedHashKey: string;
  device: string;
  deviceType: DeviceType;
  pushMode: PushMode;
  pollIntervalMs: number;
  heartbeatIntervalMs: number;
  reporterMetadataJson: string;
  reporterEnabled: boolean;
}

export interface ActivityMedia {
  title: string;
  singer?: string;
}

export interface ActivityMetadata extends Record<string, unknown> {
  source?: string;
  play_source?: string;
  media?: ActivityMedia;
}

export interface ActivityPayload {
  generatedHashKey: string;
  process_name: string;
  device?: string;
  process_title?: string;
  persist_minutes?: number;
  battery_level?: number;
  is_charging?: boolean;
  device_type?: DeviceType;
  push_mode?: PushMode;
  metadata?: ActivityMetadata;
}

export interface ActivityFeedItem {
  id?: number | string;
  device?: string;
  processName?: string;
  processTitle?: string | null;
  statusText?: string;
  startedAt?: string;
  updatedAt?: string;
  endedAt?: string | null;
  metadata?: Record<string, unknown> | null;
}

export interface ActivityFeedData {
  activeStatuses: ActivityFeedItem[];
  recentActivities: ActivityFeedItem[];
}

export interface InspirationEntry {
  id: number;
  title: string | null;
  content: string;
  imageDataUrl?: string | null;
  statusSnapshot?: string | null;
  createdAt: string;
  updatedAt?: string;
}

export interface InspirationEntryCreateInput {
  title: string;
  content: string;
  imageDataUrl?: string;
  generatedHashKey?: string;
}

export interface InspirationAssetUploadResult {
  publicKey: string;
  url: string;
}

export interface ApiError {
  status: number;
  message: string;
  details?: unknown;
}

export interface ApiResult<T> {
  success: boolean;
  status: number;
  data?: T;
  error?: ApiError;
}

export interface RecentPreset {
  process_name: string;
  process_title?: string;
  media_title?: string;
  media_singer?: string;
  lastUsedAt: string;
}

export interface ImportedIntegrationConfig {
  reportEndpoint?: string;
  token?: string;
  tokenName?: string;
  raw: Record<string, unknown>;
}

export interface AppStatePayload {
  config: ClientConfig;
  recentPresets: RecentPreset[];
  onboardingDismissed: boolean;
  reporterConfigPromptHandled?: boolean;
}

export type ReporterLogLevel = "info" | "success" | "warn" | "error";

export interface ReporterLogEntry {
  id: string;
  timestamp: string;
  level: ReporterLogLevel;
  title: string;
  detail: string;
  payload?: Record<string, unknown> | null;
}

export interface RealtimeReporterSnapshot {
  running: boolean;
  logs: ReporterLogEntry[];
  currentActivity?: ActivityFeedItem | null;
  lastHeartbeatAt?: string | null;
  lastError?: string | null;
  lastPendingApprovalMessage?: string | null;
  lastPendingApprovalUrl?: string | null;
}

export interface PlatformProbeResult {
  success: boolean;
  summary: string;
  detail: string;
  guidance?: string[];
}

export interface PlatformSelfTestResult {
  platform: string;
  foreground: PlatformProbeResult;
  windowTitle: PlatformProbeResult;
  media: PlatformProbeResult;
}

export interface ExistingReporterConfig {
  found: boolean;
  path?: string;
  config?: ClientConfig;
}
