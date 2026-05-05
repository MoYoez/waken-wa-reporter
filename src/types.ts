export type DeviceType = "desktop" | "tablet" | "mobile";

export type PushMode = "realtime" | "active";

export interface ClientCapabilities {
  realtimeReporter: boolean;
  tray: boolean;
  platformSelfTest: boolean;
  discordPresence: boolean;
  autostart: boolean;
}

export interface ClientConfig {
  baseUrl: string;
  apiToken: string;
  generatedHashKey: string;
  useSystemProxy: boolean;
  device: string;
  deviceType: DeviceType;
  pushMode: PushMode;
  pollIntervalMs: number;
  heartbeatIntervalMs: number;
  reporterMetadataJson: string;
  reporterEnabled: boolean;
  reportForegroundApp: boolean;
  reportWindowTitle: boolean;
  reportMedia: boolean;
  reportPlaySource: boolean;
  reportMediaArtwork: boolean;
  discordEnabled: boolean;
  discordApplicationId: string;
  discordSourceId: string;
  launchOnStartup: boolean;
}

export interface ActivityMedia {
  title: string;
  singer?: string;
  artist?: string;
  album?: string;
  coverDataUrl?: string;
  coverUrl?: string;
  status?: "playing" | "paused" | "stopped";
  isPlaying?: boolean;
  isPaused?: boolean;
  positionMs?: number;
  durationMs?: number;
  timestamps?: {
    start?: number | string;
    end?: number | string;
  };
  reportedAt?: number | string;
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
  contentLexical?: string | null;
  imageDataUrl?: string | null;
  statusSnapshot?: string | null;
  createdAt: string;
  updatedAt?: string;
}

export interface PaginationMeta {
  limit: number;
  offset: number;
  total: number;
}

export interface InspirationEntryListResponse {
  data: InspirationEntry[];
  pagination?: PaginationMeta;
}

export interface InspirationEntryCreateInput {
  title: string;
  content: string;
  contentLexical?: string;
  imageDataUrl?: string;
  generatedHashKey?: string;
  attachCurrentStatus?: boolean;
  preComputedStatusSnapshot?: string;
  attachStatusDeviceHash?: string;
  attachStatusActivityKey?: string;
  attachStatusIncludeDeviceInfo?: boolean;
}

export interface InspirationAssetUploadResult {
  publicKey: string;
  url: string;
}

export interface ApiError {
  status: number;
  message: string;
  code?: string | null;
  params?: Record<string, unknown> | null;
  details?: unknown;
}

export interface ApiResult<T> {
  success: boolean;
  status: number;
  data?: T;
  error?: ApiError;
}

export interface PendingApprovalInfo {
  message: string;
  approvalUrl?: string | null;
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
  deviceName?: string;
  raw: Record<string, unknown>;
}

export interface AppStatePayload {
  config: ClientConfig;
  recentPresets: RecentPreset[];
  onboardingDismissed: boolean;
  locale?: string;
  reporterConfigPromptHandled?: boolean;
  verifiedGeneratedHashKey?: string;
}

export type ReporterLogLevel = "info" | "success" | "warn" | "error";

export interface ReporterLogEntry {
  id: string;
  timestamp: string;
  level: ReporterLogLevel;
  title: string;
  detail: string;
  titleKey?: string | null;
  titleParams?: Record<string, unknown> | null;
  detailKey?: string | null;
  detailParams?: Record<string, unknown> | null;
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

export interface DiscordPresenceSnapshot {
  running: boolean;
  connected: boolean;
  lastSyncAt?: string | null;
  lastError?: string | null;
  currentSummary?: string | null;
}

export interface LocalizedTextEntry {
  text: string;
  key?: string | null;
  params?: Record<string, unknown> | null;
}

export interface PlatformProbeResult {
  success: boolean;
  summary: string;
  detail: string;
  guidance?: string[];
  summaryKey?: string | null;
  summaryParams?: Record<string, unknown> | null;
  detailKey?: string | null;
  detailParams?: Record<string, unknown> | null;
  guidanceEntries?: LocalizedTextEntry[] | null;
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

export interface MobileConnectivityState {
  checking: boolean;
  checked: boolean;
  ok: boolean | null;
  summary: string;
  detail: string;
  checkedAt?: string | null;
}
