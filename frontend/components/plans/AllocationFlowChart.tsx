"use client";

import { useMemo } from "react";

export interface BeneficiaryFlow {
  name: string;
  allocation_percentage: number;
  isFiat: boolean;
}

interface AllocationFlowChartProps {
  totalAmount: number;
  beneficiaries: BeneficiaryFlow[];
  className?: string;
}

const VB_W = 800;
const VB_H = 300;
const ROOT_X = VB_W / 2;
const ROOT_Y_TOP = 55;
const ROOT_Y_BOTTOM = 95;
const BEN_Y_TOP = 175;
const BEN_Y_BOTTOM = 255;
const FUND_NODE_W = 180;
const FUND_NODE_H = 46;
const BEN_NODE_W = 156;
const BEN_NODE_H = 82;

function formatCurrency(amount: number): string {
  if (amount >= 1_000_000) {
    return `$${(amount / 1_000_000).toFixed(2)}M`;
  }
  if (amount >= 1_000) {
    return `$${(amount / 1_000).toFixed(2)}K`;
  }
  return `$${amount.toFixed(2)}`;
}

function formatExact(amount: number): string {
  return `$${amount.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}

function linkPath(x1: number, y1: number, x2: number, y2: number): string {
  const cy = (y1 + y2) / 2;
  return `M${x1},${y1} C${x1},${cy} ${x2},${cy} ${x2},${y2}`;
}

export function AllocationFlowChart({
  totalAmount,
  beneficiaries,
  className = "",
}: AllocationFlowChartProps) {
  const layout = useMemo(() => {
    const count = beneficiaries.length;
    const benXPositions: number[] = [];
    if (count === 0) {
      return { benXPositions: [], hasMultiple: false };
    }
    if (count === 1) {
      benXPositions.push(VB_W / 2);
    } else {
      const totalWidth = count * BEN_NODE_W;
      const gap = Math.max(20, (VB_W - 120 - totalWidth) / (count - 1));
      const startX = (VB_W - (count * BEN_NODE_W + (count - 1) * gap)) / 2;
      for (let i = 0; i < count; i++) {
        benXPositions.push(startX + i * (BEN_NODE_W + gap) + BEN_NODE_W / 2);
      }
    }
    return { benXPositions, hasMultiple: count > 1 };
  }, [beneficiaries]);

  const { benXPositions, hasMultiple } = layout;
  const benCount = beneficiaries.length;

  return (
    <div className={`w-full overflow-x-auto ${className}`}>
      <svg
        viewBox={`0 0 ${VB_W} ${VB_H}`}
        className="w-full h-auto"
        style={{ minHeight: "240px", maxHeight: "360px" }}
        role="img"
        aria-label={`Allocation flow chart: $${totalAmount.toLocaleString()} distributed among ${benCount} beneficiaries`}
      >
        <defs>
          <linearGradient
            id="fundNodeGrad"
            x1="0"
            y1="0"
            x2="1"
            y2="1"
          >
            <stop offset="0%" stopColor="#0A1A1F" />
            <stop offset="100%" stopColor="#0F232A" />
          </linearGradient>
          <filter id="nodeShadow">
            <feDropShadow dx="0" dy="2" stdDeviation="4" floodColor="#33C5E0" floodOpacity="0.12" />
          </filter>
          <marker
            id="arrowhead"
            markerWidth="8"
            markerHeight="6"
            refX="8"
            refY="3"
            orient="auto"
          >
            <polygon points="0 0, 8 3, 0 6" fill="#2A3338" />
          </marker>
        </defs>

        {/* Connecting paths */}
        {beneficiaries.map((ben, i) => {
          const bx = benXPositions[i] ?? VB_W / 2;
          return (
            <g key={`path-${i}`}>
              <path
                d={linkPath(ROOT_X, ROOT_Y_BOTTOM, bx, BEN_Y_TOP)}
                fill="none"
                stroke="#2A3338"
                strokeWidth="2"
                strokeLinecap="round"
                opacity="0.8"
              />
              {hasMultiple && i === 0 && (
                <line
                  x1={ROOT_X}
                  y1={ROOT_Y_BOTTOM}
                  x2={bx}
                  y2={ROOT_Y_BOTTOM}
                  stroke="#2A3338"
                  strokeWidth="2"
                  opacity="0.8"
                />
              )}
              {hasMultiple && i > 0 && (
                <line
                  x1={benXPositions[i - 1]!}
                  y1={ROOT_Y_BOTTOM}
                  x2={bx}
                  y2={ROOT_Y_BOTTOM}
                  stroke="#2A3338"
                  strokeWidth="2"
                  opacity="0.8"
                />
              )}
            </g>
          );
        })}

        {/* Fund node */}
        <g filter="url(#nodeShadow)">
          <rect
            x={ROOT_X - FUND_NODE_W / 2}
            y={ROOT_Y_TOP}
            width={FUND_NODE_W}
            height={FUND_NODE_H}
            rx="10"
            ry="10"
            fill="url(#fundNodeGrad)"
            stroke="#33C5E0"
            strokeWidth="1.5"
          />
          <text
            x={ROOT_X}
            y={ROOT_Y_TOP + 20}
            textAnchor="middle"
            fill="#FFFFFF"
            fontSize="12"
            fontWeight="600"
            fontFamily="var(--font-outfit), system-ui, sans-serif"
          >
            Inheritance Fund
          </text>
          <text
            x={ROOT_X}
            y={ROOT_Y_TOP + 37}
            textAnchor="middle"
            fill="#33C5E0"
            fontSize="14"
            fontWeight="700"
            fontFamily="var(--font-mono), monospace"
          >
            {formatExact(totalAmount)}
          </text>
        </g>

        {/* Beneficiary nodes */}
        {beneficiaries.map((ben, i) => {
          const bx = benXPositions[i] ?? VB_W / 2;
          const amount =
            totalAmount * (ben.allocation_percentage / 100);
          return (
            <g key={`ben-${i}`}>
              <rect
                x={bx - BEN_NODE_W / 2}
                y={BEN_Y_TOP}
                width={BEN_NODE_W}
                height={BEN_NODE_H}
                rx="8"
                ry="8"
                fill="#161E22"
                stroke="#2A3338"
                strokeWidth="1"
              />
              {/* Name */}
              <text
                x={bx}
                y={BEN_Y_TOP + 19}
                textAnchor="middle"
                fill="#FFFFFF"
                fontSize="11"
                fontWeight="600"
                fontFamily="var(--font-outfit), system-ui, sans-serif"
              >
                {ben.name.length > 14
                  ? ben.name.slice(0, 13) + "…"
                  : ben.name}
              </text>
              {/* Percentage badge */}
              <rect
                x={bx - 30}
                y={BEN_Y_TOP + 25}
                width="60"
                height="16"
                rx="8"
                fill="#33C5E014"
              />
              <text
                x={bx}
                y={BEN_Y_TOP + 37}
                textAnchor="middle"
                fill="#33C5E0"
                fontSize="10"
                fontWeight="700"
                fontFamily="var(--font-mono), monospace"
              >
                {ben.allocation_percentage}%
              </text>
              {/* Amount */}
              <text
                x={bx}
                y={BEN_Y_TOP + 54}
                textAnchor="middle"
                fill="#92A5A8"
                fontSize="11"
                fontWeight="500"
                fontFamily="var(--font-mono), monospace"
              >
                {formatCurrency(amount)}
              </text>
              {/* Payout type badge */}
              <rect
                x={bx - 40}
                y={BEN_Y_TOP + 60}
                width="80"
                height="16"
                rx="8"
                fill={ben.isFiat ? "#F59E0B14" : "#48BB7814"}
              />
              <text
                x={bx}
                y={BEN_Y_TOP + 72}
                textAnchor="middle"
                fill={ben.isFiat ? "#F59E0B" : "#48BB78"}
                fontSize="9"
                fontWeight="600"
                fontFamily="var(--font-outfit), system-ui, sans-serif"
              >
                {ben.isFiat ? "FIAT OFF-RAMP" : "DIRECT TOKEN"}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}

export default AllocationFlowChart;
