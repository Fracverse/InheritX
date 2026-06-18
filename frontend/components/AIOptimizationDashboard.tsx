"use client";

import React, { useState, useCallback, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Brain,
  TrendingUp,
  TrendingDown,
  Shield,
  ChevronDown,
  ChevronUp,
  Check,
  X,
  Sliders,
  Info,
  AlertTriangle,
  Target,
  Activity,
  BarChart2,
  Clock,
  RefreshCw,
} from "lucide-react";
import type {
  AssetAllocation,
  OptimizationRecommendation,
} from "@/app/lib/api/aiOptimization";

// ─── Types ────────────────────────────────────────────────────────────────────

export interface AIOptimizationDashboardProps {
  inheritancePlanId: number;
  currentAllocations: AssetAllocation[];
  optimizationRecommendations: OptimizationRecommendation;
  onAcceptRecommendation: (recommendation: OptimizationRecommendation) => void;
  onRejectRecommendation: (reason: string) => void;
  onCustomizeRecommendation: (customAllocations: AssetAllocation[]) => void;
}

export enum OptimizationScenario {
  CONSERVATIVE = "conservative",
  MODERATE = "moderate",
  AGGRESSIVE = "aggressive",
  CUSTOM = "custom",
}

interface DashboardState {
  customAllocations: AssetAllocation[];
  selectedScenario: OptimizationScenario;
  showDetails: boolean;
  activeTab: "overview" | "allocation" | "projections" | "customize";
  showRejectModal: boolean;
  showAcceptConfirm: boolean;
  rejectReason: string;
  tooltipId: string | null;
  expandedReasoning: boolean;
}

// ─── Color Palette ────────────────────────────────────────────────────────────

const CHART_COLORS = [
  "#33C5E0",
  "#D4A017",
  "#22c55e",
  "#a855f7",
  "#f97316",
  "#ec4899",
  "#06b6d4",
  "#84cc16",
];

const scenarioMultipliers: Record<OptimizationScenario, number> = {
  [OptimizationScenario.CONSERVATIVE]: 0.6,
  [OptimizationScenario.MODERATE]: 1.0,
  [OptimizationScenario.AGGRESSIVE]: 1.4,
  [OptimizationScenario.CUSTOM]: 1.0,
};

// ─── SVG Chart: Donut ─────────────────────────────────────────────────────────

interface DonutSegment {
  label: string;
  value: number;
  color: string;
}

