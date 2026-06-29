"use client";

import { useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Clock, AlertCircle, CheckCircle, Loader2, RefreshCw } from "lucide-react";
import { useInactivityTimer, type InactivityTimerState } from "@/app/hooks/useInactivityTimer";
import { useWallet } from "@/context/WalletContext";

// ─── Types ────────────────────────────────────────────────────────────────────

interface InactivityTimerCardProps {
  planId: string;
  onPingSuccess?: () => void;
  onPingError?: (error: Error) => void;
}

type PingStatus = "idle" | "signing" | "pinging" | "success" | "error";

// ─── Helpers ──────────────────────────────────────────────────────────────────

function formatTimerDisplay(timer: InactivityTimerState): string {
  if (timer.isClaimable) {
    return "Plan is claimable";
  }
  return `${timer.days}d ${timer.hours}h ${timer.minutes}m`;
}

function getTimerStatusColor(timer: InactivityTimerState): {
  bg: string;
  border: string;
  text: string;
  badge: string;
} {
  if (timer.isClaimable) {
    return {
      bg: "bg-[#F5656514]",
      border: "border-[#F5656540]",
      text: "text-[#F56565]",
      badge: "bg-[#F56565] text-white",
    };
  }
  if (timer.isSoonWarning) {
    return {
      bg: "bg-[#ED8936]14",
      border: "border-[#ED8936]40",
      text: "text-[#ED8936]",
      badge: "bg-[#ED8936] text-white",
    };
  }
  return {
    bg: "bg-[#48BB7814]",
    border: "border-[#48BB7840]",
    text: "text-[#48BB78]",
    badge: "bg-[#48BB78] text-white",
  };
}

// ─── Main Component ───────────────────────────────────────────────────────────

