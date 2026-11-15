import { translate } from "../../i18n";
import { resolveApiErrorMessage } from "../localizedText";
import type { ApiResult } from "../../types";
import { invokeApi } from "./invoke";

function unwrap<T>(result: ApiResult<T>, fallbackKey: string): T {
  if (result.success && result.data !== undefined) {
    return result.data;
  }
  const message = resolveApiErrorMessage(result.error, translate, translate(fallbackKey));
  throw new Error(message);
}

export async function isAutostartEnabled(): Promise<boolean> {
  const result = await invokeApi<boolean>("is_autostart_enabled");
  return unwrap(result, "backendErrors.autostartQueryFailed");
}

export async function setAutostartEnabled(enabled: boolean): Promise<void> {
  const result = await invokeApi<boolean>("set_autostart_enabled", { enabled });
  unwrap(
    result,
    enabled ? "backendErrors.autostartEnableFailed" : "backendErrors.autostartDisableFailed",
  );
}
