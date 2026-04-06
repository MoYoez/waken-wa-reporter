import { defineStore } from "pinia";

export type InspirationComposeTab = "edit" | "preview";

interface InspirationDraftPatch {
  title?: string;
  content?: string;
  contentLexical?: string;
  coverImageDataUrl?: string;
  composeTab?: InspirationComposeTab;
  statusSnapshotInput?: string;
  statusSnapshotDeviceName?: string;
  selectedActivityKey?: string;
  attachCurrentStatus?: boolean;
  attachStatusIncludeDeviceInfo?: boolean;
}

interface InspirationDraftState {
  title: string;
  content: string;
  contentLexical: string;
  coverImageDataUrl: string;
  composeTab: InspirationComposeTab;
  statusSnapshotInput: string;
  statusSnapshotDeviceName: string;
  selectedActivityKey: string;
  attachCurrentStatus: boolean;
  attachStatusIncludeDeviceInfo: boolean;
}

function createInitialState(): InspirationDraftState {
  return {
    title: "",
    content: "",
    contentLexical: "",
    coverImageDataUrl: "",
    composeTab: "edit",
    statusSnapshotInput: "",
    statusSnapshotDeviceName: "",
    selectedActivityKey: "",
    attachCurrentStatus: false,
    attachStatusIncludeDeviceInfo: false,
  };
}

export const useInspirationDraftStore = defineStore("inspirationDraft", {
  state: (): InspirationDraftState => createInitialState(),
  actions: {
    patchDraft(patch: InspirationDraftPatch) {
      if (typeof patch.title === "string") this.title = patch.title;
      if (typeof patch.content === "string") this.content = patch.content;
      if (typeof patch.contentLexical === "string") this.contentLexical = patch.contentLexical;
      if (typeof patch.coverImageDataUrl === "string") this.coverImageDataUrl = patch.coverImageDataUrl;
      if (patch.composeTab === "edit" || patch.composeTab === "preview") this.composeTab = patch.composeTab;
      if (typeof patch.statusSnapshotInput === "string") this.statusSnapshotInput = patch.statusSnapshotInput;
      if (typeof patch.statusSnapshotDeviceName === "string") this.statusSnapshotDeviceName = patch.statusSnapshotDeviceName;
      if (typeof patch.selectedActivityKey === "string") this.selectedActivityKey = patch.selectedActivityKey;
      if (typeof patch.attachCurrentStatus === "boolean") this.attachCurrentStatus = patch.attachCurrentStatus;
      if (typeof patch.attachStatusIncludeDeviceInfo === "boolean") this.attachStatusIncludeDeviceInfo = patch.attachStatusIncludeDeviceInfo;
    },
    resetDraft() {
      Object.assign(this, createInitialState());
    },
  },
});