export function InactivityTimerCard({
  planId,
  onPingSuccess,
  onPingError,
}: InactivityTimerCardProps) {
  const { kit, selectedWalletId } = useWallet();
  const { timerState, loading, error, ping } = useInactivityTimer({
    planId,
    enabled: true,
    pollIntervalMs: 30000, // Poll every 30 seconds
    warningThresholdHours: 24,
  });

  const [pingStatus, setPingStatus] = useState<PingStatus>("idle");
  const [pingError, setPingError] = useState<string>("");

  const buildKeepAliveXdr = useCallback((): string => {
    // Build unsigned XDR for keep-alive transaction
    // In production, this would call the Soroban contract SDK
    return `unsigned-xdr::keep-alive::${planId}::${Date.now()}`;
  }, [planId]);

  const signWithWallet = useCallback(
    async (xdr: string): Promise<string> => {
      if (!kit || !selectedWalletId) {
        throw new Error("No wallet connected. Please connect your wallet first.");
      }
      const result = await kit.signTransaction(xdr);
      return result.signedTxXdr;
    },
    [kit, selectedWalletId]
  );

  const handlePing = useCallback(async () => {
    setPingStatus("signing");
    setPingError("");

    try {
      let signedTransaction: string | undefined;

      // Try to sign with wallet
      if (kit && selectedWalletId) {
        const xdr = buildKeepAliveXdr();
        signedTransaction = await signWithWallet(xdr);
      }

      setPingStatus("pinging");
      await ping(signedTransaction);

      setPingStatus("success");
      onPingSuccess?.();

      // Reset to idle after delay
      setTimeout(() => setPingStatus("idle"), 2000);
    } catch (err) {
      const errorMsg =
        err instanceof Error ? err.message : "Failed to ping keep-alive";
      setPingError(errorMsg);
      setPingStatus("error");
      onPingError?.(err instanceof Error ? err : new Error(errorMsg));

      // Reset to idle after delay
      setTimeout(() => {
        setPingStatus("idle");
        setPingError("");
      }, 3000);
    }
  }, [kit, selectedWalletId, buildKeepAliveXdr, signWithWallet, ping, onPingSuccess, onPingError]);

  const statusColors = getTimerStatusColor(timerState);

  if (loading && timerState.lastPingTimestamp === Date.now()) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        className={`rounded-xl border p-4 ${statusColors.bg} ${statusColors.border}`}
      >
        <div className="flex items-center justify-center gap-2">
          <Loader2 size={16} className="animate-spin text-[#92A5A8]" />
          <p className="text-sm text-[#92A5A8]">Loading inactivity timer...</p>
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      className={`rounded-xl border overflow-hidden transition-all ${statusColors.bg} ${statusColors.border}`}
    >
      {/* Header */}
      <div className="p-4 border-b border-inherit flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className={`p-2 rounded-lg bg-white/10`}>
            <Clock size={18} className={statusColors.text} />
          </div>
          <div>
            <p className="text-xs font-semibold text-[#92A5A8] uppercase tracking-wider">
              Inactivity Timer
            </p>
            <p className={`text-sm font-mono font-semibold mt-0.5 ${statusColors.text}`}>
              {formatTimerDisplay(timerState)}
            </p>
          </div>
        </div>

        <AnimatePresence mode="wait">
          {timerState.isClaimable ? (
            <motion.div
              key="claimable"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              className="flex items-center gap-1 px-3 py-1 rounded-full bg-[#F56565] text-white text-xs font-semibold"
            >
              <AlertCircle size={12} />
              CLAIMABLE
            </motion.div>
          ) : timerState.isSoonWarning ? (
            <motion.div
              key="warning"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              className="flex items-center gap-1 px-3 py-1 rounded-full bg-[#ED8936] text-white text-xs font-semibold"
            >
              <AlertCircle size={12} />
              LOW TIME
            </motion.div>
          ) : (
            <motion.div
              key="active"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              className="flex items-center gap-1 px-3 py-1 rounded-full bg-[#48BB78] text-white text-xs font-semibold"
            >
              <CheckCircle size={12} />
              ACTIVE
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Body */}
      <div className="p-4 space-y-3">
        {/* Description */}
        <p className="text-xs text-[#92A5A8] leading-relaxed">
          {timerState.isClaimable
            ? "This plan has entered the claimable state. Beneficiaries can now claim their allocated assets."
            : timerState.isSoonWarning
              ? "⚠️ Plan will become claimable soon. Send a keep-alive ping to reset the timer."
              : "Your plan is active. Send a ping periodically to keep the inheritance plan active and prevent early claiming."}
        </p>

        {/* Timer Details */}
        <div className="grid grid-cols-4 gap-2 text-center text-xs">
          <div className="bg-black/20 rounded-lg p-2">
            <p className="text-[#92A5A8] text-[10px] uppercase tracking-wider mb-1">
              Days
            </p>
            <p className={`text-sm font-bold ${statusColors.text}`}>
              {timerState.days}
            </p>
          </div>
          <div className="bg-black/20 rounded-lg p-2">
            <p className="text-[#92A5A8] text-[10px] uppercase tracking-wider mb-1">
              Hours
            </p>
            <p className={`text-sm font-bold ${statusColors.text}`}>
              {timerState.hours}
            </p>
          </div>
          <div className="bg-black/20 rounded-lg p-2">
            <p className="text-[#92A5A8] text-[10px] uppercase tracking-wider mb-1">
              Minutes
            </p>
            <p className={`text-sm font-bold ${statusColors.text}`}>
              {timerState.minutes}
            </p>
          </div>
          <div className="bg-black/20 rounded-lg p-2">
            <p className="text-[#92A5A8] text-[10px] uppercase tracking-wider mb-1">
              Seconds
            </p>
            <p className={`text-sm font-bold ${statusColors.text}`}>
              {timerState.seconds}
            </p>
          </div>
        </div>

        {/* Error message */}
        <AnimatePresence>
          {(error || pingError) && (
            <motion.div
              initial={{ opacity: 0, y: -4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              className="flex items-start gap-2 p-3 rounded-lg bg-[#F5656514] border border-[#F5656540] text-[#F56565] text-xs"
            >
              <AlertCircle size={14} className="mt-0.5 flex-shrink-0" />
              <span>{pingError || error?.message}</span>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Success message */}
        <AnimatePresence>
          {pingStatus === "success" && (
            <motion.div
              initial={{ opacity: 0, y: -4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              className="flex items-center gap-2 p-3 rounded-lg bg-[#48BB7814] border border-[#48BB7840] text-[#48BB78] text-xs"
            >
              <CheckCircle size={14} className="flex-shrink-0" />
              <span>Keep-alive ping successful! Timer reset.</span>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Footer - Action Button */}
      <div className="px-4 py-3 bg-black/20 border-t border-inherit flex gap-2">
        <button
          type="button"
          onClick={handlePing}
          disabled={
            !selectedWalletId ||
            timerState.isClaimable ||
            pingStatus === "signing" ||
            pingStatus === "pinging" ||
            pingStatus === "success"
          }
          className="flex-1 flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed bg-[#33C5E0] text-black hover:bg-cyan-300"
        >
          {pingStatus === "signing" && (
            <>
              <Loader2 size={14} className="animate-spin" />
              Signing…
            </>
          )}
          {pingStatus === "pinging" && (
            <>
              <Loader2 size={14} className="animate-spin" />
              Pinging…
            </>
          )}
          {pingStatus === "success" && (
            <>
              <CheckCircle size={14} />
              Pinged!
            </>
          )}
          {(pingStatus === "idle" || pingStatus === "error") && (
            <>
              <RefreshCw size={14} />
              Ping (Verify Alive)
            </>
          )}
        </button>

        {!selectedWalletId && (
          <p className="text-[10px] text-[#92A5A8] pt-2.5">
            Connect wallet to ping
          </p>
        )}
      </div>
    </motion.div>
  );
}

export default InactivityTimerCard;
