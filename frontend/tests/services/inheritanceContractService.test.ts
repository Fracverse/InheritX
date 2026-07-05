import { describe, expect, it, vi } from "vitest";
import {
  buildCreatePlanRequest,
  invokeCreatePlan,
  type CreatePlanContractInput,
} from "@/app/services/inheritanceContractService";

const baseInput: CreatePlanContractInput = {
  owner: "GAIE4IHLNGMX2ZGURV2DZEAFFU6P3X7UPFNRSGRZI2QUD2IU4GVOMKIV",
  token: "XLM",
  amount: 250,
  gracePeriodDays: 90,
  earnYield: true,
  timelockDays: 5,
  beneficiaries: [
    {
      address: "GCDFLQR2SGPDRQ473YUJ3Z5Z64BAJOH7EFFF4DC6ZEABSUWNLAR7Q7KJ",
      name: "Alice",
      allocation_bps: 7000,
      fiat_anchor_info: "",
    },
    {
      address: "GDP2PVYRRAB35TQMJ4DPJOV5BBBM6PJUHWKMSTF6YYXUDXUT7BCLCIFE",
      name: "Bob",
      allocation_bps: 3000,
      fiat_anchor_info: "",
    },
  ],
};

describe("inheritanceContractService", () => {
  it("builds the backend create plan request from contract input", () => {
    const request = buildCreatePlanRequest(baseInput);

    expect(request.owner).toBe(baseInput.owner);
    expect(request.token).toBe("XLM");
    expect(request.amount).toBe(250);
    expect(request.grace_period).toBe(90 * 86_400);
    expect(request.earn_yield).toBe(true);
    expect(request.yield_rate_bps).toBe(500);
    expect(request.beneficiaries).toHaveLength(2);
  });

  it("disables yield rate when Earn Yield is off", () => {
    const request = buildCreatePlanRequest({ ...baseInput, earnYield: false });

    expect(request.earn_yield).toBe(false);
    expect(request.yield_rate_bps).toBe(0);
  });

  it("signs the create_plan transaction when a wallet kit is available", async () => {
    const signTransaction = vi.fn().mockResolvedValue({ signedTxXdr: "signed-xdr" });

    const result = await invokeCreatePlan({
      contractInput: baseInput,
      selectedWalletId: "freighter",
      kit: { signTransaction } as never,
    });

    expect(signTransaction).toHaveBeenCalledWith(
      expect.stringContaining("unsigned-xdr::create_plan::")
    );
    expect(result.signedTransactionXdr).toBe("signed-xdr");
    expect(result.request.grace_period).toBe(90 * 86_400);
  });
});
