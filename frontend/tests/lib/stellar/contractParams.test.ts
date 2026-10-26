import { describe, expect, it } from "vitest";
import { scValToNative, xdr } from "@stellar/stellar-sdk";
import {
  encodeCreateInheritancePlanParams,
  encodeBeneficiaryTuple,
  encodeDistributionMethod,
  toAtomicAmount,
  type CreateInheritancePlanParamsInput,
} from "@/app/lib/stellar/contractParams";

const OWNER = "GAIE4IHLNGMX2ZGURV2DZEAFFU6P3X7UPFNRSGRZI2QUD2IU4GVOMKIV";
const TOKEN = "GDP2PVYRRAB35TQMJ4DPJOV5BBBM6PJUHWKMSTF6YYXUDXUT7BCLCIFE";

const baseParams: CreateInheritancePlanParamsInput = {
  owner: OWNER,
  token: TOKEN,
  planName: "Family Trust",
  description: "Plan for my family",
  totalAmountAtomic: BigInt("10000000000"),
  distributionMethod: "LumpSum",
  isLendable: true,
  beneficiaries: [
    {
      fullName: "Alice",
      email: "alice@example.com",
      claimCode: 123456,
      bankAccount: "0123456789",
      allocationBp: 6000,
      priority: 1,
    },
    {
      fullName: "Bob",
      email: "bob@example.com",
      claimCode: 654321,
      bankAccount: "9876543210",
      allocationBp: 4000,
      priority: 2,
    },
  ],
};

describe("encodeDistributionMethod", () => {
  it("encodes a fieldless enum variant as Vec([Symbol(variant)])", () => {
    const scv = encodeDistributionMethod("Monthly");
    expect(scv.switch().name).toBe("scvVec");
    const vec = scv.vec()!;
    expect(vec).toHaveLength(1);
    expect(vec[0].switch().name).toBe("scvSymbol");
    expect(vec[0].sym().toString()).toBe("Monthly");
  });
});

describe("encodeBeneficiaryTuple", () => {
  it("encodes the beneficiary tuple positionally", () => {
    const scv = encodeBeneficiaryTuple(baseParams.beneficiaries[0]);
    const vec = scv.vec()!;
    expect(vec).toHaveLength(6);
    expect(scValToNative(vec[0])).toBe("Alice");
    expect(scValToNative(vec[1])).toBe("alice@example.com");
    expect(scValToNative(vec[2])).toBe(123456);
    expect(Buffer.from(vec[3].bytes()).toString("utf-8")).toBe("0123456789");
    expect(scValToNative(vec[4])).toBe(6000);
    expect(scValToNative(vec[5])).toBe(1);
  });
});

describe("encodeCreateInheritancePlanParams", () => {
  it("builds a Map with alphabetically sorted Symbol keys", () => {
    const scv = encodeCreateInheritancePlanParams(baseParams);
    expect(scv.switch().name).toBe("scvMap");
    const map = scv.map()!;
    const keys = map.map((entry) => entry.key().sym().toString());

    expect(keys).toEqual([
      "beneficiaries_data",
      "description",
      "distribution_method",
      "is_lendable",
      "owner",
      "plan_name",
      "token",
      "total_amount",
    ]);
    expect(keys).toEqual([...keys].sort());
  });

  it("round-trips scalar fields through scValToNative", () => {
    const scv = encodeCreateInheritancePlanParams(baseParams);
    const map = new Map(
      scv.map()!.map((entry) => [entry.key().sym().toString(), entry.val()])
    );

    expect(scValToNative(map.get("plan_name")!)).toBe("Family Trust");
    expect(scValToNative(map.get("description")!)).toBe("Plan for my family");
    expect(scValToNative(map.get("is_lendable")!)).toBe(true);
    expect(scValToNative(map.get("total_amount")!).toString()).toBe("10000000000");

    const beneficiaries = map.get("beneficiaries_data")!.vec()!;
    expect(beneficiaries).toHaveLength(2);
  });

  it("encodes owner/token as ScAddress", () => {
    const scv = encodeCreateInheritancePlanParams(baseParams);
    const map = new Map(
      scv.map()!.map((entry) => [entry.key().sym().toString(), entry.val()])
    );
    expect(map.get("owner")!.switch()).toBe(xdr.ScValType.scvAddress());
    expect(map.get("token")!.switch()).toBe(xdr.ScValType.scvAddress());
  });
});

describe("toAtomicAmount", () => {
  it("converts a whole-number decimal string using 7 decimals by default", () => {
    expect(toAtomicAmount("100")).toBe(BigInt("1000000000"));
  });

  it("converts a fractional decimal string", () => {
    expect(toAtomicAmount("1.5")).toBe(BigInt("15000000"));
  });

  it("truncates extra fractional digits beyond the token's precision", () => {
    expect(toAtomicAmount("1.123456789")).toBe(BigInt("11234567"));
  });

  it("rejects invalid input", () => {
    expect(() => toAtomicAmount("abc")).toThrow(RangeError);
    expect(() => toAtomicAmount("-1")).toThrow(RangeError);
  });
});
