"use client";

import React, { useState, useCallback, useEffect } from "react";
import {
  Bell,
  Mail,
  Send,
  Globe,
  CheckCircle,
  QrCode,
  Loader2,
  AlertTriangle,
  Shield,
} from "lucide-react";

interface NotificationChannel {
  id: "email" | "telegram" | "webpush";
  enabled: boolean;
  label: string;
  icon: React.ReactNode;
  description: string;
}

interface NotificationPreferencesProps {
  email?: string;
  telegramChatId?: string;
  onSave?: (channels: NotificationChannel[]) => Promise<void>;
}

const DEFAULT_CHANNELS: NotificationChannel[] = [
  {
    id: "email",
    enabled: true,
    label: "Email",
    icon: <Mail size={16} />,
    description: "Receive plan alerts and beneficiary notifications via email",
  },
  {
    id: "telegram",
    enabled: false,
    label: "Telegram Bot",
    icon: <Send size={16} />,
    description: "Real-time alerts through our Telegram notification bot",
  },
  {
    id: "webpush",
    enabled: false,
    label: "Web Push",
    icon: <Globe size={16} />,
    description: "Browser push notifications even when the tab is closed",
  },
];

export function NotificationPreferences({ email, telegramChatId, onSave }: NotificationPreferencesProps) {
  const [channels, setChannels] = useState<NotificationChannel[]>(DEFAULT_CHANNELS);
  const [saving, setSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<"idle" | "success" | "error">("idle");
  const [showTelegramPairing, setShowTelegramPairing] = useState(false);
  const [webPushPermission, setWebPushPermission] = useState<NotificationPermission | "unsupported">("default");

  useEffect(() => {
    if (typeof window !== "undefined" && "Notification" in window) {
      setWebPushPermission(Notification.permission);
    } else {
      setWebPushPermission("unsupported");
    }
  }, []);

  const toggleChannel = useCallback((id: NotificationChannel["id"]) => {
    setChannels((prev) =>
      prev.map((c) => (c.id === id ? { ...c, enabled: !c.enabled } : c))
    );
  }, []);

  const requestWebPush = useCallback(async () => {
    if (webPushPermission === "unsupported") return;
    try {
      const permission = await Notification.requestPermission();
      setWebPushPermission(permission);
      if (permission === "granted") {
        setChannels((prev) =>
          prev.map((c) => (c.id === "webpush" ? { ...c, enabled: true } : c))
        );
      }
    } catch {
      // permission denied
    }
  }, [webPushPermission]);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setSaveStatus("idle");
    try {
      await onSave?.(channels);
      setSaveStatus("success");
      setTimeout(() => setSaveStatus("idle"), 3000);
    } catch {
      setSaveStatus("error");
    } finally {
      setSaving(false);
    }
  }, [channels, onSave]);

  const enabledCount = channels.filter((c) => c.enabled).length;

  return (
    <div
      className="rounded-2xl bg-[#0A0F11] border border-white/[0.06] p-6"
      data-testid="notification-preferences"
    >
      {/* Header */}
      <div className="flex items-center gap-3 mb-5">
        <div className="w-10 h-10 rounded-xl bg-[#33C5E0]/10 border border-[#33C5E0]/20 flex items-center justify-center">
          <Bell size={18} className="text-[#33C5E0]" />
        </div>
        <div>
          <h3 className="text-base font-semibold text-white">Notification Preferences</h3>
          <p className="text-xs text-slate-500 mt-0.5">
            {enabledCount} channel{enabledCount !== 1 ? "s" : ""} active
          </p>
        </div>
      </div>

      {/* Save status */}
      {saveStatus === "success" && (
        <div className="mb-4 flex items-center gap-2 p-3 rounded-xl bg-emerald-400/10 border border-emerald-400/20 text-emerald-400 text-sm">
          <CheckCircle size={15} />
          Preferences saved
        </div>
      )}
      {saveStatus === "error" && (
        <div className="mb-4 flex items-center gap-2 p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
          <AlertTriangle size={15} />
          Failed to save preferences
        </div>
      )}

      {/* Channels */}
      <div className="space-y-3 mb-5">
        {channels.map((channel) => (
          <div
            key={channel.id}
            className={`p-4 rounded-xl border transition-colors ${
              channel.enabled
                ? "bg-[#33C5E0]/[0.04] border-[#33C5E0]/15"
                : "bg-white/[0.02] border-white/[0.05]"
            }`}
          >
            <div className="flex items-start justify-between gap-3">
              <div className="flex items-start gap-3 min-w-0">
                <div
                  className={`w-9 h-9 rounded-lg flex items-center justify-center flex-shrink-0 ${
                    channel.enabled ? "bg-[#33C5E0]/10 text-[#33C5E0]" : "bg-white/[0.04] text-slate-500"
                  }`}
                >
                  {channel.icon}
                </div>
                <div className="min-w-0">
                  <p className="text-sm font-medium text-white">{channel.label}</p>
                  <p className="text-[11px] text-slate-500 mt-0.5 leading-relaxed">
                    {channel.description}
                  </p>
                  {channel.id === "email" && email && (
                    <p className="text-[10px] text-slate-600 mt-1 font-mono truncate">{email}</p>
                  )}
                  {channel.id === "telegram" && telegramChatId && channel.enabled && (
                    <p className="text-[10px] text-emerald-400 mt-1">Paired: {telegramChatId}</p>
                  )}
                </div>
              </div>

              {/* Toggle */}
              <button
                role="switch"
                aria-checked={channel.enabled}
                aria-label={`Toggle ${channel.label} notifications`}
                onClick={() => {
                  toggleChannel(channel.id);
                  if (channel.id === "telegram" && !channel.enabled) {
                    setShowTelegramPairing(true);
                  }
                  if (channel.id === "webpush" && !channel.enabled) {
                    requestWebPush();
                  }
                }}
                className={`relative w-11 h-6 rounded-full transition-colors flex-shrink-0 ${
                  channel.enabled ? "bg-[#33C5E0]" : "bg-white/10"
                }`}
                data-testid={`toggle-${channel.id}`}
              >
                <span
                  className={`absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform ${
                    channel.enabled ? "translate-x-[22px]" : "translate-x-0.5"
                  }`}
                />
              </button>
            </div>

            {/* Telegram pairing QR */}
            {channel.id === "telegram" && showTelegramPairing && channel.enabled && (
              <div className="mt-3 p-3 rounded-lg bg-white/[0.03] border border-white/[0.05]">
                <p className="text-xs text-slate-400 mb-2">Pair with Telegram bot:</p>
                <div className="flex items-center gap-3">
                  <div className="w-20 h-20 bg-white rounded-lg flex items-center justify-center p-1.5">
                    <QrCode size={64} className="text-black" />
                  </div>
                  <div className="text-[11px] text-slate-500 space-y-1">
                    <p>1. Open Telegram and search <span className="text-[#33C5E0]">@InheritXBot</span></p>
                    <p>2. Send <span className="font-mono text-white">/pair</span></p>
                    <p>3. Scan the QR code above</p>
                  </div>
                </div>
                <button
                  onClick={() => setShowTelegramPairing(false)}
                  className="mt-2 text-[11px] text-slate-500 hover:text-white"
                >
                  Dismiss
                </button>
              </div>
            )}

            {/* Web Push permission state */}
            {channel.id === "webpush" && channel.enabled && (
              <div className="mt-3 p-2.5 rounded-lg bg-white/[0.03] border border-white/[0.05] flex items-start gap-2">
                <Shield size={12} className="text-slate-500 flex-shrink-0 mt-0.5" />
                <div className="text-[11px] text-slate-500">
                  {webPushPermission === "granted" ? (
                    <span className="text-emerald-400">Push notifications enabled</span>
                  ) : webPushPermission === "denied" ? (
                    <span className="text-red-400">
                      Blocked. Enable notifications in your browser settings.
                    </span>
                  ) : webPushPermission === "unsupported" ? (
                    <span>Not supported in this browser</span>
                  ) : (
                    <button
                      onClick={requestWebPush}
                      className="text-[#33C5E0] hover:underline"
                    >
                      Click to grant permission
                    </button>
                  )}
                </div>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Save button */}
      <button
        onClick={handleSave}
        disabled={saving}
        className="w-full py-2.5 rounded-xl bg-[#33C5E0] text-black text-sm font-semibold hover:bg-[#33C5E0]/90 transition-colors disabled:opacity-40 flex items-center justify-center gap-2"
        data-testid="save-notifications-btn"
      >
        {saving && <Loader2 size={14} className="animate-spin" />}
        {saving ? "Saving…" : "Save Preferences"}
      </button>
    </div>
  );
}
