import type { Ref } from "vue";

import type { RecentPreset } from "@/types";

interface UseAppShellRecentPresetsOptions {
  persistAppState: () => Promise<void> | void;
  recentPresets: Ref<RecentPreset[]>;
}

export function useAppShellRecentPresets(options: UseAppShellRecentPresetsOptions) {
  function handlePresetSaved(preset: RecentPreset) {
    const deduped = options.recentPresets.value.filter(
      (item) =>
        item.process_name !== preset.process_name
        || item.process_title !== preset.process_title,
    );

    options.recentPresets.value = [preset, ...deduped].slice(0, 6);
    void options.persistAppState();
  }

  return {
    handlePresetSaved,
  };
}
