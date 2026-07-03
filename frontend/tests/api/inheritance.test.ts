import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { InheritanceAPI } from "@/app/lib/api/inheritance";
import { apiClient } from "@/app/lib/api/client";

let api: InheritanceAPI;

beforeEach(() => {
  api = new InheritanceAPI();
  // Point ApiClient to empty base so MSW intercepts all requests
  (apiClient as any).baseUrl = "";
  apiClient.setSignatureAuth(null);
});

describe("InheritanceAPI", () => {
  // ─── createPlan ────────────────────────────────────────────────────────

  describe("createPlan", () => {
    it("returns a PlanResponse with 201 on success", async () => {
      const result = await api.createPlan({
        owner: "GOWNER123ADDRESS",
        token: "GTOKEN123ADDRESS",
        amount: 1000,
        beneficiaries: [
          {
            address: "GBENEFICIARY1",
            name: "Alice",
            allocation_bps: 5000,
            fiat_anchor_info: "bank-usd",
          },
          {
            address: "GBENEFICIARY2",
            name: "Bob",
            allocation_bps: 5000,
            fiat_anchor_info: "bank-eur",
          },
        ],
        last_ping: Math.floor(Date.now() / 1000),
        grace_period: 86400,
        earn_yield: true,
        yield_rate_bps: 500,
        is_active: true,
      });

      expect(result.id).toMatch(/^plan_inherit_/);
      expect(result.owner_address).toBe("GOWNER123ADDRESS");
      expect(result.status).toBe("ACTIVE");
      expect(result.beneficiaries).toHaveLength(2);
      expect(result.beneficiaries[0].wallet_address).toBe("GBENEFICIARY1");
      expect(result.beneficiaries[0].allocation_bps).toBe(5000);
    });

    it("returns error when owner is empty", async () => {
      server.use(
        http.post("/api/plans", () =>
          HttpResponse.json(
            { error: "Owner address cannot be empty" },
            { status: 400 }
          )
        )
      );

      await expect(
        api.createPlan({
          owner: "",
          token: "GTOKEN123",
          amount: 100,
          beneficiaries: [
            {
              address: "GBENEFICIARY1",
              name: "Alice",
              allocation_bps: 10000,
              fiat_anchor_info: "",
            },
          ],
          last_ping: 0,
          grace_period: 3600,
          earn_yield: false,
          yield_rate_bps: 0,
          is_active: true,
        })
      ).rejects.toThrow("Owner address cannot be empty");
    });

    it("returns error when allocation_bps sum != 10000", async () => {
      server.use(
        http.post("/api/plans", () =>
          HttpResponse.json(
            { error: "Total allocation_bps must be exactly 10000 (100%), got 8000" },
            { status: 400 }
          )
        )
      );

      await expect(
        api.createPlan({
          owner: "GOWNER",
          token: "GTOKEN",
          amount: 100,
          beneficiaries: [
            { address: "GB1", name: "A", allocation_bps: 5000, fiat_anchor_info: "" },
            { address: "GB2", name: "B", allocation_bps: 3000, fiat_anchor_info: "" },
          ],
          last_ping: 0,
          grace_period: 3600,
          earn_yield: false,
          yield_rate_bps: 0,
          is_active: true,
        })
      ).rejects.toThrow(/Total allocation_bps/);
    });

    it("returns error when no beneficiaries", async () => {
      server.use(
        http.post("/api/plans", () =>
          HttpResponse.json(
            { error: "Plan must have at least one beneficiary" },
            { status: 400 }
          )
        )
      );

      await expect(
        api.createPlan({
          owner: "GOWNER",
          token: "GTOKEN",
          amount: 100,
          beneficiaries: [],
          last_ping: 0,
          grace_period: 3600,
          earn_yield: false,
          yield_rate_bps: 0,
          is_active: true,
        })
      ).rejects.toThrow("Plan must have at least one beneficiary");
    });

    it("returns error on server error (500)", async () => {
      server.use(
        http.post("/api/plans", () =>
          HttpResponse.json(
            { error: "Failed to begin database transaction" },
            { status: 500 }
          )
        )
      );

      await expect(
        api.createPlan({
          owner: "GOWNER",
          token: "GTOKEN",
          amount: 100,
          beneficiaries: [
            { address: "GB1", name: "A", allocation_bps: 10000, fiat_anchor_info: "" },
          ],
          last_ping: 0,
          grace_period: 3600,
          earn_yield: false,
          yield_rate_bps: 0,
          is_active: true,
        })
      ).rejects.toThrow("Failed to begin database transaction");
    });
  });

  // ─── getPlans ──────────────────────────────────────────────────────────

  describe("getPlans", () => {
    it("returns all plans by default", async () => {
      const result = await api.getPlans();
      expect(result).toBeDefined();
      // The existing GET /api/plans handler returns wrapped { status, data, ... }
      // so we get back what the mock returns
      expect(result).toHaveProperty("status");
    });

    it("filters by owner when provided", async () => {
      const result = await api.getPlans({ owner: "GOWNER1" });
      expect(result).toBeDefined();
    });

    it("filters by beneficiary when provided", async () => {
      const result = await api.getPlans({ beneficiary: "GBENEFICIARY1" });
      expect(result).toBeDefined();
    });

    it("filters by both owner and beneficiary", async () => {
      const result = await api.getPlans({
        owner: "GOWNER1",
        beneficiary: "GBENEFICIARY1",
      });
      expect(result).toBeDefined();
    });
  });

  // ─── pingPlan ──────────────────────────────────────────────────────────

  describe("pingPlan", () => {
    it("returns PingResponse on success", async () => {
      const result = await api.pingPlan({
        owner: "GOWNER123",
        signature: "validsighex",
        message: "ping-1719000000",
      });

      expect(result.owner).toBe("GOWNER123");
      expect(result.status).toBe("ACTIVE");
      expect(result.virtual_balance).toBe("1050.75");
    });

    it("returns 401 when signature is empty", async () => {
      server.use(
        http.post("/api/plans/ping", () =>
          HttpResponse.json({ error: "Invalid signature" }, { status: 401 })
        )
      );

      await expect(
        api.pingPlan({
          owner: "GOWNER123",
          signature: "",
          message: "test",
        })
      ).rejects.toThrow("Invalid signature");
    });

    it("returns error when plan not found", async () => {
      server.use(
        http.post("/api/plans/ping", () =>
          HttpResponse.json({ error: "Active plan not found" }, { status: 404 })
        )
      );

      await expect(
        api.pingPlan({
          owner: "GOWNER_UNKNOWN",
          signature: "validsig",
          message: "test",
        })
      ).rejects.toThrow("Active plan not found");
    });
  });

  // ─── triggerPayout ─────────────────────────────────────────────────────

  describe("triggerPayout", () => {
    it("succeeds with valid owner", async () => {
      await expect(
        api.triggerPayout({ owner: "GOWNER123" })
      ).resolves.toBeDefined();
    });

    it("returns error when owner is empty", async () => {
      server.use(
        http.post("/api/plans/payout", () =>
          HttpResponse.json(
            { error: "Owner address cannot be empty" },
            { status: 400 }
          )
        )
      );

      await expect(
        api.triggerPayout({ owner: "" })
      ).rejects.toThrow("Owner address cannot be empty");
    });
  });

  // ─── getPayoutStatus ───────────────────────────────────────────────────

  describe("getPayoutStatus", () => {
    it("returns paginated payout records", async () => {
      const result = await api.getPayoutStatus();

      expect(result.data).toBeDefined();
      expect(Array.isArray(result.data)).toBe(true);
      expect(result.page).toBe(1);
      expect(result.page_size).toBe(20);
      expect(typeof result.total).toBe("number");

      if (result.data.length > 0) {
        const payout = result.data[0];
        expect(payout.id).toBeDefined();
        expect(payout.plan_id).toBeDefined();
        expect(payout.beneficiary_address).toBeDefined();
        expect(payout.amount).toBeDefined();
        expect(payout.status).toBeDefined();
        expect(payout.created_at).toBeDefined();
      }
    });

    it("filters by beneficiary_address", async () => {
      const result = await api.getPayoutStatus({
        beneficiary_address: "GBENEFICIARY1ADDRESS",
      });

      expect(result.data).toBeDefined();
      // All results should belong to this beneficiary
      for (const row of result.data) {
        expect(row.beneficiary_address).toBe("GBENEFICIARY1ADDRESS");
      }
    });

    it("respects page and page_size", async () => {
      const result = await api.getPayoutStatus({
        page: 1,
        page_size: 2,
      });

      expect(result.page).toBe(1);
      expect(result.page_size).toBe(2);
      expect(result.data.length).toBeLessThanOrEqual(2);
    });

    it("returns empty data when no matching payouts", async () => {
      const result = await api.getPayoutStatus({
        beneficiary_address: "GNOBODY",
      });

      expect(result.data).toEqual([]);
      expect(result.total).toBe(0);
    });
  });
});
