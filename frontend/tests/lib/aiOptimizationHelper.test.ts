import { describe, it, expect, beforeEach, vi } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";

const flags = vi.hoisted(() => ({ demo: false }));
vi.mock("@/lib/api/dataSource", () => ({
  get useMockData() {
    return flags.demo;
  },
}));

import {
  getPlanOptimization,
  applyOptimization,
  getPortfolioAnalysis,
} from "@/lib/api/aiOptimization";

beforeEach(() => {
  flags.demo = false;
});

describe("aiOptimization helper (live mode)", () => {
  it("returns the server recommendation", async () => {
    const rec = await getPlanOptimization("7");
    expect(rec.planId).toBe(7);
    expect(rec.id).toBe("rec_001");
  });

  it("surfaces live recommendation failures instead of falling back to mock data", async () => {
    server.use(
      http.get("/api/ai/optimize/:planId", () =>
        HttpResponse.json({ error: "Plan not found" }, { status: 404 }),
      ),
    );
    await expect(getPlanOptimization("1")).rejects.toThrow("Plan not found");
  });

  it("rejects an invalid plan id without calling the server", async () => {
    await expect(getPlanOptimization("abc")).rejects.toThrow("Invalid plan id");
  });

  it("persists acceptance through the respond endpoint", async () => {
    let body: unknown;
    server.use(
      http.post("/api/ai/recommendations/:id/respond", async ({ request, params }) => {
        body = { id: params.id, ...(await request.json() as object) };
        return HttpResponse.json({ status: "accepted", appliedAt: "2026-01-01T00:00:00Z" });
      }),
    );
    const result = await applyOptimization("rec_001");
    expect(body).toEqual({ id: "rec_001", action: "accept" });
    expect(result.appliedAt).toBe("2026-01-01T00:00:00Z");
  });

  it("surfaces accept failures instead of reporting success", async () => {
    server.use(
      http.post("/api/ai/recommendations/:id/respond", () =>
        HttpResponse.json({ error: "Recommendation expired" }, { status: 410 }),
      ),
    );
    await expect(applyOptimization("rec_001")).rejects.toThrow("Recommendation expired");
  });

  it("does not treat a non-accepted response as success", async () => {
    server.use(
      http.post("/api/ai/recommendations/:id/respond", () =>
        HttpResponse.json({ status: "rejected", appliedAt: "2026-01-01T00:00:00Z" }),
      ),
    );
    await expect(applyOptimization("rec_001")).rejects.toThrow();
  });

  it("fails portfolio analysis in live mode", async () => {
    await expect(getPortfolioAnalysis(["1"])).rejects.toThrow("not available");
  });
});

describe("aiOptimization helper (demo mode)", () => {
  it("returns mock data only when demo mode is on, without network calls", async () => {
    flags.demo = true;
    server.use(
      http.get("/api/ai/optimize/:planId", () => HttpResponse.error()),
      http.post("/api/ai/recommendations/:id/respond", () => HttpResponse.error()),
    );
    const rec = await getPlanOptimization("3");
    expect(rec.id).toBe("demo-rec-3");
    expect((await applyOptimization(rec.id)).status).toBe("accepted");
    expect((await getPortfolioAnalysis(["3"])).planScores).toEqual({ "3": 80 });
  });
});
