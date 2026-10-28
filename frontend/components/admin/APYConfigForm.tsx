"use client";

import React, { useState, useCallback } from "react";
import {
  Settings,
  TrendingUp,
  Save,
  RotateCcw,
  AlertTriangle,
  CheckCircle,
  Loader2,
  Info,
} from "lucide-react";

interface AssetConfig {
  symbol: string;
  name: string;
  apyBps: number;
  previousApyBps: number;
}

const DEFAULT_ASSETS: AssetConfig[] = [
  { symbol: "XLM", name: "Stellar Lumens", apyBps: 520, previousApyBps: 520 },
  { symbol: "USDC", name: "USD Coin", apyBps: 450, previousApyBps: 450 },
  { symbol: "EURC", name: "Euro Coin", apyBps: 420, previousApyBps: 420 },
];

interface APYConfigFormProps {
  onSave?: (configs: AssetConfig[]) => Promise<void>;
  isLoading?: boolean;
}

export function APYConfigForm({ onSave, isLoading }: APYConfigFormProps) {
  const [assets, setAssets] = useState<AssetConfig[]>(DEFAULT_ASSETS);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [showConfirm, setShowConfirm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<"idle" | "success" | "error">("idle");

  const updateApy = useCallback((index: number, value: string) => {
    const num = Number(value);
    setAssets((prev) =>
      prev.map((a, i) => (i === index ? { ...a, apyBps: isNaN(num) ? 0 : Math.round(num) } : a))
    );
    setErrors((prev) => {
      const next = { ...prev };
      if (isNaN(num)) next[index] = "Must be a number";
      else if (num < 0) next[index] = "Cannot be negative";
      else if (num > 5000) next[index] = "Exceeds max 50%";
      else delete next[index];
      return next;
    });
  }, []);

  const hasErrors = Object.keys(errors).length > 0;
  const hasChanges = assets.some((a) => a.apyBps !== a.previousApyBps);

  const handleSave = async () => {
    if (hasErrors || !hasChanges) return;
    setShowConfirm(false);
    setSaving(true);
    setSaveStatus("idle");
    try {
      await onSave?.(assets);
      setSaveStatus("success");
      setAssets((prev) => prev.map((a) => ({ ...a, previousApyBps: a.apyBps })));
      setTimeout(() => setSaveStatus("idle"), 3000);
    } catch {
      setSaveStatus("error");
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setAssets((prev) => prev.map((a) => ({ ...a, apyBps: a.previousApyBps })));
    setErrors({});
  };

  return (
    <div
      className="rounded-2xl bg-[#0A0F11] border border-white/[0.06] p-6"
      data-testid="apy-config-form"
    >
      {/* Header */}
      <div className="flex items-center gap-3 mb-6">
        <div className="w-10 h-10 rounded-xl bg-emerald-400/10 border border-emerald-400/20 flex items-center justify-center">
          <TrendingUp size={18} className="text-emerald-400" />
        </div>
        <div>
          <h3 className="text-base font-semibold text-white">APY Configuration</h3>
          <p className="text-xs text-slate-500 mt-0.5">
            Update yield rates for supported assets
          </p>
        </div>
      </div>

      {/* Save status */}
      {saveStatus === "success" && (
        <div className="mb-4 flex items-center gap-2 p-3 rounded-xl bg-emerald-400/10 border border-emerald-400/20 text-emerald-400 text-sm">
          <CheckCircle size={16} />
          Configuration saved successfully
        </div>
      )}
      {saveStatus === "error" && (
        <div className="mb-4 flex items-center gap-2 p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
          <AlertTriangle size={16} />
          Failed to save configuration. Please try again.
        </div>
      )}

      {/* Asset rows */}
      <div className="space-y-4 mb-6">
        {assets.map((asset, i) => {
          const changed = asset.apyBps !== asset.previousApyBps;
          return (
            <div
              key={asset.symbol}
              className={`p-4 rounded-xl border transition-colors ${
                errors[i]
                  ? "bg-red-500/[0.04] border-red-500/20"
                  : changed
                  ? "bg-emerald-400/[0.04] border-emerald-400/15"
                  : "bg-white/[0.02] border-white/[0.05]"
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 rounded-lg bg-white/[0.05] flex items-center justify-center">
                    <span className="text-[10px] font-bold text-slate-300">
                      {asset.symbol.slice(0, 3)}
                    </span>
                  </div>
                  <div>
                    <p className="text-sm font-medium text-white">{asset.symbol}</p>
                    <p className="text-[11px] text-slate-500">{asset.name}</p>
                  </div>
                </div>

                <div className="flex items-center gap-3">
                  {changed && (
                    <span className="text-[11px] text-emerald-400 flex items-center gap-1">
                      <Settings size={10} />
                      Modified
                    </span>
                  )}
                  <div className="flex items-center gap-1">
                    <input
                      type="number"
                      min={0}
                      max={5000}
                      value={asset.apyBps}
                      onChange={(e) => updateApy(i, e.target.value)}
                      className={`w-24 px-3 py-2 rounded-lg bg-white/5 border text-sm text-white text-right font-mono focus:outline-none transition-colors ${
                        errors[i]
                          ? "border-red-500/40 focus:border-red-500/60"
                          : "border-white/10 focus:border-[#33C5E0]/40"
                      }`}
                      aria-label={`${asset.symbol} APY in basis points`}
                      aria-invalid={!!errors[i]}
                    />
                    <span className="text-xs text-slate-500">bps</span>
                  </div>
                  <span className="text-xs text-slate-400 w-14 text-right">
                    {(asset.apyBps / 100).toFixed(2)}%
                  </span>
                </div>
              </div>

              {errors[i] && (
                <p className="text-[11px] text-red-400 mt-2 flex items-center gap-1">
                  <AlertTriangle size={10} />
                  {errors[i]}
                </p>
              )}
            </div>
          );
        })}
      </div>

      {/* Info note */}
      <div className="flex items-start gap-2 p-3 rounded-xl bg-[#33C5E0]/[0.04] border border-[#33C5E0]/10 mb-6">
        <Info size={14} className="text-[#33C5E0] flex-shrink-0 mt-0.5" />
        <p className="text-[11px] text-slate-400 leading-relaxed">
          APY values are in basis points (bps). 100 bps = 1.00%. Maximum allowed is 5000 bps (50%).
          Changes take effect for new plans immediately. Existing plans retain their original APY.
        </p>
      </div>

      {/* Actions */}
      <div className="flex gap-3">
        <button
          onClick={() => setShowConfirm(true)}
          disabled={hasErrors || !hasChanges || saving || isLoading}
          className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl bg-[#33C5E0] text-black text-sm font-semibold hover:bg-[#33C5E0]/90 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
          data-testid="save-apy-btn"
        >
          {saving ? (
            <Loader2 size={15} className="animate-spin" />
          ) : (
            <Save size={15} />
          )}
          {saving ? "Saving…" : "Save Changes"}
        </button>
        <button
          onClick={handleReset}
          disabled={!hasChanges || saving}
          className="px-4 py-2.5 rounded-xl bg-white/5 text-slate-300 border border-white/10 text-sm font-medium hover:bg-white/10 transition-colors disabled:opacity-40"
        >
          <RotateCcw size={15} />
        </button>
      </div>

      {/* Confirmation modal */}
      {showConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60">
          <div className="bg-[#0A0F11] border border-white/[0.08] rounded-2xl p-6 max-w-sm w-full">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-xl bg-yellow-400/10 border border-yellow-400/20 flex items-center justify-center">
                <AlertTriangle size={18} className="text-yellow-400" />
              </div>
              <h4 className="text-base font-semibold text-white">Confirm APY Changes</h4>
            </div>
            <p className="text-sm text-slate-400 mb-4">
              This will update yield rates for {assets.filter((a) => a.apyBps !== a.previousApyBps).length} asset(s).
              New plans will use these rates immediately.
            </p>
            <div className="space-y-1.5 mb-5">
              {assets
                .filter((a) => a.apyBps !== a.previousApyBps)
                .map((a) => (
                  <div key={a.symbol} className="flex items-center justify-between text-xs p-2 rounded-lg bg-white/[0.03]">
                    <span className="text-slate-300">{a.symbol}</span>
                    <span className="text-slate-500">
                      {(a.previousApyBps / 100).toFixed(2)}% →{" "}
                      <span className="text-emerald-400 font-medium">
                        {(a.apyBps / 100).toFixed(2)}%
                      </span>
                    </span>
                  </div>
                ))}
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleSave}
                disabled={saving}
                className="flex-1 py-2.5 rounded-xl bg-[#33C5E0] text-black text-sm font-semibold hover:bg-[#33C5E0]/90 transition-colors disabled:opacity-50"
              >
                {saving ? "Saving…" : "Confirm & Save"}
              </button>
              <button
                onClick={() => setShowConfirm(false)}
                disabled={saving}
                className="flex-1 py-2.5 rounded-xl bg-white/5 text-slate-300 border border-white/10 text-sm font-medium hover:bg-white/10 transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
