import { computed, ref, type ComputedRef } from "vue";

import type { AppSection, SectionNavItem } from "@/app/types";

interface UseAppShellNavigationOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  reporterSupported: ComputedRef<boolean>;
}

export function useAppShellNavigation(options: UseAppShellNavigationOptions) {
  const activeSection = ref<AppSection>("overview");

  const sections = computed<SectionNavItem[]>(() => [
    {
      key: "overview",
      title: options.t("app.sections.overview.title"),
      kicker: options.t("app.sections.overview.kicker"),
      icon: "pi pi-home",
    },
    {
      key: "inspiration",
      title: options.t("app.sections.inspiration.title"),
      kicker: options.t("app.sections.inspiration.kicker"),
      icon: "pi pi-file-edit",
    },
    {
      key: "activity",
      title: options.t("app.sections.activity.title"),
      kicker: options.t("app.sections.activity.kicker"),
      icon: "pi pi-pencil",
    },
    {
      key: "realtime",
      title: options.t("app.sections.realtime.title"),
      kicker: options.t("app.sections.realtime.kicker"),
      icon: "pi pi-chart-line",
      requiresRealtime: true,
    },
    {
      key: "settings",
      title: options.t("app.sections.settings.title"),
      kicker: options.t("app.sections.settings.kicker"),
      icon: "pi pi-cog",
    },
  ]);

  const visibleSections = computed(() =>
    sections.value.filter((section) => !section.requiresRealtime || options.reporterSupported.value),
  );

  function ensureVisibleSection() {
    if (!visibleSections.value.some((section) => section.key === activeSection.value)) {
      activeSection.value = visibleSections.value[0]?.key ?? "overview";
    }
  }

  function selectSection(section: AppSection) {
    activeSection.value = section;
  }

  return {
    activeSection,
    ensureVisibleSection,
    selectSection,
    visibleSections,
  };
}
