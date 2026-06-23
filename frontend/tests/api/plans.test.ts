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

describe("PlansAPI - Trigger and Settlement Endpoints", () => {
  describe("triggerPlan", () => {
    it("successfully triggers plan inheritance execution", async () => {
      const result = await api.triggerPlan("plan_trigger_test");
      expect(result.status).toBe("ok");
      expect(result.message).toBe("Inheritance triggered successfully");
    });

    it("throws error when trigger fails", async () => {
      server.use(
        http.post("/api/plans/:id/trigger", () =>
          HttpResponse.json({ error: "Failed to trigger" }, { status: 400 }),
        ),
      );
      await expect(api.triggerPlan("plan_trigger_err")).rejects.toThrow("Failed to trigger");
    });
  });

  describe("freezeLoans", () => {
    it("successfully freezes outstanding loans", async () => {
      const result = await api.freezeLoans("plan_freeze_test");
      expect(result.status).toBe("ok");
      expect(result.message).toBe("Loans frozen successfully");
    });
  });

  describe("recallLoans", () => {
    it("successfully recalls loans from Soroban pools", async () => {
      const result = await api.recallLoans("plan_recall_test");
      expect(result.status).toBe("ok");
      expect(result.message).toBe("Loans recalled successfully");
    });
  });

  describe("liquidateAndSettle", () => {
    it("successfully triggers auto-liquidation fallback", async () => {
      const result = await api.liquidateAndSettle("plan_liquidate_test");
      expect(result.status).toBe("ok");
      expect(result.message).toBe("Collateral liquidated and plan settled successfully");
    });
  });

  describe("getTriggerInfo", () => {
    it("successfully returns trigger status dashboard metadata", async () => {
      const result = await api.getTriggerInfo("plan_info_test");
      expect(result.status).toBe("ok");
      expect(result.data).toBeDefined();
      expect(result.data.freeze_status).toBe("PENDING");
      expect(result.data.outstanding_loans).toHaveLength(2);
    });
  });
});
