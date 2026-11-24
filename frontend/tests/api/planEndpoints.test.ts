import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { PlansAPI } from "@/app/lib/api/plans";
import { apiClient } from "@/app/lib/api/client";

let api: PlansAPI;

beforeEach(() => {
  api = new PlansAPI();
  (apiClient as any).baseUrl = "";
});

describe("PlansAPI - corrected plan routes", () => {
  it("pings the canonical keep-alive route", async () => {
    server.use(
      http.post("/api/plans/:planId/keep-alive", ({ params }) =>
        HttpResponse.json({
          status: "ok",
          data: { id: params.planId, status: "ACTIVE" },
        }),
      ),
    );

    const result = await api.pingKeepAlive("plan_3");
    expect(result.id).toBe("plan_3");
  });

  it("reports a missing plan from keep-alive", async () => {
    server.use(
      http.post("/api/plans/:planId/keep-alive", () =>
        HttpResponse.json({ error: "Plan not found" }, { status: 404 }),
      ),
    );

    await expect(api.pingKeepAlive("plan_missing")).rejects.toThrow(
      "Plan not found",
    );
  });

  it("reads inactivity status from the canonical route", async () => {
    server.use(
      http.get("/api/plans/:planId/inactivity-status", () =>
        HttpResponse.json({
          status: "ok",
          data: {
            last_ping_timestamp: 1_700_000_000_000,
            inactivity_period_days: 30,
            days_until_claimable: 5,
            is_claimable: false,
          },
        }),
      ),
    );

    const status = await api.getInactivityStatus("plan_3");
    expect(status.inactivity_period_days).toBe(30);
    expect(status.days_until_claimable).toBe(5);
    expect(status.is_claimable).toBe(false);
    expect(status.last_ping_timestamp).toBeGreaterThan(0);
  });
});