function DonutChart({
  segments,
  title,
  centerValue,
}: {
  segments: DonutSegment[];
  title: string;
  centerValue: string;
}) {
  const [hovered, setHovered] = useState<number | null>(null);
  const radius = 70;
  const cx = 90;
  const cy = 90;
  const circumference = 2 * Math.PI * radius;
  const total = segments.reduce((s, seg) => s + seg.value, 0);

  let offset = 0;
  const arcs = segments.map((seg, i) => {
    const fraction = seg.value / total;
    const dash = fraction * circumference;
    const gap = circumference - dash;
    const arc = { ...seg, dash, gap, offset, index: i };
    offset += dash;
    return arc;
  });

  return (
    <div className="flex flex-col items-center">
      <p className="text-xs text-[#92A5A8] mb-3 uppercase tracking-wide">{title}</p>
      <div className="relative">
        <svg width="180" height="180" viewBox="0 0 180 180">
          {arcs.map((arc, i) => (
            <circle
              key={i}
              cx={cx}
              cy={cy}
              r={radius}
              fill="none"
              stroke={arc.color}
              strokeWidth={hovered === i ? 26 : 22}
              strokeDasharray={`${arc.dash} ${arc.gap}`}
              strokeDashoffset={-arc.offset}
              transform={`rotate(-90 ${cx} ${cy})`}
              style={{ transition: "stroke-width 0.2s", cursor: "pointer" }}
              onMouseEnter={() => setHovered(i)}
              onMouseLeave={() => setHovered(null)}
            />
          ))}
          <text x={cx} y={cy - 8} textAnchor="middle" fill="#FCFFFF" fontSize="18" fontWeight="bold">
            {centerValue}
          </text>
          <text x={cx} y={cy + 12} textAnchor="middle" fill="#92A5A8" fontSize="10">
            Total
          </text>
        </svg>
        {hovered !== null && (
          <div
            className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-none
              bg-[#161E22] border border-[#2A3338] rounded-lg p-2 text-center min-w-[80px]"
          >
            <p className="text-[#FCFFFF] text-sm font-semibold">{arcs[hovered].value.toFixed(1)}%</p>
            <p className="text-[#92A5A8] text-xs">{arcs[hovered].label}</p>
          </div>
        )}
      </div>
      <div className="space-y-1.5 mt-3 w-full max-w-[160px]">
        {segments.map((seg, i) => (
          <div key={i} className="flex items-center justify-between text-xs">
            <div className="flex items-center gap-1.5">
              <div className="w-2 h-2 rounded-full" style={{ background: seg.color }} />
              <span className="text-[#92A5A8] truncate max-w-[90px]">{seg.label}</span>
            </div>
            <span className="text-[#FCFFFF] font-medium">{seg.value.toFixed(1)}%</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── SVG Chart: Comparison Bar ────────────────────────────────────────────────

function ComparisonBarChart({ allocations }: { allocations: AssetAllocation[] }) {
  const [hovered, setHovered] = useState<number | null>(null);
  const maxVal = Math.max(...allocations.flatMap((a) => [a.currentPercentage, a.recommendedPercentage]));
  const barH = 16;
  const gap = 8;
  const groupH = barH * 2 + gap + 20;
  const labelW = 60;
  const chartW = 300;
  const svgH = allocations.length * groupH + 20;

  return (
    <div className="overflow-x-auto">
      <svg width="100%" viewBox={`0 0 ${labelW + chartW + 60} ${svgH}`} className="min-w-[380px]">
        {allocations.map((a, i) => {
          const y = i * groupH + 10;
          const curW = (a.currentPercentage / maxVal) * chartW;
          const recW = (a.recommendedPercentage / maxVal) * chartW;
          const delta = a.recommendedPercentage - a.currentPercentage;
          const isHov = hovered === i;

          return (
            <g
              key={i}
              onMouseEnter={() => setHovered(i)}
              onMouseLeave={() => setHovered(null)}
              style={{ cursor: "pointer" }}
            >
              {/* Asset label */}
              <text x={labelW - 6} y={y + barH} textAnchor="end" fill="#92A5A8" fontSize="10">
                {a.assetSymbol}
              </text>

              {/* Current bar */}
              <rect
                x={labelW}
                y={y}
                width={curW}
                height={barH}
                rx={4}
                fill="#1C252A"
                stroke="#33C5E0"
                strokeWidth="1"
                opacity={isHov ? 1 : 0.8}
              />
              <text x={labelW + curW + 4} y={y + barH - 3} fill="#92A5A8" fontSize="9">
                {a.currentPercentage.toFixed(1)}%
              </text>

              {/* Recommended bar */}
              <rect
                x={labelW}
                y={y + barH + gap}
                width={recW}
                height={barH}
                rx={4}
                fill="#33C5E0"
                opacity={isHov ? 1 : 0.85}
              />
              <text x={labelW + recW + 4} y={y + barH * 2 + gap - 3} fill="#33C5E0" fontSize="9">
                {a.recommendedPercentage.toFixed(1)}%
              </text>

              {/* Delta badge */}
              <text
                x={labelW + chartW + 8}
                y={y + barH + gap / 2 + 5}
                fill={delta >= 0 ? "#22c55e" : "#ef4444"}
                fontSize="10"
                fontWeight="bold"
              >
                {delta >= 0 ? "+" : ""}{delta.toFixed(1)}%
              </text>
            </g>
          );
        })}

        {/* Legend */}
        <g transform={`translate(${labelW}, ${svgH - 14})`}>
          <rect width={10} height={10} rx={2} fill="#1C252A" stroke="#33C5E0" strokeWidth="1" />
          <text x={14} y={9} fill="#92A5A8" fontSize="9">Current</text>
          <rect x={70} width={10} height={10} rx={2} fill="#33C5E0" />
          <text x={84} y={9} fill="#92A5A8" fontSize="9">Recommended</text>
        </g>
      </svg>

      {hovered !== null && (
        <div className="mt-2 p-3 bg-[#1C252A] rounded-lg border border-[#2A3338]">
          <p className="text-[#FCFFFF] text-sm font-medium mb-1">
            {allocations[hovered].assetSymbol} on {allocations[hovered].chain}
          </p>
          <p className="text-[#92A5A8] text-xs">{allocations[hovered].adjustmentReason}</p>
          <p className="text-[#33C5E0] text-xs mt-1">{allocations[hovered].expectedImpact}</p>
        </div>
      )}
    </div>
  );
}

// ─── SVG Chart: Projection Line ───────────────────────────────────────────────

interface ProjectionPoint {
  year: number;
  current: number;
  optimized: number;
  conservative?: number;
  aggressive?: number;
}

function ProjectionLineChart({
  points,
  scenario,
}: {
  points: ProjectionPoint[];
  scenario: OptimizationScenario;
}) {
  const [hovered, setHovered] = useState<number | null>(null);
  const W = 500;
  const H = 220;
  const pad = { top: 20, right: 60, bottom: 40, left: 70 };
  const chartW = W - pad.left - pad.right;
  const chartH = H - pad.top - pad.bottom;

  const allVals = points.flatMap((p) => [
    p.current,
    p.optimized,
    p.conservative ?? p.optimized,
    p.aggressive ?? p.optimized,
  ]);
  const minV = Math.min(...allVals) * 0.9;
  const maxV = Math.max(...allVals) * 1.05;

  const xScale = (i: number) => (i / (points.length - 1)) * chartW + pad.left;
  const yScale = (v: number) => H - pad.bottom - ((v - minV) / (maxV - minV)) * chartH;

  const linePath = (key: keyof ProjectionPoint) =>
    points
      .map((p, i) => `${i === 0 ? "M" : "L"} ${xScale(i).toFixed(1)} ${yScale(p[key] as number).toFixed(1)}`)
      .join(" ");

  const formatK = (v: number) =>
    v >= 1_000_000 ? `$${(v / 1_000_000).toFixed(1)}M` : `$${(v / 1000).toFixed(0)}K`;

  const yTicks = 5;
  const yTickVals = Array.from({ length: yTicks }, (_, i) => minV + ((maxV - minV) * i) / (yTicks - 1));

  return (
    <div className="overflow-x-auto">
      <svg
        width="100%"
        viewBox={`0 0 ${W} ${H}`}
        className="min-w-[380px]"
        onMouseLeave={() => setHovered(null)}
      >
        {/* Grid lines */}
        {yTickVals.map((v, i) => (
          <g key={i}>
            <line
              x1={pad.left}
              x2={W - pad.right}
              y1={yScale(v)}
              y2={yScale(v)}
              stroke="#1C252A"
              strokeWidth="1"
            />
            <text x={pad.left - 6} y={yScale(v) + 4} textAnchor="end" fill="#92A5A8" fontSize="9">
              {formatK(v)}
            </text>
          </g>
        ))}

        {/* X-axis labels */}
        {points.map((p, i) => (
          <text
            key={i}
            x={xScale(i)}
            y={H - pad.bottom + 14}
            textAnchor="middle"
            fill="#92A5A8"
            fontSize="9"
          >
            Yr {p.year}
          </text>
        ))}

        {/* Monte Carlo band (aggressive - conservative) */}
        {points[0].aggressive !== undefined && (
          <path
            d={`${points.map((p, i) => `${i === 0 ? "M" : "L"} ${xScale(i).toFixed(1)} ${yScale(p.aggressive!).toFixed(1)}`).join(" ")} ${points.slice().reverse().map((p, i) => `${i === 0 ? "L" : "L"} ${xScale(points.length - 1 - i).toFixed(1)} ${yScale(p.conservative!).toFixed(1)}`).join(" ")} Z`}
            fill="#33C5E0"
            opacity="0.07"
          />
        )}

        {/* Conservative line */}
        {points[0].conservative !== undefined && (
          <path d={linePath("conservative")} fill="none" stroke="#D4A017" strokeWidth="1.5" strokeDasharray="4 3" />
        )}

        {/* Aggressive line */}
        {points[0].aggressive !== undefined && (
          <path d={linePath("aggressive")} fill="none" stroke="#22c55e" strokeWidth="1.5" strokeDasharray="4 3" />
        )}

        {/* Current baseline */}
        <path d={linePath("current")} fill="none" stroke="#4A5558" strokeWidth="2" />

        {/* Optimized line */}
        <path d={linePath("optimized")} fill="none" stroke="#33C5E0" strokeWidth="2.5" />

        {/* Hover dots */}
        {points.map((p, i) => (
          <circle
            key={i}
            cx={xScale(i)}
            cy={yScale(p.optimized)}
            r={hovered === i ? 6 : 4}
            fill="#33C5E0"
            stroke="#161E22"
            strokeWidth="2"
            style={{ cursor: "pointer", transition: "r 0.15s" }}
            onMouseEnter={() => setHovered(i)}
          />
        ))}

        {/* Hover tooltip */}
        {hovered !== null && (() => {
          const p = points[hovered];
          const tx = xScale(hovered);
          const ty = yScale(p.optimized);
          const boxX = tx > W - pad.right - 110 ? tx - 120 : tx + 10;
          return (
            <g>
              <line x1={tx} y1={pad.top} x2={tx} y2={H - pad.bottom} stroke="#33C5E0" strokeWidth="1" opacity="0.4" strokeDasharray="3 3" />
              <rect x={boxX} y={ty - 30} width={110} height={62} rx={6} fill="#1C252A" stroke="#33C5E0" strokeWidth="1" />
              <text x={boxX + 8} y={ty - 12} fill="#FCFFFF" fontSize="10" fontWeight="bold">Year {p.year}</text>
              <text x={boxX + 8} y={ty + 4} fill="#33C5E0" fontSize="9">Optimized: {formatK(p.optimized)}</text>
              <text x={boxX + 8} y={ty + 18} fill="#4A5558" fontSize="9">Current: {formatK(p.current)}</text>
              {p.conservative && (
                <text x={boxX + 8} y={ty + 32} fill="#D4A017" fontSize="9">Conservative: {formatK(p.conservative)}</text>
              )}
            </g>
          );
        })()}

        {/* Legend */}
        <g transform={`translate(${pad.left}, ${H - 8})`}>
          <rect width={8} height={3} y={-1.5} fill="#33C5E0" />
          <text x={12} y={4} fill="#92A5A8" fontSize="8">Optimized</text>
          <rect x={70} width={8} height={3} y={-1.5} fill="#4A5558" />
          <text x={82} y={4} fill="#92A5A8" fontSize="8">Current</text>
          {points[0].conservative && (
            <>
              <rect x={130} width={8} height={3} y={-1.5} fill="#D4A017" />
              <text x={142} y={4} fill="#92A5A8" fontSize="8">Conservative</text>
              <rect x={210} width={8} height={3} y={-1.5} fill="#22c55e" />
              <text x={222} y={4} fill="#92A5A8" fontSize="8">Aggressive</text>
            </>
          )}
        </g>
      </svg>
    </div>
  );
}

// ─── Confidence Arc ────────────────────────────────────────────────────────────

function ConfidenceArc({ score }: { score: number }) {
  const r = 52;
  const cx = 70;
  const cy = 70;
  const circ = 2 * Math.PI * r;
  const filled = (score / 100) * circ * 0.75;
  const color = score >= 80 ? "#22c55e" : score >= 60 ? "#D4A017" : "#ef4444";

  return (
    <svg width="140" height="100" viewBox="0 0 140 100">
      <circle cx={cx} cy={cy} r={r} fill="none" stroke="#1C252A" strokeWidth="10" strokeDasharray={`${circ * 0.75} ${circ}`} strokeDashoffset={circ * 0.125} strokeLinecap="round" />
      <circle cx={cx} cy={cy} r={r} fill="none" stroke={color} strokeWidth="10" strokeDasharray={`${filled} ${circ}`} strokeDashoffset={circ * 0.125} strokeLinecap="round" style={{ transition: "stroke-dasharray 0.8s ease" }} />
      <text x={cx} y={cy - 6} textAnchor="middle" fill="#FCFFFF" fontSize="22" fontWeight="bold">{score}</text>
      <text x={cx} y={cy + 12} textAnchor="middle" fill="#92A5A8" fontSize="10">/ 100</text>
    </svg>
  );
}

// ─── Tooltip ──────────────────────────────────────────────────────────────────

function Tooltip({ text, children }: { text: string; children: React.ReactNode }) {
  const [show, setShow] = useState(false);
  return (
    <span className="relative inline-flex items-center" onMouseEnter={() => setShow(true)} onMouseLeave={() => setShow(false)}>
      {children}
      {show && (
        <span className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-48 bg-[#1C252A] border border-[#2A3338] text-[#92A5A8] text-xs rounded-lg px-3 py-2 z-50 pointer-events-none shadow-xl">
          {text}
        </span>
      )}
    </span>
  );
}

// ─── Reject Modal ─────────────────────────────────────────────────────────────

function RejectModal({
  onConfirm,
  onCancel,
  reason,
  onReasonChange,
}: {
  onConfirm: () => void;
  onCancel: () => void;
  reason: string;
  onReasonChange: (v: string) => void;
}) {
  const PRESET_REASONS = [
    "I prefer higher risk tolerance",
    "I want to maintain current allocation",
    "I need more time to review",
    "The recommendations don't match my goals",
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm" onClick={onCancel}>
      <motion.div
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.9, opacity: 0 }}
        className="bg-[#161E22] border border-[#2A3338] rounded-2xl p-6 w-full max-w-md shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-full bg-red-500/10 flex items-center justify-center">
            <X size={18} className="text-red-400" />
          </div>
          <div>
            <h3 className="text-[#FCFFFF] font-semibold">Reject Recommendation</h3>
            <p className="text-[#92A5A8] text-xs">This helps AI learn your preferences</p>
          </div>
        </div>

        <div className="mb-4">
          <p className="text-[#92A5A8] text-xs mb-2">Select a reason or write your own:</p>
          <div className="grid grid-cols-1 gap-2 mb-3">
            {PRESET_REASONS.map((r) => (
              <button
                key={r}
                onClick={() => onReasonChange(r)}
                className={`text-left text-xs px-3 py-2 rounded-lg border transition-colors ${reason === r ? "border-[#33C5E0] bg-[#33C5E0]/10 text-[#33C5E0]" : "border-[#2A3338] text-[#92A5A8] hover:border-[#4A5558]"}`}
              >
                {r}
              </button>
            ))}
          </div>
          <textarea
            value={reason}
            onChange={(e) => onReasonChange(e.target.value)}
            placeholder="Or describe your specific reason..."
            className="w-full bg-[#1C252A] border border-[#2A3338] rounded-lg px-3 py-2 text-[#FCFFFF] text-sm placeholder:text-[#4A5558] focus:outline-none focus:border-[#33C5E0] resize-none"
            rows={3}
          />
        </div>

        <div className="flex gap-3">
          <button onClick={onCancel} className="flex-1 py-2.5 rounded-xl border border-[#2A3338] text-[#92A5A8] text-sm hover:bg-[#1C252A] transition-colors">
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={!reason.trim()}
            className="flex-1 py-2.5 rounded-xl bg-red-500/10 border border-red-500/30 text-red-400 text-sm hover:bg-red-500/20 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Confirm Rejection
          </button>
        </div>
      </motion.div>
    </div>
  );
}

// ─── Accept Confirm Modal ─────────────────────────────────────────────────────

function AcceptConfirmModal({
  recommendation,
  onConfirm,
  onCancel,
}: {
  recommendation: OptimizationRecommendation;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm" onClick={onCancel}>
      <motion.div
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.9, opacity: 0 }}
        className="bg-[#161E22] border border-[#2A3338] rounded-2xl p-6 w-full max-w-md shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-full bg-[#33C5E0]/10 flex items-center justify-center">
            <Check size={18} className="text-[#33C5E0]" />
          </div>
          <div>
            <h3 className="text-[#FCFFFF] font-semibold">Accept AI Recommendation</h3>
            <p className="text-[#92A5A8] text-xs">This will rebalance your inheritance plan</p>
          </div>
        </div>

        <div className="bg-[#1C252A] rounded-xl p-4 mb-4 space-y-2">
          <div className="flex justify-between text-sm">
            <span className="text-[#92A5A8]">Expected Annual Return</span>
            <span className="text-[#22c55e] font-semibold">+{recommendation.expectedReturn.toFixed(1)}%</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[#92A5A8]">Confidence Score</span>
            <span className="text-[#FCFFFF] font-semibold">{recommendation.confidenceScore}/100</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[#92A5A8]">Risk Score</span>
            <span className="text-[#D4A017] font-semibold">{recommendation.riskScore}/100</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[#92A5A8]">Assets Affected</span>
            <span className="text-[#FCFFFF] font-semibold">{recommendation.recommendedAllocations.length}</span>
          </div>
        </div>

        <div className="flex items-start gap-2 bg-[#D4A017]/10 border border-[#D4A017]/20 rounded-lg p-3 mb-4">
          <AlertTriangle size={14} className="text-[#D4A017] mt-0.5 shrink-0" />
          <p className="text-[#D4A017] text-xs">
            This action will modify your inheritance plan&apos;s asset allocations. Ensure you understand the implications before proceeding.
          </p>
        </div>

        <div className="flex gap-3">
          <button onClick={onCancel} className="flex-1 py-2.5 rounded-xl border border-[#2A3338] text-[#92A5A8] text-sm hover:bg-[#1C252A] transition-colors">
            Cancel
          </button>
          <button onClick={onConfirm} className="flex-1 py-2.5 rounded-xl bg-[#33C5E0] text-[#161E22] text-sm font-semibold hover:bg-[#2AB8D3] transition-colors">
            Apply Recommendations
          </button>
        </div>
      </motion.div>
    </div>
  );
}

// ─── Allocation Slider ─────────────────────────────────────────────────────────

function AllocationSlider({
  allocation,
  value,
  onChange,
  color,
}: {
  allocation: AssetAllocation;
  value: number;
  onChange: (v: number) => void;
  color: string;
}) {
  const delta = value - allocation.currentPercentage;

  return (
    <div className="bg-[#1C252A] rounded-xl p-4 border border-[#2A3338]">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full" style={{ background: color }} />
          <span className="text-[#FCFFFF] text-sm font-medium">{allocation.assetSymbol}</span>
          <span className="text-[#92A5A8] text-xs">{allocation.chain}</span>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-[#92A5A8] text-xs">Current: {allocation.currentPercentage.toFixed(1)}%</span>
          <span className="text-[#FCFFFF] text-sm font-bold">{value.toFixed(1)}%</span>
          <span className={`text-xs font-medium ${delta >= 0 ? "text-[#22c55e]" : "text-red-400"}`}>
            {delta >= 0 ? "+" : ""}{delta.toFixed(1)}%
          </span>
        </div>
      </div>
      <input
        type="range"
        min={0}
        max={100}
        step={0.5}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full h-2 rounded-full appearance-none cursor-pointer"
        style={{
          background: `linear-gradient(to right, ${color} 0%, ${color} ${value}%, #1C252A ${value}%, #1C252A 100%)`,
          WebkitAppearance: "none",
        }}
        aria-label={`Allocation for ${allocation.assetSymbol}`}
      />
      <div className="flex justify-between text-xs text-[#4A5558] mt-1">
        <span>0%</span>
        <span className="text-[#92A5A8] text-center text-xs">{allocation.adjustmentReason}</span>
        <span>100%</span>
      </div>
    </div>
  );
}

// ─── Risk Badge ────────────────────────────────────────────────────────────────

function RiskBadge({ score }: { score: number }) {
  const label = score < 33 ? "Low" : score < 66 ? "Medium" : "High";
  const color = score < 33 ? "text-[#22c55e] bg-[#22c55e]/10 border-[#22c55e]/20" : score < 66 ? "text-[#D4A017] bg-[#D4A017]/10 border-[#D4A017]/20" : "text-red-400 bg-red-400/10 border-red-400/20";
  return (
    <span className={`inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-medium border ${color}`}>
      <Shield size={10} />
      {label} Risk
    </span>
  );
}

// ─── Main Dashboard ────────────────────────────────────────────────────────────

export const AIOptimizationDashboard: React.FC<AIOptimizationDashboardProps> = ({
  inheritancePlanId,
  currentAllocations,
  optimizationRecommendations,
  onAcceptRecommendation,
  onRejectRecommendation,
  onCustomizeRecommendation,
}) => {
  const rec = optimizationRecommendations;

  const [state, setState] = useState<DashboardState>({
    customAllocations: rec.recommendedAllocations.map((a) => ({ ...a })),
    selectedScenario: OptimizationScenario.MODERATE,
    showDetails: false,
    activeTab: "overview",
    showRejectModal: false,
    showAcceptConfirm: false,
    rejectReason: "",
    tooltipId: null,
    expandedReasoning: false,
  });

  const setPartial = useCallback((patch: Partial<DashboardState>) => {
    setState((prev) => ({ ...prev, ...patch }));
  }, []);

  // Build projection data from issue's projectedOutcomes
  const projectionPoints = useMemo<ProjectionPoint[]>(() => {
    const mult = scenarioMultipliers[state.selectedScenario];
    const base = 100_000;
    return [
      { year: 0, current: base, optimized: base, conservative: base, aggressive: base },
      {
        year: 1,
        current: base * 1.04,
        optimized: base * (1 + rec.expectedReturn / 100) * mult,
        conservative: base * (1 + (rec.expectedReturn / 100) * 0.5),
        aggressive: base * (1 + (rec.expectedReturn / 100) * 1.6),
      },
      {
        year: 5,
        current: rec.projectedOutcomes.estimatedValue5Year * 0.72,
        optimized: rec.projectedOutcomes.estimatedValue5Year * mult,
        conservative: rec.projectedOutcomes.estimatedValue5Year * 0.65,
        aggressive: rec.projectedOutcomes.estimatedValue5Year * 1.35,
      },
      {
        year: 10,
        current: rec.projectedOutcomes.estimatedValue10Year * 0.68,
        optimized: rec.projectedOutcomes.estimatedValue10Year * mult,
        conservative: rec.projectedOutcomes.estimatedValue10Year * 0.55,
        aggressive: rec.projectedOutcomes.estimatedValue10Year * 1.5,
      },
    ];
  }, [rec, state.selectedScenario]);

  // Build donut segments
  const currentSegments = useMemo(
    () =>
      currentAllocations.map((a, i) => ({
        label: a.assetSymbol,
        value: a.currentPercentage,
        color: CHART_COLORS[i % CHART_COLORS.length],
      })),
    [currentAllocations]
  );

  const recommendedSegments = useMemo(
    () =>
      rec.recommendedAllocations.map((a, i) => ({
        label: a.assetSymbol,
        value: a.recommendedPercentage,
        color: CHART_COLORS[i % CHART_COLORS.length],
      })),
    [rec.recommendedAllocations]
  );

  // Custom allocations total
  const customTotal = state.customAllocations.reduce((s, a) => s + a.recommendedPercentage, 0);

  const handleCustomChange = useCallback((index: number, value: number) => {
    setPartial({
      customAllocations: state.customAllocations.map((a, i) =>
        i === index ? { ...a, recommendedPercentage: value } : a
      ),
      selectedScenario: OptimizationScenario.CUSTOM,
    });
  }, [state.customAllocations, setPartial]);

  const handleAcceptConfirm = useCallback(() => {
    onAcceptRecommendation(rec);
    setPartial({ showAcceptConfirm: false });
  }, [rec, onAcceptRecommendation, setPartial]);

  const handleRejectConfirm = useCallback(() => {
    onRejectRecommendation(state.rejectReason);
    setPartial({ showRejectModal: false, rejectReason: "" });
  }, [state.rejectReason, onRejectRecommendation, setPartial]);

  const handleApplyCustom = useCallback(() => {
    onCustomizeRecommendation(state.customAllocations);
  }, [state.customAllocations, onCustomizeRecommendation]);

  const formatDate = (dateStr: string) => {
    try {
      return new Date(dateStr).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return dateStr;
    }
  };

  const tabs = [
    { id: "overview" as const, label: "Overview", icon: Brain },
    { id: "allocation" as const, label: "Allocation", icon: BarChart2 },
    { id: "projections" as const, label: "Projections", icon: TrendingUp },
    { id: "customize" as const, label: "Customize", icon: Sliders },
  ];

  return (
    <div className="min-h-screen text-white p-4 sm:p-6">
      {/* ── Header ── */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <div className="w-8 h-8 rounded-lg bg-[#33C5E0]/10 flex items-center justify-center">
              <Brain size={18} className="text-[#33C5E0]" />
            </div>
            <h1 className="text-2xl font-semibold text-[#FCFFFF]">AI Optimization</h1>
          </div>
          <p className="text-[#92A5A8] text-sm">
            Plan #{inheritancePlanId} &mdash; Generated {formatDate(rec.generatedAt)}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-[#92A5A8] text-xs flex items-center gap-1">
            <Clock size={12} />
            Auto-refreshes daily
          </span>
          <button className="p-2 rounded-lg border border-[#2A3338] text-[#92A5A8] hover:text-[#FCFFFF] hover:bg-[#1C252A] transition-colors" aria-label="Refresh recommendations">
            <RefreshCw size={16} />
          </button>
        </div>
      </div>

      {/* ── Summary Cards ── */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        {/* Confidence */}
        <div className="col-span-2 sm:col-span-1 bg-[#1C252A] rounded-xl p-4 border border-[#2A3338] flex flex-col items-center">
          <div className="flex items-center gap-1.5 mb-2">
            <Target size={14} className="text-[#33C5E0]" />
            <span className="text-[#92A5A8] text-xs font-medium uppercase tracking-wide">Confidence</span>
            <Tooltip text="How confident the AI is in this recommendation based on historical data and market analysis.">
              <Info size={12} className="text-[#4A5558] cursor-help" />
            </Tooltip>
          </div>
          <ConfidenceArc score={rec.confidenceScore} />
        </div>

        {/* Expected Return */}
        <div className="bg-[#1C252A] rounded-xl p-4 border border-[#2A3338]">
          <div className="flex items-center gap-1.5 mb-3">
            <TrendingUp size={14} className="text-[#22c55e]" />
            <span className="text-[#92A5A8] text-xs font-medium uppercase tracking-wide">Est. Annual Return</span>
          </div>
          <p className="text-3xl font-bold text-[#22c55e]">+{rec.expectedReturn.toFixed(1)}%</p>
          <p className="text-[#92A5A8] text-xs mt-1">vs {(rec.expectedReturn * 0.72).toFixed(1)}% current</p>
          <div className="mt-3 h-1.5 bg-[#161E22] rounded-full overflow-hidden">
            <div className="h-full bg-[#22c55e] rounded-full" style={{ width: `${Math.min(rec.expectedReturn * 5, 100)}%`, transition: "width 0.8s ease" }} />
          </div>
        </div>

        {/* Risk Score */}
        <div className="bg-[#1C252A] rounded-xl p-4 border border-[#2A3338]">
          <div className="flex items-center gap-1.5 mb-3">
            <Shield size={14} className="text-[#D4A017]" />
            <span className="text-[#92A5A8] text-xs font-medium uppercase tracking-wide">Risk Score</span>
            <Tooltip text="Risk score ranges from 0 (no risk) to 100 (maximum risk). Lower is safer.">
              <Info size={12} className="text-[#4A5558] cursor-help" />
            </Tooltip>
          </div>
          <p className="text-3xl font-bold text-[#D4A017]">{rec.riskScore}</p>
          <div className="mt-2"><RiskBadge score={rec.riskScore} /></div>
          <div className="mt-3 h-1.5 bg-[#161E22] rounded-full overflow-hidden">
            <div className="h-full rounded-full" style={{ width: `${rec.riskScore}%`, background: rec.riskScore < 33 ? "#22c55e" : rec.riskScore < 66 ? "#D4A017" : "#ef4444", transition: "width 0.8s ease" }} />
          </div>
        </div>

        {/* 10-Year Value */}
        <div className="bg-[#1C252A] rounded-xl p-4 border border-[#2A3338]">
          <div className="flex items-center gap-1.5 mb-3">
            <Activity size={14} className="text-[#a855f7]" />
            <span className="text-[#92A5A8] text-xs font-medium uppercase tracking-wide">10-Year Est. Value</span>
          </div>
          <p className="text-2xl font-bold text-[#FCFFFF]">
            ${(rec.projectedOutcomes.estimatedValue10Year / 1000).toFixed(0)}K
          </p>
          <div className="mt-2 space-y-1 text-xs">
            <div className="flex justify-between">
              <span className="text-[#92A5A8]">1 Year</span>
              <span className="text-[#FCFFFF]">${(rec.projectedOutcomes.estimatedValue1Year / 1000).toFixed(0)}K</span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#92A5A8]">5 Years</span>
              <span className="text-[#FCFFFF]">${(rec.projectedOutcomes.estimatedValue5Year / 1000).toFixed(0)}K</span>
            </div>
          </div>
        </div>
      </div>

      {/* ── Tabs ── */}
      <div className="flex gap-1 mb-6 bg-[#1C252A] rounded-xl p-1 border border-[#2A3338] overflow-x-auto">
        {tabs.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setPartial({ activeTab: id })}
            className={`flex items-center gap-2 flex-1 py-2.5 px-3 rounded-lg text-sm font-medium whitespace-nowrap transition-all min-w-[100px] justify-center ${
              state.activeTab === id
                ? "bg-[#33C5E0] text-[#161E22]"
                : "text-[#92A5A8] hover:text-[#FCFFFF] hover:bg-[#2A3338]"
            }`}
          >
            <Icon size={15} />
            {label}
          </button>
        ))}
      </div>

      {/* ── Tab Content ── */}
      <AnimatePresence mode="wait">
        {/* OVERVIEW TAB */}
        {state.activeTab === "overview" && (
          <motion.div key="overview" initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -8 }} transition={{ duration: 0.2 }} className="space-y-6">

            {/* Reasoning Panel */}
            <div className="bg-[#1C252A] rounded-xl border border-[#2A3338] overflow-hidden">
              <button
                onClick={() => setPartial({ expandedReasoning: !state.expandedReasoning })}
                className="w-full flex items-center justify-between p-5 hover:bg-[#2A3338] transition-colors"
              >
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 rounded-lg bg-[#33C5E0]/10 flex items-center justify-center">
                    <Brain size={16} className="text-[#33C5E0]" />
                  </div>
                  <div className="text-left">
                    <h3 className="text-[#FCFFFF] font-medium">AI Reasoning</h3>
                    <p className="text-[#92A5A8] text-xs">Why this recommendation was generated</p>
                  </div>
                </div>
                {state.expandedReasoning ? <ChevronUp size={18} className="text-[#92A5A8]" /> : <ChevronDown size={18} className="text-[#92A5A8]" />}
              </button>
              <AnimatePresence>
                {state.expandedReasoning && (
                  <motion.div initial={{ height: 0 }} animate={{ height: "auto" }} exit={{ height: 0 }} className="overflow-hidden">
                    <div className="px-5 pb-5">
                      <p className="text-[#FCFFFF] text-sm leading-relaxed">{rec.reasoning}</p>
                      <div className="mt-4 grid grid-cols-2 gap-3">
                        <div className="bg-[#161E22] rounded-lg p-3">
                          <p className="text-[#92A5A8] text-xs mb-1">Sharpe Ratio</p>
                          <p className="text-[#FCFFFF] font-semibold">{rec.projectedOutcomes.riskMetrics.sharpeRatio.toFixed(2)}</p>
                          <p className="text-[#92A5A8] text-xs mt-1">Risk-adjusted return</p>
                        </div>
                        <div className="bg-[#161E22] rounded-lg p-3">
                          <p className="text-[#92A5A8] text-xs mb-1">Volatility</p>
                          <p className="text-[#FCFFFF] font-semibold">{rec.projectedOutcomes.riskMetrics.volatility.toFixed(1)}%</p>
                          <p className="text-[#92A5A8] text-xs mt-1">Annualized std dev</p>
                        </div>
                        <div className="bg-[#161E22] rounded-lg p-3">
                          <p className="text-[#92A5A8] text-xs mb-1">Max Drawdown</p>
                          <p className="text-red-400 font-semibold">-{rec.projectedOutcomes.riskMetrics.maxDrawdown.toFixed(1)}%</p>
                          <p className="text-[#92A5A8] text-xs mt-1">Worst-case scenario</p>
                        </div>
                        <div className="bg-[#161E22] rounded-lg p-3">
                          <p className="text-[#92A5A8] text-xs mb-1">Value at Risk (95%)</p>
                          <p className="text-[#D4A017] font-semibold">-{rec.projectedOutcomes.riskMetrics.valueAtRisk.toFixed(1)}%</p>
                          <p className="text-[#92A5A8] text-xs mt-1">Monthly VaR estimate</p>
                        </div>
                      </div>
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>

            {/* Allocation changes summary */}
            <div className="bg-[#1C252A] rounded-xl p-5 border border-[#2A3338]">
              <h3 className="text-[#FCFFFF] font-medium mb-4 flex items-center gap-2">
                <BarChart2 size={16} className="text-[#33C5E0]" />
                Key Changes
              </h3>
              <div className="space-y-3">
                {rec.recommendedAllocations
                  .map((a) => ({ ...a, delta: a.recommendedPercentage - a.currentPercentage }))
                  .sort((a, b) => Math.abs(b.delta) - Math.abs(a.delta))
                  .slice(0, 4)
                  .map((a, i) => (
                    <div key={i} className="flex items-start gap-3 p-3 bg-[#161E22] rounded-lg">
                      <div className={`mt-0.5 w-6 h-6 rounded-full flex items-center justify-center shrink-0 ${a.delta >= 0 ? "bg-[#22c55e]/10" : "bg-red-500/10"}`}>
                        {a.delta >= 0 ? <TrendingUp size={12} className="text-[#22c55e]" /> : <TrendingDown size={12} className="text-red-400" />}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between">
                          <span className="text-[#FCFFFF] text-sm font-medium">{a.assetSymbol}</span>
                          <span className={`text-sm font-bold ${a.delta >= 0 ? "text-[#22c55e]" : "text-red-400"}`}>
                            {a.delta >= 0 ? "+" : ""}{a.delta.toFixed(1)}%
                          </span>
                        </div>
                        <p className="text-[#92A5A8] text-xs mt-0.5 truncate">{a.adjustmentReason}</p>
                        <p className="text-[#33C5E0] text-xs mt-0.5">{a.expectedImpact}</p>
                      </div>
                    </div>
                  ))}
              </div>
            </div>

            {/* Educational Tips */}
            <div className="bg-[#33C5E0]/5 border border-[#33C5E0]/20 rounded-xl p-4">
              <h4 className="text-[#33C5E0] text-sm font-medium mb-2 flex items-center gap-2">
                <Info size={14} />
                Best Practices
              </h4>
              <ul className="space-y-1.5 text-xs text-[#92A5A8]">
                <li className="flex items-start gap-2"><span className="text-[#33C5E0] mt-0.5">•</span>Review recommendations quarterly for optimal performance</li>
                <li className="flex items-start gap-2"><span className="text-[#33C5E0] mt-0.5">•</span>A higher confidence score means more historical data supports the recommendation</li>
                <li className="flex items-start gap-2"><span className="text-[#33C5E0] mt-0.5">•</span>Consider using the Conservative scenario if you have low risk tolerance</li>
                <li className="flex items-start gap-2"><span className="text-[#33C5E0] mt-0.5">•</span>Custom allocations allow fine-grained control over your portfolio</li>
              </ul>
            </div>
          </motion.div>
        )}

        {/* ALLOCATION TAB */}
        {state.activeTab === "allocation" && (
          <motion.div key="allocation" initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -8 }} transition={{ duration: 0.2 }} className="space-y-6">

            {/* Side-by-side donut charts */}
            <div className="bg-[#1C252A] rounded-xl p-5 border border-[#2A3338]">
              <h3 className="text-[#FCFFFF] font-medium mb-5">Portfolio Allocation Comparison</h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-8 justify-items-center">
                <DonutChart segments={currentSegments} title="Current Allocation" centerValue="100%" />
                <DonutChart segments={recommendedSegments} title="Recommended Allocation" centerValue="100%" />
              </div>
            </div>

            {/* Bar chart comparison */}
            <div className="bg-[#1C252A] rounded-xl p-5 border border-[#2A3338]">
              <h3 className="text-[#FCFFFF] font-medium mb-4">Asset-by-Asset Comparison</h3>
              <ComparisonBarChart allocations={rec.recommendedAllocations} />
            </div>

            {/* Allocation table */}
            <div className="bg-[#1C252A] rounded-xl border border-[#2A3338] overflow-hidden">
              <div className="p-5 border-b border-[#2A3338]">
                <h3 className="text-[#FCFFFF] font-medium">Detailed Allocation Table</h3>
              </div>
              <div className="overflow-x-auto">
                <table className="w-full min-w-[600px]">
                  <thead>
                    <tr className="text-[#92A5A8] text-xs uppercase tracking-wide border-b border-[#2A3338]">
                      <th className="text-left px-5 py-3">Asset</th>
                      <th className="text-right px-5 py-3">Current</th>
                      <th className="text-right px-5 py-3">Recommended</th>
                      <th className="text-right px-5 py-3">Change</th>
                      <th className="text-left px-5 py-3">Impact</th>
                    </tr>
                  </thead>
                  <tbody>
                    {rec.recommendedAllocations.map((a, i) => {
                      const delta = a.recommendedPercentage - a.currentPercentage;
                      return (
                        <tr key={i} className="border-b border-[#2A3338] hover:bg-[#2A3338]/50 transition-colors">
                          <td className="px-5 py-3">
                            <div className="flex items-center gap-2">
                              <div className="w-2.5 h-2.5 rounded-full" style={{ background: CHART_COLORS[i % CHART_COLORS.length] }} />
                              <span className="text-[#FCFFFF] text-sm font-medium">{a.assetSymbol}</span>
                              <span className="text-[#92A5A8] text-xs">{a.chain}</span>
                            </div>
                          </td>
                          <td className="text-right px-5 py-3 text-[#92A5A8] text-sm">{a.currentPercentage.toFixed(1)}%</td>
                          <td className="text-right px-5 py-3 text-[#FCFFFF] text-sm font-medium">{a.recommendedPercentage.toFixed(1)}%</td>
                          <td className={`text-right px-5 py-3 text-sm font-semibold ${delta >= 0 ? "text-[#22c55e]" : "text-red-400"}`}>
                            {delta >= 0 ? "+" : ""}{delta.toFixed(1)}%
                          </td>
                          <td className="px-5 py-3 text-[#33C5E0] text-xs max-w-[200px]">{a.expectedImpact}</td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>
          </motion.div>
        )}

        {/* PROJECTIONS TAB */}
        {state.activeTab === "projections" && (
          <motion.div key="projections" initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -8 }} transition={{ duration: 0.2 }} className="space-y-6">

            {/* Scenario selector */}
            <div className="bg-[#1C252A] rounded-xl p-5 border border-[#2A3338]">
              <div className="flex items-center gap-2 mb-4">
                <h3 className="text-[#FCFFFF] font-medium">Scenario Analysis</h3>
                <Tooltip text="Select a scenario to see projected outcomes under different market conditions.">
                  <Info size={14} className="text-[#4A5558] cursor-help" />
                </Tooltip>
              </div>
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 mb-6">
                {[
                  { id: OptimizationScenario.CONSERVATIVE, label: "Conservative", color: "#D4A017", desc: "Lower risk, steady growth" },
                  { id: OptimizationScenario.MODERATE, label: "Moderate", color: "#33C5E0", desc: "Balanced risk-return" },
                  { id: OptimizationScenario.AGGRESSIVE, label: "Aggressive", color: "#22c55e", desc: "Higher risk, maximum growth" },
                  { id: OptimizationScenario.CUSTOM, label: "Custom", color: "#a855f7", desc: "Your custom allocation" },
                ].map((s) => (
                  <button
                    key={s.id}
                    onClick={() => setPartial({ selectedScenario: s.id })}
                    className={`p-3 rounded-xl border text-left transition-all ${state.selectedScenario === s.id ? "border-[#33C5E0] bg-[#33C5E0]/10" : "border-[#2A3338] hover:border-[#4A5558]"}`}
                  >
                    <div className="w-3 h-3 rounded-full mb-2" style={{ background: s.color }} />
                    <p className={`text-sm font-medium ${state.selectedScenario === s.id ? "text-[#FCFFFF]" : "text-[#92A5A8]"}`}>{s.label}</p>
                    <p className="text-[#92A5A8] text-xs mt-0.5">{s.desc}</p>
                  </button>
                ))}
              </div>
              <ProjectionLineChart points={projectionPoints} scenario={state.selectedScenario} />
            </div>

            {/* Projection milestones */}
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              {[
                { label: "1 Year", value: rec.projectedOutcomes.estimatedValue1Year, icon: "📅" },
                { label: "5 Years", value: rec.projectedOutcomes.estimatedValue5Year, icon: "📆" },
                { label: "10 Years", value: rec.projectedOutcomes.estimatedValue10Year, icon: "🏦" },
              ].map((m) => {
                const mult = scenarioMultipliers[state.selectedScenario];
                const adjusted = m.value * mult;
                return (
                  <div key={m.label} className="bg-[#1C252A] rounded-xl p-5 border border-[#2A3338] text-center">
                    <div className="text-2xl mb-2">{m.icon}</div>
                    <p className="text-[#92A5A8] text-xs mb-1">{m.label} Projection</p>
                    <p className="text-2xl font-bold text-[#FCFFFF]">
                      ${adjusted >= 1_000_000 ? `${(adjusted / 1_000_000).toFixed(2)}M` : `${(adjusted / 1000).toFixed(1)}K`}
                    </p>
                    <p className="text-[#92A5A8] text-xs mt-1">
                      {state.selectedScenario !== OptimizationScenario.MODERATE && (
                        <span className={mult > 1 ? "text-[#22c55e]" : "text-[#D4A017]"}>
                          {mult > 1 ? "+" : ""}{((mult - 1) * 100).toFixed(0)}% scenario adj.
                        </span>
                      )}
                    </p>
                  </div>
                );
              })}
            </div>

            {/* Risk metrics */}
            <div className="bg-[#1C252A] rounded-xl p-5 border border-[#2A3338]">
              <h3 className="text-[#FCFFFF] font-medium mb-4">Risk Metrics</h3>
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
                {[
                  { label: "Volatility", value: `${rec.projectedOutcomes.riskMetrics.volatility.toFixed(1)}%`, desc: "Annualized standard deviation", color: "#D4A017" },
                  { label: "Sharpe Ratio", value: rec.projectedOutcomes.riskMetrics.sharpeRatio.toFixed(2), desc: "Risk-adjusted performance", color: "#22c55e" },
                  { label: "Max Drawdown", value: `-${rec.projectedOutcomes.riskMetrics.maxDrawdown.toFixed(1)}%`, desc: "Largest peak-to-trough decline", color: "#ef4444" },
                  { label: "Value at Risk", value: `-${rec.projectedOutcomes.riskMetrics.valueAtRisk.toFixed(1)}%`, desc: "95% confidence monthly loss", color: "#a855f7" },
                ].map((m) => (
                  <div key={m.label} className="bg-[#161E22] rounded-lg p-4">
                    <p className="text-[#92A5A8] text-xs mb-2">{m.label}</p>
                    <p className="text-xl font-bold" style={{ color: m.color }}>{m.value}</p>
                    <p className="text-[#92A5A8] text-xs mt-1">{m.desc}</p>
                  </div>
                ))}
              </div>
            </div>
          </motion.div>
        )}

        {/* CUSTOMIZE TAB */}
        {state.activeTab === "customize" && (
          <motion.div key="customize" initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -8 }} transition={{ duration: 0.2 }} className="space-y-6">

            <div className="bg-[#1C252A] rounded-xl p-5 border border-[#2A3338]">
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-[#FCFFFF] font-medium flex items-center gap-2">
                  <Sliders size={16} className="text-[#33C5E0]" />
                  Custom Allocation
                </h3>
                <div className="flex items-center gap-3">
                  <span className="text-xs text-[#92A5A8]">Total:</span>
                  <span className={`text-sm font-bold ${Math.abs(customTotal - 100) < 0.5 ? "text-[#22c55e]" : "text-red-400"}`}>
                    {customTotal.toFixed(1)}%
                  </span>
                </div>
              </div>
              {Math.abs(customTotal - 100) >= 0.5 && (
                <div className="flex items-center gap-2 bg-red-500/10 border border-red-500/20 rounded-lg p-2 mb-4">
                  <AlertTriangle size={12} className="text-red-400 shrink-0" />
                  <p className="text-red-400 text-xs">Allocations must sum to 100%. Current total: {customTotal.toFixed(1)}%</p>
                </div>
              )}
              <p className="text-[#92A5A8] text-xs mb-4">Adjust individual asset allocations using the sliders. Ensure they sum to 100%.</p>
              <div className="space-y-3">
                {state.customAllocations.map((a, i) => (
                  <AllocationSlider
                    key={i}
                    allocation={a}
                    value={a.recommendedPercentage}
                    onChange={(v) => handleCustomChange(i, v)}
                    color={CHART_COLORS[i % CHART_COLORS.length]}
                  />
                ))}
              </div>
            </div>

            {/* What-if preview */}
            <div className="bg-[#1C252A] rounded-xl p-5 border border-[#2A3338]">
              <h3 className="text-[#FCFFFF] font-medium mb-4 flex items-center gap-2">
                <Activity size={16} className="text-[#a855f7]" />
                What-If Analysis
              </h3>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div className="bg-[#161E22] rounded-lg p-3">
                  <p className="text-[#92A5A8] text-xs mb-1">Estimated Custom Return</p>
                  <p className="text-xl font-bold text-[#22c55e]">
                    +{(rec.expectedReturn * (customTotal / 100) * 0.95).toFixed(1)}%
                  </p>
                </div>
                <div className="bg-[#161E22] rounded-lg p-3">
                  <p className="text-[#92A5A8] text-xs mb-1">Allocation Deviation</p>
                  <p className="text-xl font-bold text-[#D4A017]">
                    {state.customAllocations.reduce((acc, a) => acc + Math.abs(a.recommendedPercentage - a.currentPercentage), 0).toFixed(1)}%
                  </p>
                </div>
              </div>
              <button
                onClick={() => setPartial({ selectedScenario: OptimizationScenario.CUSTOM })}
                className="mt-4 w-full py-2 rounded-lg border border-[#a855f7]/40 text-[#a855f7] text-sm hover:bg-[#a855f7]/10 transition-colors"
              >
                Preview in Projections
              </button>
            </div>

            <button
              onClick={handleApplyCustom}
              disabled={Math.abs(customTotal - 100) >= 0.5}
              className="w-full py-3 rounded-xl bg-[#33C5E0] text-[#161E22] font-semibold hover:bg-[#2AB8D3] transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Apply Custom Allocations
            </button>
          </motion.div>
        )}
      </AnimatePresence>

      {/* ── Action Panel ── */}
      <div className="mt-8 bg-[#1C252A] rounded-xl p-5 border border-[#2A3338]">
        <h3 className="text-[#FCFFFF] font-medium mb-4">Actions</h3>
        <div className="flex flex-col sm:flex-row gap-3">
          <button
            onClick={() => setPartial({ showAcceptConfirm: true })}
            className="flex-1 flex items-center justify-center gap-2 py-3 px-5 rounded-xl bg-[#33C5E0] text-[#161E22] font-semibold hover:bg-[#2AB8D3] transition-colors"
          >
            <Check size={18} />
            Accept All Recommendations
          </button>
          <button
            onClick={() => setPartial({ activeTab: "customize" })}
            className="flex-1 flex items-center justify-center gap-2 py-3 px-5 rounded-xl border border-[#33C5E0] text-[#33C5E0] hover:bg-[#33C5E0]/10 transition-colors"
          >
            <Sliders size={18} />
            Customize Allocations
          </button>
          <button
            onClick={() => setPartial({ showRejectModal: true })}
            className="flex items-center justify-center gap-2 py-3 px-5 rounded-xl border border-[#2A3338] text-[#92A5A8] hover:text-red-400 hover:border-red-400/30 transition-colors"
          >
            <X size={18} />
            Reject
          </button>
        </div>
        <p className="text-[#92A5A8] text-xs mt-3 text-center">
          Recommendations are valid for 24 hours · Generated by InheritX AI v2.0
        </p>
      </div>

      {/* ── Modals ── */}
      <AnimatePresence>
        {state.showRejectModal && (
          <RejectModal
            reason={state.rejectReason}
            onReasonChange={(r) => setPartial({ rejectReason: r })}
            onConfirm={handleRejectConfirm}
            onCancel={() => setPartial({ showRejectModal: false })}
          />
        )}
        {state.showAcceptConfirm && (
          <AcceptConfirmModal
            recommendation={rec}
            onConfirm={handleAcceptConfirm}
            onCancel={() => setPartial({ showAcceptConfirm: false })}
          />
        )}
      </AnimatePresence>
    </div>
  );
};

export default AIOptimizationDashboard;
