import { describe, expect, it } from "vitest";
import {
  LIQUIDATION_LTV_BPS,
  bufferToLiquidationBps,
  calcHealthFactorBps,
  calcLtvBps,
  formatBps,
  formatTokenAmount,
  getRiskLevel,
} from "@/app/lib/loans/ltv";

describe("ltv helpers", () => {
  it("mirrors the contract's integer LTV and health factor math", () => {
    expect(calcLtvBps(BigInt(6_200), BigInt(10_000))).toBe(6_200);
    expect(calcLtvBps(BigInt(1), BigInt(3))).toBe(3_333);
    expect(calcLtvBps(BigInt(5), BigInt(0))).toBe(0);
    expect(calcHealthFactorBps(BigInt(10_000), BigInt(15_000))).toBe(15_000);
    expect(calcHealthFactorBps(BigInt(0), BigInt(1))).toBe(Number.POSITIVE_INFINITY);
  });

  it("places the liquidation line where the contract's 150% health factor does", () => {
    expect(LIQUIDATION_LTV_BPS).toBe(6_666);
    // health 15000 is NOT liquidatable (contract checks `>= threshold` to reject)
    expect(getRiskLevel(calcLtvBps(BigInt(10_000), BigInt(15_000)))).not.toBe("liquidatable");
    // health 14999 is liquidatable
    expect(getRiskLevel(calcLtvBps(BigInt(10_000), BigInt(14_999)))).toBe("liquidatable");
  });

  it("classifies risk bands", () => {
    expect(getRiskLevel(0, false)).toBe("none");
    expect(getRiskLevel(3_000)).toBe("safe");
    expect(getRiskLevel(5_500)).toBe("warning");
    expect(getRiskLevel(6_200)).toBe("danger");
    expect(getRiskLevel(7_000)).toBe("liquidatable");
  });

  it("computes the collateral drop buffer to liquidation", () => {
    expect(bufferToLiquidationBps(0)).toBe(10_000);
    expect(bufferToLiquidationBps(3_333)).toBe(5_000);
    expect(bufferToLiquidationBps(7_000)).toBe(0);
  });

  it("formats bps and atomic token amounts", () => {
    expect(formatBps(6_666)).toBe("66.66%");
    expect(formatTokenAmount(BigInt("62000000000"))).toBe("6,200");
    expect(formatTokenAmount(BigInt("12345678"))).toBe("1.2345");
    expect(formatTokenAmount(BigInt(0))).toBe("0");
  });
});
