import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { AIOptimizationAPI } from "@/app/lib/api/aiOptimization";
import type { AssetAllocation } from "@/app/lib/api/aiOptimization";

const getToken = () => "test-token";
let api: AIOptimizationAPI;

const MOCK_ALLOCATIONS: AssetAllocation[] = [
  {
    assetSymbol: "XLM",
    chain: "Stellar",
    currentPercentage: 45,
    recommendedPercentage: 30,
    adjustmentReason: "Reduce concentration risk",
    expectedImpact: "Lower volatility",
  },
  {
    assetSymbol: "USDC",
    chain: "Stellar",
    currentPercentage: 25,
    recommendedPercentage: 35,
    adjustmentReason: "Increase stable assets",
    expectedImpact: "Capital preservation",
  },
];

beforeEach(() => {
  api = new AIOptimizationAPI("", getToken);
});

describe("AIOptimizationAPI", () => {
  describe("getOptimizationRecommendation", () => {
    it("returns recommendation for a plan", async () => {
      const rec = await api.getOptimizationRecommendation(1);
      expect(rec.planId).toBe(1);
      expect(rec.confidenceScore).toBe(87);
      expect(rec.expectedReturn).toBe(14.3);
      expect(rec.riskScore).toBe(42);
    });

    it("returns recommended allocations array", async () => {
      const rec = await api.getOptimizationRecommendation(1);
      expect(Array.isArray(rec.recommendedAllocations)).toBe(true);
      expect(rec.recommendedAllocations.length).toBeGreaterThan(0);
    });

    it("returns projected outcomes with risk metrics", async () => {
      const rec = await api.getOptimizationRecommendation(1);
      expect(rec.projectedOutcomes.estimatedValue1Year).toBeGreaterThan(0);
      expect(rec.projectedOutcomes.estimatedValue5Year).toBeGreaterThan(0);
      expect(rec.projectedOutcomes.estimatedValue10Year).toBeGreaterThan(0);
      expect(rec.projectedOutcomes.riskMetrics.sharpeRatio).toBeGreaterThan(0);
    });

    it("throws on server error", async () => {
      server.use(
        http.get("/api/ai/optimize/:planId", () =>
          HttpResponse.json({ error: "Plan not found" }, { status: 404 }),
        ),
      );
      await expect(api.getOptimizationRecommendation(999)).rejects.toThrow("Plan not found");
    });

    it("throws on internal server error", async () => {
      server.use(
        http.get("/api/ai/optimize/:planId", () =>
          HttpResponse.json({ error: "Internal server error" }, { status: 500 }),
        ),
      );
      await expect(api.getOptimizationRecommendation(1)).rejects.toThrow();
    });
  });

  describe("acceptRecommendation", () => {
    it("returns accepted status", async () => {
      const result = await api.acceptRecommendation("rec_001");
      expect(result.status).toBe("accepted");
      expect(result.appliedAt).toBeTruthy();
    });

    it("throws on server error", async () => {
      server.use(
        http.post("/api/ai/recommendations/:id/respond", () =>
          HttpResponse.json({ error: "Recommendation expired" }, { status: 410 }),
        ),
      );
      await expect(api.acceptRecommendation("rec_expired")).rejects.toThrow(
        "Recommendation expired",
      );
    });
  });

  describe("rejectRecommendation", () => {
    it("returns rejected status with reason", async () => {
      const result = await api.rejectRecommendation("rec_001", "Too risky");
      expect(result.status).toBe("rejected");
      expect(result.reason).toBe("Too risky");
    });

    it("submits reason in request body", async () => {
      let capturedBody: unknown;
      server.use(
        http.post("/api/ai/recommendations/:id/respond", async ({ request }) => {
          capturedBody = await request.json();
          return HttpResponse.json({
            status: "rejected",
            reason: (capturedBody as { reason: string }).reason,
            appliedAt: new Date().toISOString(),
          });
        }),
      );
      await api.rejectRecommendation("rec_001", "Prefer conservative");
      expect((capturedBody as { reason: string }).reason).toBe("Prefer conservative");
      expect((capturedBody as { action: string }).action).toBe("reject");
    });

    it("throws on authorization failure", async () => {
      server.use(
        http.post("/api/ai/recommendations/:id/respond", () =>
          HttpResponse.json({ error: "Unauthorized" }, { status: 401 }),
        ),
      );
      await expect(api.rejectRecommendation("rec_001", "reason")).rejects.toThrow(
        "Unauthorized",
      );
    });
  });

  describe("getCustomProjection", () => {
    it("returns custom projection for given allocations", async () => {
      const result = await api.getCustomProjection(1, MOCK_ALLOCATIONS);
      expect(result.expectedReturn).toBeGreaterThan(0);
      expect(result.riskScore).toBeGreaterThanOrEqual(0);
      expect(result.projectedOutcomes).toBeDefined();
    });

    it("returns allocations back in response", async () => {
      const result = await api.getCustomProjection(1, MOCK_ALLOCATIONS);
      expect(Array.isArray(result.allocations)).toBe(true);
    });

    it("throws on invalid allocation total", async () => {
      server.use(
        http.post("/api/ai/optimize/:planId/custom", () =>
          HttpResponse.json({ error: "Allocations must sum to 100%" }, { status: 400 }),
        ),
      );
      await expect(api.getCustomProjection(1, MOCK_ALLOCATIONS)).rejects.toThrow(
        "Allocations must sum to 100%",
      );
    });
  });
});
