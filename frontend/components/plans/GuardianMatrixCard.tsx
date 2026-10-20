"use client";

import React, { useState } from "react";
import {
  Shield,
  ShieldCheck,
  ShieldAlert,
  Users,
  CheckCircle,
  XCircle,
  Clock,
  AlertTriangle,
  Loader2,
} from "lucide-react";

export interface Guardian {
  address: string;
  name?: string;
  hasSigned: boolean;
  signedAt?: number;
}

export interface GuardianMatrixProps {
  guardians: Guardian[];
  quorum: number;
  isTriggered: boolean;
  isOwner: boolean;
  onInitiateTrigger?: () => Promise<void>;
  isLoading?: boolean;
}

function truncateAddress(addr: string): string {
  return `${addr.slice(0, 6)}…${addr.slice(-4)}`;
}

export function GuardianMatrixCard({
  guardians,
  quorum,
  isTriggered,
  isOwner,
  onInitiateTrigger,
  isLoading = false,
}: GuardianMatrixProps) {
  const [showConfirm, setShowConfirm] = useState(false);
  const [initiating, setInitiating] = useState(false);

  const signedCount = guardians.filter((g) => g.hasSigned).length;
  const quorumMet = signedCount >= quorum;
  const canInitiate = isOwner && !isTriggered && !showConfirm;

  const handleInitiate = async () => {
    if (!onInitiateTrigger) return;
    setInitiating(true);
    try {
      await onInitiateTrigger();
      setShowConfirm(false);
    } catch {
      // error handled by parent
    } finally {
      setInitiating(false);
    }
  };

  return (
    <div
      className="rounded-2xl bg-[#0A0F11] border border-white/[0.06] p-6"
      data-testid="guardian-matrix-card"
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-5">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-pink-400/10 border border-pink-400/20 flex items-center justify-center">
            {quorumMet ? (
              <ShieldCheck size={18} className="text-pink-400" />
            ) : (
              <Shield size={18} className="text-pink-400" />
            )}
          </div>
          <div>
            <h3 className="text-base font-semibold text-white">Emergency Guardians</h3>
            <p className="text-xs text-slate-500 mt-0.5">
              {quorum}-of-{guardians.length} multi-sig required
            </p>
          </div>
        </div>

        {/* Quorum status badge */}
        {isTriggered ? (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[11px] font-medium bg-red-500/10 text-red-400 border border-red-500/20">
            <ShieldAlert size={11} />
            Triggered
          </span>
        ) : quorumMet ? (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[11px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <CheckCircle size={11} />
            Quorum Met
          </span>
        ) : (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[11px] font-medium bg-yellow-500/10 text-yellow-400 border border-yellow-500/20">
            <Clock size={11} />
            Pending
          </span>
        )}
      </div>

      {/* Progress bar */}
      <div className="mb-5">
        <div className="flex items-center justify-between text-xs text-slate-500 mb-1.5">
          <span>Signatures collected</span>
          <span className="font-medium text-slate-300">
            {signedCount}/{guardians.length} (need {quorum})
          </span>
        </div>
        <div className="h-2 rounded-full bg-white/[0.06] overflow-hidden">
          <div
            className={`h-full rounded-full transition-all duration-500 ${
              quorumMet ? "bg-emerald-400" : "bg-[#33C5E0]"
            }`}
            style={{
              width: `${guardians.length > 0 ? (signedCount / guardians.length) * 100 : 0}%`,
            }}
            role="progressbar"
            aria-valuenow={signedCount}
            aria-valuemin={0}
            aria-valuemax={guardians.length}
            aria-label={`Signature progress: ${signedCount} of ${guardians.length}`}
          />
        </div>
      </div>

      {/* Guardian list */}
      <div className="space-y-2 mb-5" role="list" aria-label="Guardian signatures">
        {guardians.map((guardian, i) => (
          <div
            key={guardian.address}
            role="listitem"
            className={`flex items-center justify-between p-3 rounded-xl border transition-colors ${
              guardian.hasSigned
                ? "bg-emerald-400/[0.04] border-emerald-400/10"
                : "bg-white/[0.02] border-white/[0.05]"
            }`}
          >
            <div className="flex items-center gap-3 min-w-0">
              {/* Signature indicator */}
              <div
                className={`w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 ${
                  guardian.hasSigned
                    ? "bg-emerald-400/10 text-emerald-400"
                    : "bg-white/[0.04] text-slate-600"
                }`}
              >
                {guardian.hasSigned ? (
                  <CheckCircle size={14} />
                ) : (
                  <XCircle size={14} />
                )}
              </div>
              <div className="min-w-0">
                <p className="text-sm font-medium text-white truncate">
                  {guardian.name || `Guardian ${i + 1}`}
                </p>
                <p className="text-[11px] text-slate-500 font-mono truncate">
                  {truncateAddress(guardian.address)}
                </p>
              </div>
            </div>
            <div className="text-right flex-shrink-0 ml-3">
              {guardian.hasSigned ? (
                <span className="text-[11px] text-emerald-400 font-medium">Signed</span>
              ) : (
                <span className="text-[11px] text-slate-500">Awaiting</span>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Initiate trigger button */}
      {isOwner && !isTriggered && (
        <div>
          {!showConfirm ? (
            <button
              onClick={() => setShowConfirm(true)}
              disabled={!quorumMet || isLoading}
              className="w-full py-2.5 rounded-xl text-sm font-semibold bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20 transition-colors disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-2"
              data-testid="initiate-trigger-btn"
            >
              <AlertTriangle size={14} />
              Initiate Guardian Trigger
            </button>
          ) : (
            <div className="p-4 rounded-xl bg-red-500/[0.06] border border-red-500/20">
              <p className="text-sm text-red-300 mb-3">
                This will initiate an emergency recovery. This action is irreversible.
              </p>
              <div className="flex gap-2">
                <button
                  onClick={handleInitiate}
                  disabled={initiating}
                  className="flex-1 py-2 rounded-lg text-sm font-semibold bg-red-500 text-white hover:bg-red-600 transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
                >
                  {initiating && <Loader2 size={14} className="animate-spin" />}
                  {initiating ? "Initiating…" : "Yes, Initiate"}
                </button>
                <button
                  onClick={() => setShowConfirm(false)}
                  disabled={initiating}
                  className="flex-1 py-2 rounded-lg text-sm font-medium bg-white/5 text-slate-300 border border-white/10 hover:bg-white/10 transition-colors disabled:opacity-50"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
          {!quorumMet && !isLoading && (
            <p className="text-[11px] text-slate-500 text-center mt-2">
              Requires {quorum} signatures to enable trigger
            </p>
          )}
        </div>
      )}

      {/* Non-owner notice */}
      {!isOwner && (
        <p className="text-[11px] text-slate-500 text-center">
          Only the plan owner can initiate an emergency trigger
        </p>
      )}
    </div>
  );
}
