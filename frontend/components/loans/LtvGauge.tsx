"use client";

import {
  LIQUIDATION_LTV_BPS,
  WARNING_LTV_BPS,
  formatBps,
  getRiskLevel,
  type LoanRiskLevel,
} from "@/app/lib/loans/ltv";

const RISK_COLORS: Record<LoanRiskLevel, string> = {
  none: "#64748B",
  safe: "#34D399",
  warning: "#FACC15",
  danger: "#FB923C",
  liquidatable: "#F87171",
};

const RISK_LABELS: Record<LoanRiskLevel, string> = {
  none: "No debt",
  safe: "Healthy",
  warning: "Caution",
  danger: "At risk",
  liquidatable: "Liquidation zone",
};

/** Gauge spans 0–100% LTV across a 180° arc. */
const RADIUS = 80;
const ARC_LENGTH = Math.PI * RADIUS;

function pointOnArc(fraction: number, radius: number): { x: number; y: number } {
  const angle = Math.PI * (1 - fraction);
  return { x: 100 + radius * Math.cos(angle), y: 100 - radius * Math.sin(angle) };
}

export interface LtvGaugeProps {
  /** Current LTV in basis points (principal / collateral × 10000). */
  ltvBps: number;
  /** Optional projected LTV (e.g. after repayment) drawn as a ghost marker. */
  projectedLtvBps?: number;
  hasDebt?: boolean;
  size?: number;
  label?: string;
}

export function LtvGauge({ ltvBps, projectedLtvBps, hasDebt = true, size = 200, label = "Loan-to-value" }: LtvGaugeProps) {
  const clamped = Math.max(0, Math.min(10_000, ltvBps));
  const risk = getRiskLevel(clamped, hasDebt);
  const color = RISK_COLORS[risk];
  const dashOffset = ARC_LENGTH * (1 - clamped / 10_000);

  const thresholdOuter = pointOnArc(LIQUIDATION_LTV_BPS / 10_000, RADIUS + 12);
  const thresholdInner = pointOnArc(LIQUIDATION_LTV_BPS / 10_000, RADIUS - 12);
  const warningOuter = pointOnArc(WARNING_LTV_BPS / 10_000, RADIUS + 8);
  const warningInner = pointOnArc(WARNING_LTV_BPS / 10_000, RADIUS - 8);
  const projected =
    projectedLtvBps !== undefined && projectedLtvBps !== clamped
      ? pointOnArc(Math.max(0, Math.min(10_000, projectedLtvBps)) / 10_000, RADIUS)
      : null;

  return (
    <div className="flex flex-col items-center" style={{ width: size }}>
      <svg
        viewBox="0 0 200 116"
        width={size}
        height={(size * 116) / 200}
        role="meter"
        aria-label={label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Number((clamped / 100).toFixed(2))}
        aria-valuetext={`${formatBps(clamped)} — ${RISK_LABELS[risk]}`}
        data-risk={risk}
      >
        <path
          d={`M ${100 - RADIUS} 100 A ${RADIUS} ${RADIUS} 0 0 1 ${100 + RADIUS} 100`}
          fill="none"
          stroke="rgba(255,255,255,0.08)"
          strokeWidth={14}
          strokeLinecap="round"
        />
        <path
          d={`M ${100 - RADIUS} 100 A ${RADIUS} ${RADIUS} 0 0 1 ${100 + RADIUS} 100`}
          fill="none"
          stroke={color}
          strokeWidth={14}
          strokeLinecap="round"
          strokeDasharray={ARC_LENGTH}
          strokeDashoffset={dashOffset}
          style={{ transition: "stroke-dashoffset 700ms ease-out, stroke 400ms ease-out" }}
          data-testid="ltv-gauge-fill"
        />
        <line
          x1={warningInner.x}
          y1={warningInner.y}
          x2={warningOuter.x}
          y2={warningOuter.y}
          stroke="#FACC15"
          strokeOpacity={0.5}
          strokeWidth={2}
        />
        <line
          x1={thresholdInner.x}
          y1={thresholdInner.y}
          x2={thresholdOuter.x}
          y2={thresholdOuter.y}
          stroke="#F87171"
          strokeWidth={2.5}
        >
          <title>{`Liquidation threshold ${formatBps(LIQUIDATION_LTV_BPS)}`}</title>
        </line>
        {projected && (
          <circle
            cx={projected.x}
            cy={projected.y}
            r={5}
            fill="#33C5E0"
            stroke="#0B1114"
            strokeWidth={2}
            data-testid="ltv-gauge-projected"
          >
            <title>{`After repayment ${formatBps(projectedLtvBps ?? 0)}`}</title>
          </circle>
        )}
        <text x={100} y={86} textAnchor="middle" fill="white" fontSize={24} fontWeight={600}>
          {formatBps(clamped, 1)}
        </text>
        <text x={100} y={106} textAnchor="middle" fill={color} fontSize={11} fontWeight={500}>
          {RISK_LABELS[risk]}
        </text>
      </svg>
      <div className="flex w-full justify-between text-[10px] text-slate-500 -mt-1 px-1">
        <span>0%</span>
        <span className="text-red-400/80">Liq. {formatBps(LIQUIDATION_LTV_BPS, 1)}</span>
        <span>100%</span>
      </div>
    </div>
  );
}

export default LtvGauge;
