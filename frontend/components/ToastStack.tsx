"use client";

import { useAppStore } from "@/store";

const TONE: Record<string, string> = {
  success: "border-emerald-500/30 bg-emerald-500/10 text-emerald-300",
  info: "border-blue-500/30 bg-blue-500/10 text-blue-300",
  warning: "border-yellow-500/30 bg-yellow-500/10 text-yellow-300",
  error: "border-red-500/30 bg-red-500/10 text-red-300",
};

export default function ToastStack() {
  const toasts = useAppStore((s) => s.toasts);
  const dismissToast = useAppStore((s) => s.dismissToast);
  if (toasts.length === 0) return null;
  return (
    <div
      className="fixed bottom-4 right-4 z-50 flex w-80 flex-col gap-2"
      role="status"
      aria-live="polite"
    >
      {toasts.map((t) => (
        <div
          key={t.id}
          onClick={() => dismissToast(t.id)}
          className={`cursor-pointer rounded-xl border p-3 backdrop-blur ${TONE[t.kind]}`}
        >
          <p className="text-sm font-semibold capitalize">{t.title}</p>
          {t.message && <p className="mt-0.5 text-xs opacity-80">{t.message}</p>}
        </div>
      ))}
    </div>
  );
}
