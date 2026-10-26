import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { PlansAPI } from "@/app/lib/api/plans";
import type { UpdatePlanRequest } from "@/app/lib/api/plans";
import { apiClient } from "@/app/lib/api/client";

let api: PlansAPI;

beforeEach(() => {
  api = new PlansAPI();
  (apiClient as any).baseUrl = "";
});

describe("PlansAPI - Update Plan", () => {
  it("successfully updates a plan with new beneficiaries and settings", async () => {
    const updateRequest: UpdatePlanRequest = {
      beneficiaries: [
        {
          address: "GCDFLQR2SGPDRQ473YUJ3Z5Z64BAJOH7EFFF4DC6ZEABSUWNLAR7Q7KJ",
          name: "Alice",
          allocation_bps: 6000,
          fiat_anchor_info: "",
        },
        {
          address: "GDP2PVYRRAB35TQMJ4DPJOV5BBBM6PJUHWKMSTF6YYXUDXUT7BCLCIFE",
          name: "Bob",
          allocation_bps: 4000,
          fiat_anchor_info: "",
        },
      ],
      grace_period: 365 * 86400,
      earn_yield: true,
      yield_rate_bps: 500,
    };

    const result = await api.updatePlan("plan_1", updateRequest);
    expect(result).toBeDefined();
    expect(result.id).toBe("plan_1");
    expect(result.beneficiaries).toHaveLength(2);
    expect(result.earn_yield).toBe(true);
  });

  it("throws an error when updating a non-existent plan", async () => {
    await expect(
      api.updatePlan("plan_nonexistent", { beneficiaries: [] })
    ).rejects.toThrow();
  });

  it("throws error when server returns 500", async () => {
    server.use(
      http.put("/api/plans/:id", () =>
        HttpResponse.json({ error: "Internal server error" }, { status: 500 })
      )
    );
    await expect(
      api.updatePlan("plan_1", { beneficiaries: [] })
    ).rejects.toThrow("Internal server error");
  });
});

describe("PlansAPI - Deposit Address", () => {
  it("gets the plan deposit address and unique memo", async () => {
    const result = await api.getDepositAddress("plan_deposit_test");
    expect(result).toEqual({
      address: "GDEPOSITADDRESS",
      memo: "plan_deposit_test",
      memo_type: "text",
    });
  });
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
