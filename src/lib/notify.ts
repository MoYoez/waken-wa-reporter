import { translate } from "../i18n";

interface ToastLike {
  add: (payload: {
    severity: "success" | "info" | "warn" | "error";
    summary: string;
    detail: string;
    life?: number;
  }) => void;
}

export interface NotifyPayload {
  severity: "success" | "info" | "warn" | "error";
  summary: string;
  detail: string;
  life?: number;
}

export function createNotifier(toast: ToastLike, useNativeNotice: () => boolean) {
  return {
    notify(payload: NotifyPayload) {
      if (useNativeNotice()) {
        const prefix =
          payload.severity === "success"
            ? translate("notify.successPrefix")
            : payload.severity === "error"
              ? translate("notify.errorPrefix")
              : payload.severity === "warn"
                ? translate("notify.warnPrefix")
                : translate("notify.infoPrefix");
        window.alert(`${prefix} ${payload.summary}\n${payload.detail}`);
        return;
      }

      toast.add(payload);
    },
  };
}
