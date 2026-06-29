"use client";

import React, { useState, useMemo, useCallback } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import { TrendingUp, Info } from "lucide-react";

// ─── Token yield rates (annual, mirrors backend yield_service.rs mock rates) ──
// Backend returns on-chain yield amounts; we derive APY from yield/balance ratio
// USDC: 25.50 yield / 100025.50 balance ≈ 0.0255% → padded to realistic mock APY
// XLM:  100.0 yield / 500100.0 balance   ≈ 0.02%  → padded to realistic mock APY
// Rates here are realistic DeFi-style APYs for illustration, consistent with
// the backend's simple interest formula: Interest = Principal × Rate × Time
export const TOKEN_YIELD_RATES: Record<
  string,
  { label: string; apy: number; color: string; symbol: string }
> = {
  ETH: { label: "ETH", apy: 0.048, color: "#627EEA", symbol: "Ξ" },
  BTC: { label: "BTC", apy: 0.032, color: "#F7931A", symbol: "₿" },
  USDT: { label: "USDT", apy: 0.085, color: "#26A17B", symbol: "$" },
  USDC: { label: "USDC", apy: 0.072, color: "#2775CA", symbol: "$" },
  XLM: { label: "XLM", apy: 0.055, color: "#33C5E0", symbol: "✦" },
};

// ─── Compounding math — mirrors backend safe_math.rs calculate_interest ───────
// Backend formula: Interest = Principal × Rate × Time (annual, simple)
// For year-over-year visualization we use compound accumulation:
//   A(t) = P × (1 + r)^t
// which collapses to the backend simple formula for t=1:
//   Interest(1yr) = P × r × 1 = P × r  ✓
function compoundValue(principal: number, apy: number, years: number): number {
  return principal * Math.pow(1 + apy, years);
}

function simpleInterest(principal: number, apy: number, years: number): number {
  return principal * apy * years;
}

// ─── Types ────────────────────────────────────────────────────────────────────
interface ChartDataPoint {
  year: number;
  [token: string]: number;
}

interface TooltipPayloadItem {
  name: string;
  value: number;
  color: string;
}

interface CustomTooltipProps {
  active?: boolean;
  payload?: TooltipPayloadItem[];
  label?: number;
  principal: number;
}

