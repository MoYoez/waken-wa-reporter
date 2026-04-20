import { computed, ref, type ComputedRef, type Ref } from "vue";

import type { ClientConfig, DeviceType } from "@/types";

interface UseAppShellViewportOptions {
  reporterSupported: ComputedRef<boolean>;
  config: Ref<ClientConfig>;
  onboardingDraftConfig: Ref<ClientConfig>;
}

export function useAppShellViewport(options: UseAppShellViewportOptions) {
  const viewportWidth = ref(1200);
  const isPhone = computed(() => viewportWidth.value < 900);

  function inferMobileDeviceType(): DeviceType {
    return isPhone.value ? "mobile" : "tablet";
  }

  function syncDeviceTypeByViewport() {
    const nextType = options.reporterSupported.value ? "desktop" : inferMobileDeviceType();

    if (options.config.value.deviceType !== nextType) {
      options.config.value = { ...options.config.value, deviceType: nextType };
    }

    if (options.onboardingDraftConfig.value.deviceType !== nextType) {
      options.onboardingDraftConfig.value = {
        ...options.onboardingDraftConfig.value,
        deviceType: nextType,
      };
    }
  }

  function syncViewportWidth(width: number) {
    viewportWidth.value = width;
    syncDeviceTypeByViewport();
  }

  return {
    inferMobileDeviceType,
    isPhone,
    syncDeviceTypeByViewport,
    syncViewportWidth,
  };
}
