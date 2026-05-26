import { computed, ref, type ComputedRef, type Ref } from "vue";

import { readDevicePlatform } from "@/lib/deviceInfo";
import type { ClientConfig, DeviceType } from "@/types";

interface UseAppShellViewportOptions {
  reporterSupported: ComputedRef<boolean>;
  config: Ref<ClientConfig>;
  onboardingDraftConfig: Ref<ClientConfig>;
}

export function useAppShellViewport(options: UseAppShellViewportOptions) {
  const viewportWidth = ref(1200);
  const isPhone = computed(() => viewportWidth.value < 900);
  const isAndroid = ref(false);

  // Check device platform on initialization
  readDevicePlatform().then((platform) => {
    isAndroid.value = platform === "android";
  });

  function inferMobileDeviceType(): DeviceType {
    return isPhone.value ? "mobile" : "tablet";
  }

  function syncDeviceTypeByViewport() {
    // On Android, always use inferred mobile device type even if reporter is supported
    const nextType = (options.reporterSupported.value && !isAndroid.value)
      ? "desktop"
      : inferMobileDeviceType();

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
