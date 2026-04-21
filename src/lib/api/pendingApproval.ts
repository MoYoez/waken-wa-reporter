import { translate } from "../../i18n";
import { resolveApiErrorMessage } from "../localizedText";
import type { ApiResult, PendingApprovalInfo } from "../../types";

function pendingApprovalPayload(
  candidate: unknown,
): { pending: boolean; message: string; approvalUrl: string } | null {
  if (!candidate || typeof candidate !== "object" || Array.isArray(candidate)) {
    return null;
  }

  const payload = candidate as Record<string, unknown>;
  if (payload.pending !== true) {
    return null;
  }

  return {
    pending: true,
    message:
      typeof payload.error === "string"
        ? payload.error
        : typeof payload.message === "string"
          ? payload.message
          : "",
    approvalUrl: typeof payload.approvalUrl === "string" ? payload.approvalUrl : "",
  };
}

export function extractPendingApprovalInfo(
  result: ApiResult<unknown>,
): PendingApprovalInfo | null {
  if (result.status !== 202) {
    return null;
  }

  const payload =
    pendingApprovalPayload(result.error?.details)
    ?? pendingApprovalPayload(result.data);

  if (!payload) {
    return null;
  }

  return {
    message:
      payload.message
      || resolveApiErrorMessage(
        result.error,
        translate,
        translate("errors.pendingApprovalDefault"),
      )
      || translate("errors.pendingApprovalDefault"),
    approvalUrl: payload.approvalUrl || null,
  };
}

export function formatPendingApprovalDetail(info: PendingApprovalInfo) {
  return info.approvalUrl
    ? translate("errors.pendingApprovalWithUrl", { approvalUrl: info.approvalUrl })
    : translate("errors.pendingApprovalWithoutUrl");
}
