"use client";

import React, { useState } from "react";
import { AIOptimizationDashboard } from "@/components/AIOptimizationDashboard";
import type {
  AssetAllocation,
  OptimizationRecommendation,
} from "@/app/lib/api/aiOptimization";

// ─── Mock Data ────────────────────────────────────────────────────────────────
// Replace with real API call: createAIOptimizationAPI().getOptimizationRecommendation(planId)

const MOCK_CURRENT_ALLOCATIONS: AssetAllocation[] = [
  {
    assetSymbol: "XLM",
    chain: "Stellar",
    currentPercentage: 45,
    recommendedPercentage: 30,
    adjustmentReason: "Reduce concentration risk in native token",
    expectedImpact: "Lower volatility exposure",
  },
  {
    assetSymbol: "USDC",
    chain: "Stellar",
    currentPercentage: 25,
    recommendedPercentage: 35,
    adjustmentReason: "Increase stable asset allocation for inheritance security",
    expectedImpact: "Improved capital preservation",
  },
  {
    assetSymbol: "BTC",
    chain: "Bitcoin",
    currentPercentage: 20,
    recommendedPercentage: 22,
    adjustmentReason: "Slight increase for long-term store of value",
    expectedImpact: "Enhanced 10-year value projection",
  },
  {
    assetSymbol: "ETH",
    chain: "Ethereum",
    currentPercentage: 10,
    recommendedPercentage: 13,
    adjustmentReason: "Diversify into DeFi yield-generating assets",
    expectedImpact: "Additional yield of ~4.2% APY",
  },
];

const MOCK_RECOMMENDATION: OptimizationRecommendation = {
  id: "rec_plan_001_20250618",
  planId: 1,
  recommendedAllocations: MOCK_CURRENT_ALLOCATIONS,
  confidenceScore: 87,
  expectedReturn: 14.3,
  riskScore: 42,
  reasoning:
    "Analysis of your inheritance plan reveals an overweight position in XLM (45%) relative to your stated risk tolerance and 10-year succession timeline. Historical volatility data suggests reducing XLM exposure and increasing USDC allocation provides a more resilient inheritance portfolio. The BTC and ETH increases leverage long-term appreciation while the stable coin buffer ensures liquidity for beneficiary distributions. Monte Carlo simulations across 10,000 scenarios show this allocation outperforms the current portfolio in 73% of cases over a 10-year horizon.",
  generatedAt: new Date().toISOString(),
  projectedOutcomes: {
    estimatedValue1Year: 114_300,
    estimatedValue5Year: 197_600,
    estimatedValue10Year: 389_200,
    riskMetrics: {
      volatility: 18.4,
      sharpeRatio: 1.34,
      maxDrawdown: 28.7,
      valueAtRisk: 8.2,
    },
  },
};

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function AIOptimizationPage() {
  const [status, setStatus] = useState<"idle" | "accepted" | "rejected" | "customized">("idle");
  const [message, setMessage] = useState<string | null>(null);

  const handleAccept = (rec: OptimizationRecommendation) => {
    // TODO: call createAIOptimizationAPI().acceptRecommendation(rec.id)
    setStatus("accepted");
    setMessage(`Recommendation accepted. Your inheritance plan #${rec.planId} has been updated.`);
  };

  const handleReject = (reason: string) => {
    // TODO: call createAIOptimizationAPI().rejectRecommendation(rec.id, reason)
    setStatus("rejected");
    setMessage(`Recommendation rejected. Reason: "${reason}". We'll refine future suggestions.`);
  };

  const handleCustomize = (customAllocations: AssetAllocation[]) => {
    // TODO: call createAIOptimizationAPI().getCustomProjection(planId, customAllocations)
    setStatus("customized");
    setMessage(`Custom allocations applied for ${customAllocations.length} assets.`);
  };

  return (
    <div>
      {message && (
        <div
          className={`mx-4 sm:mx-6 mt-4 p-4 rounded-xl border text-sm flex items-start justify-between gap-3 ${
            status === "accepted" || status === "customized"
              ? "bg-[#22c55e]/10 border-[#22c55e]/20 text-[#22c55e]"
              : "bg-red-500/10 border-red-500/20 text-red-400"
          }`}
        >
          <p>{message}</p>
          <button onClick={() => setMessage(null)} className="shrink-0 opacity-60 hover:opacity-100">
            ✕
          </button>
        </div>
      )}
      <AIOptimizationDashboard
        inheritancePlanId={1}
        currentAllocations={MOCK_CURRENT_ALLOCATIONS}
        optimizationRecommendations={MOCK_RECOMMENDATION}
        onAcceptRecommendation={handleAccept}
        onRejectRecommendation={handleReject}
        onCustomizeRecommendation={handleCustomize}
      />
    </div>
  );
}
