import type { ComputedRef, Ref } from "vue";

import {
  extractPendingApprovalInfo,
  formatPendingApprovalDetail,
  probeConnectivity,
} from "@/lib/api";
import type { ClientConfig, MobileConnectivityState, PendingApprovalInfo } from "@/types";

interface UseAppShellMobileConnectivityOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  apiErrorDetail: (
    error: { message?: string; code?: string | null; params?: Record<string, unknown> | null } | null | undefined,
    fallback: string,
  ) => string;
  config: Ref<ClientConfig>;
  mobileConnectivity: Ref<MobileConnectivityState>;
  lastMobileConnectivitySignature: Ref<string>;
  reporterSupported: ComputedRef<boolean>;
  readiness: ComputedRef<boolean>;
  rememberVerifiedGeneratedHashKey: (value: string) => void;
  onPendingApproval: (info: PendingApprovalInfo) => void;
}

export function useAppShellMobileConnectivity(options: UseAppShellMobileConnectivityOptions) {
  function resetMobileConnectivity(summary = options.t("app.mobileConnectivity.pending"), detail = "") {
    options.mobileConnectivity.value = {
      checking: false,
      checked: false,
      ok: null,
      summary,
      detail,
      checkedAt: null,
    };
  }

  async function runMobileConnectivityProbe(force = false) {
    if (options.reporterSupported.value) {
      resetMobileConnectivity();
      return;
    }

    if (!options.readiness.value) {
      options.lastMobileConnectivitySignature.value = "";
      resetMobileConnectivity(
        options.t("app.mobileConnectivity.waitingConfig"),
        options.t("app.mobileConnectivity.waitingConfigDetail"),
      );
      return;
    }

    const signature = [
      options.config.value.baseUrl.trim(),
      options.config.value.apiToken.trim(),
      options.config.value.generatedHashKey.trim(),
    ].join("|");

    if (
      !force
      && options.mobileConnectivity.value.checked
      && options.lastMobileConnectivitySignature.value === signature
    ) {
      return;
    }

    options.lastMobileConnectivitySignature.value = signature;
    options.mobileConnectivity.value = {
      checking: true,
      checked: false,
      ok: null,
      summary: options.t("app.mobileConnectivity.checking"),
      detail: options.t("app.mobileConnectivity.checkingDetail"),
      checkedAt: null,
    };

    const result = await probeConnectivity(options.config.value);
    const checkedAt = new Date().toISOString();
    const pendingApproval = extractPendingApprovalInfo(result);

    if (pendingApproval) {
      options.mobileConnectivity.value = {
        checking: false,
        checked: true,
        ok: false,
        summary: options.t("app.mobileConnectivity.pendingApproval"),
        detail: formatPendingApprovalDetail(pendingApproval),
        checkedAt,
      };
      options.onPendingApproval(pendingApproval);
      return;
    }

    if (result.success) {
      options.rememberVerifiedGeneratedHashKey(options.config.value.generatedHashKey.trim());
      options.mobileConnectivity.value = {
        checking: false,
        checked: true,
        ok: true,
        summary: options.t("app.mobileConnectivity.passed"),
        detail: options.t("app.mobileConnectivity.passedDetail"),
        checkedAt,
      };
      return;
    }

    let summary = options.t("app.mobileConnectivity.failed");
    let detail = options.apiErrorDetail(result.error, options.t("app.mobileConnectivity.failedDetail"));

    if (result.status === 401) {
      summary = options.t("app.mobileConnectivity.tokenUnavailable");
      detail = options.t("app.mobileConnectivity.tokenUnavailableDetail");
    } else if (result.status === 403) {
      summary = options.t("app.mobileConnectivity.deviceUnavailable");
      detail = options.apiErrorDetail(result.error, options.t("app.mobileConnectivity.deviceUnavailableDetail"));
    } else if (result.status === 400) {
      summary = options.t("app.mobileConnectivity.configIncomplete");
      detail = options.apiErrorDetail(result.error, options.t("app.mobileConnectivity.configIncompleteDetail"));
    } else if (result.status === 0) {
      summary = options.t("app.mobileConnectivity.siteUnreachable");
      detail = options.apiErrorDetail(result.error, options.t("app.mobileConnectivity.siteUnreachableDetail"));
    }

    options.mobileConnectivity.value = {
      checking: false,
      checked: true,
      ok: false,
      summary,
      detail,
      checkedAt,
    };
  }

  return {
    runMobileConnectivityProbe,
  };
}
