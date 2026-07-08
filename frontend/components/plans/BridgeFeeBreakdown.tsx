"use client";

import { Loader2 } from "lucide-react";
import type { BridgeQuote } from "@/lib/bridge/types";

interface BridgeFeeBreakdownProps {
  quote: BridgeQuote | null;
  isLoading?: boolean;
}

function formatUsd(value: number): string {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
  }).format(value);
}

export function BridgeFeeBreakdown({ quote, isLoading }: BridgeFeeBreakdownProps) {
  if (isLoading) {
    return (
      <div className="rounded-xl border border-[#2A3338] bg-[#0A0F11] p-4 flex items-center gap-3">
        <Loader2 size={16} className="animate-spin text-[#33C5E0]" />
        <p className="text-sm text-[#92A5A8]">Fetching Allbridge route and fees…</p>
      </div>
    );
  }

  if (!quote) {
    return (
      <div className="rounded-xl border border-dashed border-[#2A3338] bg-[#0A0F11]/50 p-4">
        <p className="text-sm text-[#92A5A8]">
          Enter a deposit amount to see bridging fees and estimated delivery time.
        </p>
      </div>
    );
  }

  const rows = [
    { label: "Relayer fee", value: quote.relayerFeeUsd },
    { label: "Source chain gas", value: quote.gasFeeUsd },
    { label: "Destination fee", value: quote.destinationFeeUsd },
  ];

  return (
    <div className="rounded-xl border border-[#2A3338] bg-[#0A0F11] p-4 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h4 className="text-xs font-semibold uppercase tracking-wider text-[#33C5E0]">
          Bridging Fee Breakdown
        </h4>
        <span className="text-[11px] text-[#92A5A8]">
          ~{quote.estimatedDurationMinutes} min via Allbridge
        </span>
      </div>

      <dl className="space-y-2">
        {rows.map((row) => (
          <div key={row.label} className="flex items-center justify-between text-sm">
            <dt className="text-[#92A5A8]">{row.label}</dt>
            <dd className="font-mono text-slate-200">{formatUsd(row.value)}</dd>
          </div>
        ))}
        <div className="border-t border-[#2A3338] pt-2 flex items-center justify-between">
          <dt className="text-sm font-medium text-slate-100">Total fees</dt>
          <dd className="font-mono text-[#33C5E0] font-semibold">
            {formatUsd(quote.totalFeeUsd)}
          </dd>
        </div>
      </dl>

      <div className="rounded-lg bg-white/5 px-3 py-2 text-xs text-[#92A5A8]">
        Estimated receive on Stellar:{" "}
        <span className="font-mono text-slate-200">
          {quote.estimatedReceiveAmount} {quote.estimatedReceiveSymbol}
        </span>
      </div>
    </div>
  );
}

export default BridgeFeeBreakdown;
