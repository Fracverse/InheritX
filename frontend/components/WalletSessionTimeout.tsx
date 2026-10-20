"use client";

import React, { useState, useEffect, useCallback, useRef } from "react";
import { Clock, AlertTriangle, LogOut, RefreshCw } from "lucide-react";

const INACTIVITY_LIMIT_MS = 30 * 60 * 1000; // 30 minutes
const WARNING_BEFORE_MS = 60 * 1000; // 60 seconds warning
const CHECK_INTERVAL_MS = 1000; // check every second

interface WalletSessionTimeoutProps {
  onDisconnect: () => void;
  onExtendSession?: () => void;
  isEnabled?: boolean;
}

export function WalletSessionTimeout({
  onDisconnect,
  onExtendSession,
  isEnabled = true,
}: WalletSessionTimeoutProps) {
  const [showWarning, setShowWarning] = useState(false);
  const [remainingSeconds, setRemainingSeconds] = useState(60);
  const lastActivityRef = useRef<number>(Date.now());
  const warningShownRef = useRef(false);

  const resetActivity = useCallback(() => {
    lastActivityRef.current = Date.now();
    warningShownRef.current = false;
    setShowWarning(false);
  }, []);

  const handleExtend = useCallback(() => {
    resetActivity();
    onExtendSession?.();
  }, [resetActivity, onExtendSession]);

  useEffect(() => {
    if (!isEnabled) return;

    const events = ["mousedown", "keydown", "touchstart", "scroll", "wheel"];
    const handleActivity = () => resetActivity();
    events.forEach((e) => window.addEventListener(e, handleActivity, { passive: true }));

    const interval = setInterval(() => {
      const elapsed = Date.now() - lastActivityRef.current;
      const remaining = INACTIVITY_LIMIT_MS - elapsed;

      if (remaining <= 0) {
        clearInterval(interval);
        onDisconnect();
        return;
      }

      if (remaining <= WARNING_BEFORE_MS) {
        if (!warningShownRef.current) {
          warningShownRef.current = true;
          setShowWarning(true);
        }
        setRemainingSeconds(Math.ceil(remaining / 1000));
      }
    }, CHECK_INTERVAL_MS);

    return () => {
      events.forEach((e) => window.removeEventListener(e, handleActivity));
      clearInterval(interval);
    };
  }, [isEnabled, onDisconnect, resetActivity]);

  if (!showWarning) return null;

  return (
    <>
      {/* Backdrop */}
      <div className="fixed inset-0 z-[55] bg-black/60 backdrop-blur-sm" aria-hidden="true" />

      {/* Warning modal */}
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="session-timeout-title"
        aria-describedby="session-timeout-desc"
        className="fixed inset-0 z-[56] flex items-center justify-center p-4"
        data-testid="session-timeout-modal"
      >
        <div className="bg-[#0A0F11] border border-yellow-400/30 rounded-2xl p-6 max-w-sm w-full shadow-2xl">
          {/* Header */}
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-yellow-400/10 border border-yellow-400/20 flex items-center justify-center">
              <Clock size={18} className="text-yellow-400" />
            </div>
            <div>
              <h3 id="session-timeout-title" className="text-base font-semibold text-white">
                Session Expiring
              </h3>
              <p className="text-xs text-slate-500 mt-0.5">Auto-disconnect in progress</p>
            </div>
          </div>

          {/* Countdown */}
          <div className="text-center mb-5">
            <div className="relative inline-flex items-center justify-center">
              <svg width="80" height="80" className="-rotate-90">
                <circle
                  cx="40"
                  cy="40"
                  r="35"
                  fill="none"
                  stroke="#1C252A"
                  strokeWidth="4"
                />
                <circle
                  cx="40"
                  cy="40"
                  r="35"
                  fill="none"
                  stroke={remainingSeconds <= 10 ? "#EF4444" : "#EAB308"}
                  strokeWidth="4"
                  strokeLinecap="round"
                  strokeDasharray={`${2 * Math.PI * 35}`}
                  strokeDashoffset={`${2 * Math.PI * 35 * (1 - remainingSeconds / 60)}`}
                  className="transition-all duration-1000"
                />
              </svg>
              <span
                className={`absolute text-xl font-bold ${
                  remainingSeconds <= 10 ? "text-red-400" : "text-yellow-400"
                }`}
                aria-live="polite"
              >
                {remainingSeconds}s
              </span>
            </div>
          </div>

          <p id="session-timeout-desc" className="text-sm text-slate-400 text-center mb-5">
            Your wallet session will auto-disconnect in{" "}
            <span className="font-medium text-white">{remainingSeconds} second{remainingSeconds !== 1 ? "s" : ""}</span>{" "}
            due to inactivity.
          </p>

          {/* Actions */}
          <div className="space-y-2">
            <button
              onClick={handleExtend}
              className="w-full py-2.5 rounded-xl bg-[#33C5E0] text-black text-sm font-semibold hover:bg-[#33C5E0]/90 transition-colors flex items-center justify-center gap-2"
              data-testid="extend-session-btn"
            >
              <RefreshCw size={14} />
              Extend Session
            </button>
            <button
              onClick={() => {
                setShowWarning(false);
                onDisconnect();
              }}
              className="w-full py-2.5 rounded-xl bg-white/5 text-slate-300 border border-white/10 text-sm font-medium hover:bg-white/10 transition-colors flex items-center justify-center gap-2"
            >
              <LogOut size={14} />
              Disconnect Now
            </button>
          </div>

          {/* Security note */}
          <div className="flex items-start gap-2 mt-4 p-2.5 rounded-lg bg-white/[0.02] border border-white/[0.04]">
            <AlertTriangle size={11} className="text-slate-500 flex-shrink-0 mt-0.5" />
            <p className="text-[10px] text-slate-500 leading-relaxed">
              For security, sessions auto-disconnect after 30 minutes of inactivity.
              This protects your wallet on shared or public devices.
            </p>
          </div>
        </div>
      </div>
    </>
  );
}
