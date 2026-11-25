/**
 * AI Optimization helper.
 *
 * Live mode (default) calls the recommendation and response endpoints through
 * `AIOptimizationAPI`. Failures are thrown to the caller; there is no random
 * or hard-coded fallback. Mock output is only returned when demo mode is
 * explicitly enabled with `NEXT_PUBLIC_USE_MOCK_DATA=true`.
 */

import {
  createAIOptimizationAPI,
  type OptimizationRecommendation,
  type RecommendationResponse,
} from "../../app/lib/api/aiOptimization";
import { useMockData } from "./dataSource";

export type { OptimizationRecommendation, RecommendationResponse };

function parsePlanId(planId: string): number {
  const id = Number(planId);
  if (!Number.isInteger(id) || id < 0) {
    throw new Error(`Invalid plan id: ${planId}`);
  }
  return id;
}

/**
 * Demo-only recommendation. Never reachable unless demo mode is enabled.
 */
function demoRecommendation(planId: number): OptimizationRecommendation {
  return {
    id: `demo-rec-${planId}`,
    planId,
    recommendedAllocations: [
      {
        assetSymbol: "XLM",
        chain: "Stellar",
        currentPercentage: 45,
        recommendedPercentage: 30,
        adjustmentReason: "Reduce concentration risk (demo data)",
        expectedImpact: "Lower volatility (demo data)",
      },
      {
        assetSymbol: "USDC",
        chain: "Stellar",
        currentPercentage: 55,
        recommendedPercentage: 70,
        adjustmentReason: "Increase stable assets (demo data)",
        expectedImpact: "Capital preservation (demo data)",
      },
    ],
    confidenceScore: 80,
    expectedReturn: 10,
    riskScore: 40,
    reasoning: "Demo data. Not a real analysis of this plan.",
    generatedAt: new Date().toISOString(),
    projectedOutcomes: {
      estimatedValue1Year: 0,
      estimatedValue5Year: 0,
      estimatedValue10Year: 0,
      riskMetrics: { volatility: 0, sharpeRatio: 0, maxDrawdown: 0, valueAtRisk: 0 },
    },
  };
}

/**
 * Fetch the AI optimization recommendation for a plan.
 */
export async function getPlanOptimization(
  planId: string
): Promise<OptimizationRecommendation> {
  const id = parsePlanId(planId);
  if (useMockData) return demoRecommendation(id);
  return createAIOptimizationAPI().getOptimizationRecommendation(id);
}

/**
 * Accept a recommendation. Resolves with the server's response; rejects if the
 * server fails or does not report the recommendation as accepted.
 */
export async function applyOptimization(
  recommendationId: string
): Promise<RecommendationResponse> {
  if (useMockData) {
    return { status: "accepted", appliedAt: new Date().toISOString() };
  }
  const result = await createAIOptimizationAPI().acceptRecommendation(recommendationId);
  if (result.status !== "accepted") {
    throw new Error("The server did not accept this recommendation.");
  }
  return result;
}

/**
 * Portfolio analysis has no live endpoint. It is available in demo mode only
 * and fails explicitly in live mode instead of returning invented numbers.
 */
export async function getPortfolioAnalysis(planIds: string[]): Promise<{
  overallHealthScore: number;
  totalEstimatedSavings: number;
  recommendations: string[];
  planScores: Record<string, number>;
}> {
  if (!useMockData) {
    throw new Error("Portfolio analysis is not available: no live endpoint exists.");
  }
  const planScores = Object.fromEntries(planIds.map((id) => [id, 80]));
  return {
    overallHealthScore: planIds.length ? 80 : 0,
    totalEstimatedSavings: 0,
    recommendations: ["Demo data. Not a real analysis."],
    planScores,
  };
}
