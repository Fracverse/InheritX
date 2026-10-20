"use client";

import React, { useState, useMemo, useCallback } from "react";
import {
  Calculator,
  TrendingUp,
  Users,
  Clock,
  DollarSign,
  ArrowRight,
  Info,
} from "lucide-react";

const APY_RATE = 5.2;
const PROTOCOL_FEE_BPS = 50;
const NETWORK_FEE_XLM = 0.0001;

export interface BeneficiarySplit {
  name: string;
  percentage: number;
}

interface PayoutSimulatorProps {
  defaultDeposit?: number;
  defaultDuration?: number;
}

export function PayoutSimulator({
  defaultDeposit = 1000,
  defaultDuration = 365,
}: PayoutSimulatorProps) {
  const [deposit, setDeposit] = useState(defaultDeposit);
  const [duration, setDuration] = useState(defaultDuration);
  const [beneficiaries, setBeneficiaries] = useState<BeneficiarySplit[]>([
    { name: "Beneficiary 1", percentage: 60 },
    { name: "Beneficiary 2", percentage: 40 },
  ]);
  const [customApy, setCustomApy] = useState(APY_RATE);

  const addBeneficiary = useCallback(() => {
    setBeneficiaries((prev) => {
      if (prev.length >= 5) return prev;
      const remaining = 100 - prev.reduce((s, b) => s + b.percentage, 0);
      const newShare = Math.max(0, Math.round(remaining / (prev.length + 1)));
      return [
        ...prev.map((b) => ({ ...b, percentage: Math.floor(b.percentage * 0.7) })),
        { name: `Beneficiary ${prev.length + 1}`, percentage: newShare },
      ];
    });
  }, []);

  const removeBeneficiary = useCallback((index: number) => {
    setBeneficiaries((prev) => {
      if (prev.length <= 1) return prev;
      const removed = prev[index];
      const others = prev.filter((_, i) => i !== index);
      const share = Math.round(removed.percentage / (others.length || 1));
      return others.map((b) => ({ ...b, percentage: b.percentage + share }));
    });
  }, []);

  const updateBeneficiaryPct = useCallback((index: number, pct: number) => {
    setBeneficiaries((prev) =>
      prev.map((b, i) => (i === index ? { ...b, percentage: Math.max(0, Math.min(100, pct)) } : b))
    );
  }, []);

  const totalPct = beneficiaries.reduce((s, b) => s + b.percentage, 0);
  const pctValid = totalPct === 100;

  const projections = useMemo(() => {
    const years = duration / 365;
    const grossYield = deposit * (customApy / 100) * years;
    const protocolFee = grossYield * (PROTOCOL_FEE_BPS / 10000);
    const netYield = grossYield - protocolFee;
    const totalPayout = deposit + netYield;
    const networkFees = NETWORK_FEE_XLM * (beneficiaries.length + 2);

    return {
      grossYield,
      protocolFee,
      netYield,
      totalPayout,
      networkFees,
      monthlyYield: netYield / (duration / 30),
    };
  }, [deposit, duration, customApy, beneficiaries.length]);

  return (
    <div
      className="rounded-2xl bg-[#0A0F11] border border-white/[0.06] p-6"
      data-testid="payout-simulator"
    >
      {/* Header */}
      <div className="flex items-center gap-3 mb-6">
        <div className="w-10 h-10 rounded-xl bg-[#33C5E0]/10 border border-[#33C5E0]/20 flex items-center justify-center">
          <Calculator size={18} className="text-[#33C5E0]" />
        </div>
        <div>
          <h3 className="text-base font-semibold text-white">Payout Simulator</h3>
          <p className="text-xs text-slate-500 mt-0.5">
            Project your yield and payout distribution
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Inputs */}
        <div className="space-y-4">
          {/* Deposit */}
          <div>
            <label htmlFor="sim-deposit" className="block text-xs text-slate-400 mb-1.5">
              Deposit Amount (USD)
            </label>
            <div className="relative">
              <DollarSign size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
              <input
                id="sim-deposit"
                type="number"
                min={1}
                value={deposit}
                onChange={(e) => setDeposit(Math.max(0, Number(e.target.value)))}
                className="w-full pl-9 pr-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-sm text-white focus:outline-none focus:border-[#33C5E0]/40 transition-colors"
              />
            </div>
          </div>

          {/* Duration */}
          <div>
            <label htmlFor="sim-duration" className="block text-xs text-slate-400 mb-1.5">
              Duration
            </label>
            <div className="flex items-center gap-2">
              <input
                id="sim-duration"
                type="range"
                min={30}
                max={1825}
                step={30}
                value={duration}
                onChange={(e) => setDuration(Number(e.target.value))}
                className="flex-1 accent-[#33C5E0]"
              />
              <span className="text-sm text-white font-medium w-16 text-right">
                {duration}d
              </span>
            </div>
            <p className="text-[11px] text-slate-500 mt-1">
              ≈ {(duration / 365).toFixed(1)} year{duration >= 730 ? "s" : ""}
            </p>
          </div>

          {/* APY */}
          <div>
            <label htmlFor="sim-apy" className="block text-xs text-slate-400 mb-1.5">
              Estimated APY (%)
            </label>
            <div className="relative">
              <TrendingUp size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
              <input
                id="sim-apy"
                type="number"
                min={0}
                max={50}
                step={0.1}
                value={customApy}
                onChange={(e) => setCustomApy(Math.max(0, Math.min(50, Number(e.target.value))))}
                className="w-full pl-9 pr-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-sm text-white focus:outline-none focus:border-[#33C5E0]/40 transition-colors"
              />
            </div>
          </div>

          {/* Beneficiaries */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="text-xs text-slate-400">Beneficiaries</label>
              <button
                onClick={addBeneficiary}
                disabled={beneficiaries.length >= 5}
                className="text-[11px] text-[#33C5E0] hover:underline disabled:opacity-40 disabled:no-underline"
              >
                + Add
              </button>
            </div>
            <div className="space-y-2">
              {beneficiaries.map((b, i) => (
                <div key={i} className="flex items-center gap-2">
                  <span className="text-xs text-slate-400 w-24 truncate">{b.name}</span>
                  <input
                    type="number"
                    min={0}
                    max={100}
                    value={b.percentage}
                    onChange={(e) => updateBeneficiaryPct(i, Number(e.target.value))}
                    className="w-16 px-2 py-1.5 rounded-lg bg-white/5 border border-white/10 text-xs text-white text-center focus:outline-none focus:border-[#33C5E0]/40"
                    aria-label={`${b.name} percentage`}
                  />
                  <span className="text-xs text-slate-500">%</span>
                  {beneficiaries.length > 1 && (
                    <button
                      onClick={() => removeBeneficiary(i)}
                      className="text-[11px] text-red-400 hover:text-red-300 ml-auto"
                      aria-label={`Remove ${b.name}`}
                    >
                      ×
                    </button>
                  )}
                </div>
              ))}
            </div>
            <div className={`flex items-center justify-between mt-2 text-[11px] ${pctValid ? "text-emerald-400" : "text-red-400"}`}>
              <span>Total: {totalPct}%</span>
              {pctValid ? (
                <span className="flex items-center gap-1">
                  <Info size={10} /> Valid allocation
                </span>
              ) : (
                <span>Must equal 100%</span>
              )}
            </div>
          </div>
        </div>

        {/* Projections */}
        <div className="space-y-4">
          {/* Summary card */}
          <div className="p-4 rounded-xl bg-gradient-to-br from-[#33C5E0]/[0.06] to-transparent border border-[#33C5E0]/10">
            <p className="text-xs text-slate-400 mb-1">Total Projected Payout</p>
            <p className="text-3xl font-bold text-white">
              ${projections.totalPayout.toLocaleString("en-US", { maximumFractionDigits: 2 })}
            </p>
            <p className="text-[11px] text-emerald-400 mt-1">
              +${projections.netYield.toLocaleString("en-US", { maximumFractionDigits: 2 })} net yield
            </p>
          </div>

          {/* Breakdown */}
          <div className="space-y-3">
            <div className="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/[0.05]">
              <span className="text-xs text-slate-400">Initial Deposit</span>
              <span className="text-sm text-white font-medium">
                ${deposit.toLocaleString()}
              </span>
            </div>
            <div className="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/[0.05]">
              <span className="text-xs text-slate-400">Gross Yield</span>
              <span className="text-sm text-emerald-400 font-medium">
                +${projections.grossYield.toLocaleString("en-US", { maximumFractionDigits: 2 })}
              </span>
            </div>
            <div className="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/[0.05]">
              <span className="text-xs text-slate-400">
                Protocol Fee ({PROTOCOL_FEE_BPS / 100}%)
              </span>
              <span className="text-sm text-red-400 font-medium">
                -${projections.protocolFee.toLocaleString("en-US", { maximumFractionDigits: 2 })}
              </span>
            </div>
            <div className="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/[0.05]">
              <span className="text-xs text-slate-400">Est. Network Fees</span>
              <span className="text-sm text-slate-300 font-medium">
                {projections.networkFees.toFixed(6)} XLM
              </span>
            </div>
          </div>

          {/* Per-beneficiary projection */}
          {pctValid && (
            <div className="p-4 rounded-xl bg-white/[0.03] border border-white/[0.05]">
              <p className="text-xs text-slate-400 mb-3 flex items-center gap-1.5">
                <Users size={12} />
                Per-Beneficiary Payout
              </p>
              <div className="space-y-2">
                {beneficiaries.map((b, i) => {
                  const payout = (projections.totalPayout * b.percentage) / 100;
                  return (
                    <div key={i} className="flex items-center justify-between">
                      <span className="text-xs text-slate-300">
                        {b.name} ({b.percentage}%)
                      </span>
                      <span className="text-sm text-white font-medium">
                        ${payout.toLocaleString("en-US", { maximumFractionDigits: 2 })}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Monthly yield */}
          <div className="flex items-center gap-2 text-[11px] text-slate-500">
            <Clock size={11} />
            <span>
              ≈ ${projections.monthlyYield.toLocaleString("en-US", { maximumFractionDigits: 2 })}
              /month avg yield
            </span>
            <ArrowRight size={10} />
            <span>compounded</span>
          </div>
        </div>
      </div>

      {/* Disclaimer */}
      <p className="text-[10px] text-slate-600 mt-6 leading-relaxed">
        Projections are estimates based on current APY rates and are not guaranteed.
        Actual returns may vary due to market conditions, protocol fee changes,
        and network transaction costs. This is not financial advice.
      </p>
    </div>
  );
}
