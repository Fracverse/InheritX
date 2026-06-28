import { describe, expect, it } from "vitest";
import {
  getSelectedTokenIdentifier,
  percentageToBasisPoints,
  validateInheritancePlanDraft,
  type InheritancePlanDraft,
} from "@/app/lib/validation/inheritancePlan";

const VALID_OWNER = "GAIE4IHLNGMX2ZGURV2DZEAFFU6P3X7UPFNRSGRZI2QUD2IU4GVOMKIV";
const VALID_BENEFICIARY = "GCDFLQR2SGPDRQ473YUJ3Z5Z64BAJOH7EFFF4DC6ZEABSUWNLAR7Q7KJ";
const VALID_BENEFICIARY_2 = "GDP2PVYRRAB35TQMJ4DPJOV5BBBM6PJUHWKMSTF6YYXUDXUT7BCLCIFE";
const VALID_CONTRACT = "CAAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQC526";

function validDraft(overrides: Partial<InheritancePlanDraft> = {}): InheritancePlanDraft {
  return {
    owner: VALID_OWNER,
    tokenType: "XLM",
    customTokenAddress: "",
    amount: "100",
    earnYield: true,
    gracePeriodDays: 180,
    timelockDays: 7,
    beneficiaries: [
      {
        address: VALID_BENEFICIARY,
        name: "Alice",
        allocationPercentage: 100,
      },
    ],
    ...overrides,
  };
}

describe("inheritance plan validation", () => {
  it("accepts a complete draft", () => {
    expect(validateInheritancePlanDraft(validDraft()).isValid).toBe(true);
  });

  it("rejects invalid addresses and empty amounts", () => {
    const result = validateInheritancePlanDraft(
      validDraft({
        owner: "not-a-wallet",
        amount: "0",
        beneficiaries: [{ address: "bad", name: "", allocationPercentage: 100 }],
      })
    );

    expect(result.isValid).toBe(false);
    expect(result.errors.owner).toBeTruthy();
    expect(result.errors.amount).toBeTruthy();
    expect(result.errors["beneficiary.0.address"]).toBeTruthy();
    expect(result.errors["beneficiary.0.name"]).toBeTruthy();
  });

  it("requires allocations to total exactly 100 percent", () => {
    const result = validateInheritancePlanDraft(
      validDraft({
        beneficiaries: [
          { address: VALID_BENEFICIARY, name: "Alice", allocationPercentage: 60 },
          { address: VALID_BENEFICIARY_2, name: "Bob", allocationPercentage: 30 },
        ],
      })
    );

    expect(result.isValid).toBe(false);
    expect(result.errors.allocationTotal).toBeTruthy();
  });

  it("detects duplicate beneficiary addresses", () => {
    const result = validateInheritancePlanDraft(
      validDraft({
        beneficiaries: [
          { address: VALID_BENEFICIARY, name: "Alice", allocationPercentage: 50 },
          { address: VALID_BENEFICIARY, name: "Bob", allocationPercentage: 50 },
        ],
      })
    );

    expect(result.isValid).toBe(false);
    expect(result.errors["beneficiary.1.address"]).toContain("duplicate");
  });

  it("supports custom Stellar contract token identifiers", () => {
    expect(getSelectedTokenIdentifier("CUSTOM", VALID_CONTRACT)).toBe(VALID_CONTRACT);
    expect(
      validateInheritancePlanDraft(
        validDraft({ tokenType: "CUSTOM", customTokenAddress: VALID_CONTRACT })
      ).isValid
    ).toBe(true);
  });

  it("converts allocation percentages to basis points", () => {
    expect(percentageToBasisPoints(12.5)).toBe(1250);
  });
});
