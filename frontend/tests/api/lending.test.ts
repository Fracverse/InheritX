import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { LendingAPI } from "@/app/lib/api/lending";

const getToken = () => "test-token";
let api: LendingAPI;

beforeEach(() => {
  api = new LendingAPI("", getToken);
});

describe("LendingAPI", () => {
  describe("getPoolState", () => {
    it("returns pool state", async () => {
      const state = await api.getPoolState();
      expect(state.utilization_rate).toBe(70);
      expect(state.current_apy).toBe(8.45);
      expect(state.total_deposits).toBe("12500000");
    });

    it("throws on server error", async () => {
      server.use(
        http.get("/api/lending/pool-state", () =>
          HttpResponse.json({ error: "Internal error" }, { status: 500 }),
        ),
      );
      await expect(api.getPoolState()).rejects.toThrow("Internal error");
    });
  });

  describe("getUserShares", () => {
    it("returns user lending data", async () => {
      const data = await api.getUserShares("GXYZ123");
      expect(data.shares).toBe("5240");
      expect(data.total_earnings).toBe("142.50");
    });
  });

  describe("getCurrentRate", () => {
    it("returns current APY", async () => {
      const rate = await api.getCurrentRate();
      expect(rate.apy).toBe(8.45);
    });
  });

  describe("deposit", () => {
    it("returns tx_hash on success", async () => {
      const result = await api.deposit("1000");
      expect(result.tx_hash).toContain("1000");
    });

    it("throws on insufficient funds", async () => {
      server.use(
        http.post("/api/lending/deposit", () =>
          HttpResponse.json({ error: "Insufficient funds" }, { status: 400 }),
        ),
      );
      await expect(api.deposit("999999999")).rejects.toThrow(
        "Insufficient funds",
      );
    });
  });

  describe("withdraw", () => {
    it("returns tx_hash on success", async () => {
      const result = await api.withdraw("500");
      expect(result.tx_hash).toContain("500");
    });

    it("throws when shares exceed balance", async () => {
      server.use(
        http.post("/api/lending/withdraw", () =>
          HttpResponse.json(
            { error: "Insufficient shares" },
            { status: 400 },
          ),
        ),
      );
      await expect(api.withdraw("999999")).rejects.toThrow(
        "Insufficient shares",
      );
    });
  });
});