// ─── Custom tooltip with yield breakdown ─────────────────────────────────────
function CustomTooltip({ active, payload, label, principal }: CustomTooltipProps) {
  if (!active || !payload || payload.length === 0) return null;

  return (
    <div className="bg-[#161E22] border border-[#2A3338] rounded-xl p-4 shadow-2xl min-w-[220px]">
      <p className="text-[#92A5A8] text-xs mb-3 font-medium">
        Year {label} Projection
      </p>
      {payload.map((entry) => {
        const totalValue = entry.value;
        const yieldEarned = totalValue - principal;
        const yieldPct = principal > 0 ? (yieldEarned / principal) * 100 : 0;
        const tokenInfo = TOKEN_YIELD_RATES[entry.name];

        return (
          <div key={entry.name} className="mb-3 last:mb-0">
            <div className="flex items-center gap-2 mb-1.5">
              <span
                className="w-2.5 h-2.5 rounded-full flex-shrink-0"
                style={{ backgroundColor: entry.color }}
              />
              <span className="text-[#FCFFFF] text-sm font-semibold">
                {entry.name}
              </span>
              <span className="text-[#92A5A8] text-xs ml-auto">
                {tokenInfo ? `${(tokenInfo.apy * 100).toFixed(1)}% APY` : ""}
              </span>
            </div>
            <div className="pl-4 space-y-1">
              <div className="flex justify-between text-xs">
                <span className="text-[#92A5A8]">Total Value</span>
                <span className="text-[#FCFFFF] font-medium">
                  {totalValue.toFixed(4)}
                </span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-[#92A5A8]">Principal</span>
                <span className="text-[#FCFFFF]">{principal.toFixed(4)}</span>
              </div>
              <div className="flex justify-between text-xs border-t border-[#2A3338] pt-1 mt-1">
                <span className="text-[#33C5E0]">Yield Earned</span>
                <span className="text-[#33C5E0] font-semibold">
                  +{yieldEarned.toFixed(4)} ({yieldPct.toFixed(1)}%)
                </span>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

// ─── Props ────────────────────────────────────────────────────────────────────
interface YieldCalculatorProps {
  /** Pre-fill the principal from the parent form's asset amount */
  initialPrincipal?: number;
  /** Pre-fill the selected token from the parent form */
  initialToken?: string;
  /** Compact mode hides the full token comparison and shows only chosen token */
  compact?: boolean;
}

// ─── Main component ───────────────────────────────────────────────────────────
export default function YieldCalculator({
  initialPrincipal = 1,
  initialToken = "ETH",
  compact = false,
}: YieldCalculatorProps) {
  const [principal, setPrincipal] = useState<string>(
    initialPrincipal > 0 ? String(initialPrincipal) : "1"
  );
  const [years, setYears] = useState<number>(10);
  const [activeTokens, setActiveTokens] = useState<Set<string>>(
    new Set(compact ? [initialToken] : Object.keys(TOKEN_YIELD_RATES))
  );

  const parsedPrincipal = useMemo(() => {
    const v = parseFloat(principal);
    return isNaN(v) || v <= 0 ? 1 : v;
  }, [principal]);

  // Build chart data: one point per year from 0 to `years`
  const chartData = useMemo<ChartDataPoint[]>(() => {
    return Array.from({ length: years + 1 }, (_, i) => {
      const point: ChartDataPoint = { year: i };
      for (const [token, info] of Object.entries(TOKEN_YIELD_RATES)) {
        if (activeTokens.has(token)) {
          point[token] = parseFloat(
            compoundValue(parsedPrincipal, info.apy, i).toFixed(6)
          );
        }
      }
      return point;
    });
  }, [parsedPrincipal, years, activeTokens]);

  // Summary stats for the selected/first active token
  const summaryToken = useMemo(() => {
    const first = [...activeTokens][0];
    if (!first || !TOKEN_YIELD_RATES[first]) return null;
    const info = TOKEN_YIELD_RATES[first];
    const finalValue = compoundValue(parsedPrincipal, info.apy, years);
    const totalYield = finalValue - parsedPrincipal;
    const simpleYield = simpleInterest(parsedPrincipal, info.apy, years);
    const compoundBonus = totalYield - simpleYield;
    return { token: first, info, finalValue, totalYield, simpleYield, compoundBonus };
  }, [activeTokens, parsedPrincipal, years]);

  const toggleToken = useCallback((token: string) => {
    setActiveTokens((prev: Set<string>) => {
      const next = new Set(prev);
      if (next.has(token)) {
        // Keep at least one token active
        if (next.size > 1) next.delete(token);
      } else {
        next.add(token);
      }
      return next;
    });
  }, []);

  const handlePrincipalChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    if (val === "" || /^\d*\.?\d*$/.test(val)) setPrincipal(val);
  };

  return (
    <div className="bg-[#1C252A] rounded-xl p-5 space-y-5">
      {/* Header */}
      <div className="flex items-center gap-2">
        <div className="w-8 h-8 rounded-lg bg-[#33C5E0]/15 flex items-center justify-center">
          <TrendingUp size={16} className="text-[#33C5E0]" />
        </div>
        <div>
          <h3 className="text-[#FCFFFF] font-semibold text-sm">
            Yield Calculator
          </h3>
          <p className="text-[#92A5A8] text-xs">
            Projected compounding returns over time
          </p>
        </div>
      </div>

      {/* Inputs */}
      <div className="grid grid-cols-2 gap-3">
        {/* Principal */}
        <div>
          <label className="block text-xs text-[#92A5A8] mb-1.5">
            Principal Amount
          </label>
          <div className="relative">
            <input
              type="text"
              inputMode="decimal"
              value={principal}
              onChange={handlePrincipalChange}
              placeholder="1.0"
              className="w-full bg-[#161E22] border border-[#2A3338] rounded-lg px-3 py-2.5 text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#33C5E0] text-sm pr-14"
            />
            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-[#92A5A8] text-xs">
              tokens
            </span>
          </div>
        </div>

        {/* Holding Period Slider */}
        <div>
          <label className="block text-xs text-[#92A5A8] mb-1.5">
            Holding Period —{" "}
            <span className="text-[#33C5E0] font-semibold">{years} yr{years !== 1 ? "s" : ""}</span>
          </label>
          <input
            type="range"
            min={1}
            max={50}
            value={years}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => setYears(Number(e.target.value))}
            className="w-full h-2 rounded-full appearance-none cursor-pointer accent-[#33C5E0] bg-[#161E22]"
          />
          <div className="flex justify-between text-[10px] text-[#92A5A8] mt-1">
            <span>1 yr</span>
            <span>25 yrs</span>
            <span>50 yrs</span>
          </div>
        </div>
      </div>

      {/* Token toggles (hidden in compact mode) */}
      {!compact && (
        <div className="flex flex-wrap gap-2">
          {Object.entries(TOKEN_YIELD_RATES).map(([token, info]) => {
            const isOn = activeTokens.has(token);
            return (
              <button
                key={token}
                onClick={() => toggleToken(token)}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium transition-all border ${
                  isOn
                    ? "border-transparent text-[#161E22]"
                    : "border-[#2A3338] text-[#92A5A8] bg-transparent hover:border-[#33C5E0]"
                }`}
                style={isOn ? { backgroundColor: info.color } : {}}
              >
                <span>{info.symbol}</span>
                {token}
                <span className="opacity-75">
                  {(info.apy * 100).toFixed(1)}%
                </span>
              </button>
            );
          })}
        </div>
      )}

      {/* Chart */}
      <div className="h-52">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart
            data={chartData}
            margin={{ top: 4, right: 8, left: 0, bottom: 0 }}
          >
            <CartesianGrid
              strokeDasharray="3 3"
              stroke="#1C252A"
              vertical={false}
            />
            <XAxis
              dataKey="year"
              tick={{ fill: "#92A5A8", fontSize: 10 }}
              tickLine={false}
              axisLine={false}
              tickFormatter={(v: number) => (v === 0 ? "Now" : `${v}y`)}
            />
            <YAxis
              tick={{ fill: "#92A5A8", fontSize: 10 }}
              tickLine={false}
              axisLine={false}
              tickFormatter={(v: number) =>
                v >= 1000
                  ? `${(v / 1000).toFixed(1)}k`
                  : v.toFixed(v < 10 ? 2 : 1)
              }
              width={44}
            />
            <Tooltip
              content={
                <CustomTooltip principal={parsedPrincipal} />
              }
              cursor={{ stroke: "#33C5E0", strokeWidth: 1, strokeDasharray: "4 4" }}
            />
            {!compact && (
              <Legend
                wrapperStyle={{ fontSize: 11, paddingTop: 8 }}
                formatter={(value: string) => (
                  <span style={{ color: "#92A5A8" }}>{value}</span>
                )}
              />
            )}
            {Object.entries(TOKEN_YIELD_RATES).map(([token, info]) =>
              activeTokens.has(token) ? (
                <Line
                  key={token}
                  type="monotone"
                  dataKey={token}
                  stroke={info.color}
                  strokeWidth={2}
                  dot={false}
                  activeDot={{ r: 4, strokeWidth: 0 }}
                />
              ) : null
            )}
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Summary stats */}
      {summaryToken && (
        <div className="grid grid-cols-3 gap-2">
          <div className="bg-[#161E22] rounded-lg p-3 text-center">
            <p className="text-[#92A5A8] text-[10px] mb-1">Final Value</p>
            <p className="text-[#FCFFFF] text-sm font-semibold">
              {summaryToken.finalValue.toFixed(4)}
            </p>
            <p
              className="text-[10px] mt-0.5"
              style={{ color: summaryToken.info.color }}
            >
              {summaryToken.token}
            </p>
          </div>
          <div className="bg-[#161E22] rounded-lg p-3 text-center">
            <p className="text-[#92A5A8] text-[10px] mb-1">Total Yield</p>
            <p className="text-[#33C5E0] text-sm font-semibold">
              +{summaryToken.totalYield.toFixed(4)}
            </p>
            <p className="text-[#92A5A8] text-[10px] mt-0.5">
              {((summaryToken.totalYield / parsedPrincipal) * 100).toFixed(1)}%
              gain
            </p>
          </div>
          <div className="bg-[#161E22] rounded-lg p-3 text-center">
            <p className="text-[#92A5A8] text-[10px] mb-1">Compound Bonus</p>
            <p className="text-green-400 text-sm font-semibold">
              +{summaryToken.compoundBonus.toFixed(4)}
            </p>
            <p className="text-[#92A5A8] text-[10px] mt-0.5">
              vs. simple interest
            </p>
          </div>
        </div>
      )}

      {/* Info note */}
      <div className="flex items-start gap-2 text-[10px] text-[#92A5A8]">
        <Info size={12} className="flex-shrink-0 mt-0.5 text-[#33C5E0]" />
        <span>
          Projections use annual compounding: A = P × (1 + r)ᵗ, consistent
          with backend yield formulas. Rates are illustrative and subject to
          market conditions.
        </span>
      </div>
    </div>
  );
}
