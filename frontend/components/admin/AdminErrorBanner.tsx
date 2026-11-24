"use client";
import { AlertCircle } from "lucide-react";

interface AdminErrorBannerProps {
  message: string;
  /** Shown as a "Retry" button when provided (use for failed loads). */
  onRetry?: () => void;
  /** Shown as a "Dismiss" button when provided (use for failed actions). */
  onDismiss?: () => void;
}

export function AdminErrorBanner({ message, onRetry, onDismiss }: AdminErrorBannerProps) {
  return (
    <div
      role="alert"
      className="rounded-xl border border-red-500/20 bg-red-500/5 px-4 py-3 flex items-start gap-2"
    >
      <AlertCircle size={14} className="text-red-400 mt-0.5 flex-shrink-0" />
      <p className="text-xs text-red-300 flex-1">{message}</p>
      {onRetry && (
        <button
          onClick={onRetry}
          className="text-xs font-medium text-red-300 underline hover:text-red-200"
        >
          Retry
        </button>
      )}
      {onDismiss && (
        <button
          onClick={onDismiss}
          className="text-xs font-medium text-red-300 underline hover:text-red-200"
        >
          Dismiss
        </button>
      )}
    </div>
  );
}
