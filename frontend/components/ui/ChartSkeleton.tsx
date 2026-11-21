"use client";

import React from "react";

interface ChartSkeletonProps {
  width?: number;
  height?: number;
  variant?: "area" | "line" | "bar";
  className?: string;
}

export function ChartSkeleton({
  width = 600,
  height = 240,
  variant = "area",
  className = "",
}: ChartSkeletonProps) {
  const gridLines = [0, 1, 2, 3, 4];
  const dataPoints = variant === "bar" ? 8 : 12;

  return (
    <div
      className={`relative overflow-hidden ${className}`}
      style={{ width: "100%", height }}
      role="status"
      aria-label="Loading chart"
      data-testid="chart-skeleton"
    >
      <svg
        width="100%"
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        preserveAspectRatio="none"
        className="opacity-60"
      >
        {/* Grid lines */}
        {gridLines.map((i) => (
          <line
            key={i}
            x1={40}
            y1={20 + (i * (height - 40)) / 4}
            x2={width - 10}
            y2={20 + (i * (height - 40)) / 4}
            stroke="#1C252A"
            strokeWidth={1}
          />
        ))}

        {/* Data visualization skeleton */}
        {variant === "area" && (
          <path
            d={`M 40 ${height * 0.7}
                C ${width * 0.15} ${height * 0.55}, ${width * 0.25} ${height * 0.65}, ${width * 0.35} ${height * 0.5}
                S ${width * 0.55} ${height * 0.4}, ${width * 0.65} ${height * 0.45}
                S ${width * 0.85} ${height * 0.3}, ${width - 10} ${height * 0.35}
                L ${width - 10} ${height - 20} L 40 ${height - 20} Z`}
            fill="url(#skeletonGradient)"
            className="animate-pulse"
          />
        )}

        {variant === "line" && (
          <path
            d={`M 40 ${height * 0.6}
                C ${width * 0.2} ${height * 0.5}, ${width * 0.3} ${height * 0.55}, ${width * 0.4} ${height * 0.45}
                S ${width * 0.6} ${height * 0.35}, ${width * 0.7} ${height * 0.4}
                S ${width * 0.85} ${height * 0.3}, ${width - 10} ${height * 0.35}`}
            fill="none"
            stroke="#33C5E0"
            strokeWidth={2}
            strokeLinecap="round"
            className="animate-pulse"
            strokeDasharray="8 4"
          />
        )}

        {variant === "bar" &&
          Array.from({ length: dataPoints }).map((_, i) => {
            const barWidth = (width - 60) / dataPoints - 8;
            const barHeight = (Math.sin(i * 0.8) * 0.3 + 0.5) * (height - 60);
            const x = 40 + i * ((width - 60) / dataPoints);
            return (
              <rect
                key={i}
                x={x}
                y={height - 20 - barHeight}
                width={barWidth}
                height={barHeight}
                rx={4}
                fill="#1C252A"
                className="animate-pulse"
                style={{ animationDelay: `${i * 100}ms` }}
              />
            );
          })}

        {/* Glow filter */}
        <defs>
          <linearGradient id="skeletonGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#33C5E0" stopOpacity={0.15} />
            <stop offset="100%" stopColor="#33C5E0" stopOpacity={0.02} />
          </linearGradient>
          <filter id="skeletonGlow">
            <feGaussianBlur stdDeviation="3" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Axis labels skeleton */}
        {Array.from({ length: 6 }).map((_, i) => (
          <rect
            key={`label-${i}`}
            x={40 + i * ((width - 50) / 5)}
            y={height - 12}
            width={24}
            height={8}
            rx={2}
            fill="#1C252A"
            className="animate-pulse"
            style={{ animationDelay: `${i * 50}ms` }}
          />
        ))}
      </svg>

      {/* Shimmer overlay */}
      <div
        className="absolute inset-0 pointer-events-none"
        style={{
          background:
            "linear-gradient(90deg, transparent 0%, rgba(51,197,224,0.04) 50%, transparent 100%)",
          backgroundSize: "200% 100%",
          animation: "shimmer 2s infinite",
        }}
      />
      <style jsx global>{`
        @keyframes shimmer {
          0% { background-position: -200% 0; }
          100% { background-position: 200% 0; }
        }
      `}</style>
    </div>
  );
}

interface ChartSkeletonCardProps {
  title: string;
  subtitle?: string;
  height?: number;
  variant?: "area" | "line" | "bar";
}

export function ChartSkeletonCard({
  title,
  subtitle,
  height = 240,
  variant = "area",
}: ChartSkeletonCardProps) {
  return (
    <div className="rounded-xl border border-white/5 bg-white/[0.02] sm:p-6 p-4 h-full">
      <div className="mb-5">
        <div className="h-4 w-40 bg-[#1C252A] rounded animate-pulse" />
        {subtitle && (
          <div className="h-3 w-28 bg-[#1C252A] rounded animate-pulse mt-2" />
        )}
        <span className="sr-only">Loading {title}</span>
      </div>
      <ChartSkeleton height={height} variant={variant} />
    </div>
  );
}
