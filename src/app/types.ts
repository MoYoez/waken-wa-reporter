export type AppSection = "overview" | "settings" | "activity" | "realtime" | "inspiration";

export interface SectionNavItem {
  key: AppSection;
  title: string;
  kicker: string;
  icon: string;
  requiresRealtime?: boolean;
}

export interface SingleInstanceAttemptPayload {
  args: string[];
  cwd: string;
}
