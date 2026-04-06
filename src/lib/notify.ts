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
            ? "[成功]"
            : payload.severity === "error"
              ? "[错误]"
              : payload.severity === "warn"
                ? "[提示]"
                : "[信息]";
        window.alert(`${prefix} ${payload.summary}\n${payload.detail}`);
        return;
      }

      toast.add(payload);
    },
  };
}
