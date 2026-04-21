import { invoke } from "@tauri-apps/api/core";

import { translate } from "../../i18n";
import type { ApiResult } from "../../types";

function toInvokeError(message: string, details?: unknown): ApiResult<never> {
  return {
    success: false,
    status: 0,
    error: {
      status: 0,
      message,
      details,
    },
  };
}

export async function invokeApi<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<ApiResult<T>> {
  try {
    return await invoke<ApiResult<T>>(command, args);
  } catch (error) {
    return toInvokeError(
      error instanceof Error ? error.message : translate("errors.tauriInvokeFailed"),
      error,
    );
  }
}
