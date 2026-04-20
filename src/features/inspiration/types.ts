import type { ActivityFeedItem, InspirationEntry } from "@/types";

export type InspirationComposeTab = "edit" | "preview";

export interface ActivitySelectOption {
  value: string;
  label: string;
  snapshot: string;
  group: "active" | "recent";
  item: ActivityFeedItem;
  deviceName: string;
}

export interface InspirationEntryCardView {
  entry: InspirationEntry;
  createdAtLabel: string;
  previewImageUrl: string;
  previewText: string;
}
