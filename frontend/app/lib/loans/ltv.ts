/**
 * Loan-to-value (LTV) and liquidation-risk helpers that mirror the math in
 * `contracts/lending-contract/src/lib.rs`, so the dashboard gauge tells the
 * user exactly what the contract will decide.
 *
 * The contract measures health as `collateral_amount * 10000 / principal`
 * and allows `liquidate` once that drops below `LIQUIDATION_THRESHOLD_BPS`
 * (15000 = 150%) *and* the loan is past `due_date + grace period`. Expressed
 * as LTV (`principal / collateral`), the liquidation line is 1 / 1.5 ≈ 66.67%.
 */

/** Mirrors `LIQUIDATION_THRESHOLD_BPS` in the lending contract (150% health factor). */
export const LIQUIDATION_HEALTH_FACTOR_BPS = 15_000;

/** LTV (bps) at which the health factor reaches the liquidation threshold: 10000² / 15000. */
export const LIQUIDATION_LTV_BPS = Math.floor((10_000 * 10_000) / LIQUIDATION_HEALTH_FACTOR_BPS); // 6666

/** LTV (bps) from which the dashboard starts warning the owner. */
export const WARNING_LTV_BPS = 5_000;

/** LTV (bps) from which the position is treated as close to liquidation. */
export const DANGER_LTV_BPS = 6_000;

export type LoanRiskLevel = "none" | "safe" | "warning" | "danger" | "liquidatable";

/** Same integer math as the contract's `calc_ltv_ratio_bps`. */
export function calcLtvBps(principal: bigint, collateralAmount: bigint): number {
  if (collateralAmount <= BigInt(0)) return 0;
  const ltv = (principal * BigInt(10_000)) / collateralAmount;
  return Number(ltv > BigInt(0xffffffff) ? BigInt(0xffffffff) : ltv);
}

/** Same integer math as the health factor computed inside the contract's `liquidate`. */
export function calcHealthFactorBps(principal: bigint, collateralAmount: bigint): number {
  if (principal <= BigInt(0)) return Number.POSITIVE_INFINITY;
  return Number((collateralAmount * BigInt(10_000)) / principal);
}

/** Classifies an LTV reading for the gauge colour and warning banner. */
export function getRiskLevel(ltvBps: number, hasDebt = true): LoanRiskLevel {
  if (!hasDebt) return "none";
  if (ltvBps > LIQUIDATION_LTV_BPS) return "liquidatable";
  if (ltvBps >= DANGER_LTV_BPS) return "danger";
  if (ltvBps >= WARNING_LTV_BPS) return "warning";
  return "safe";
}

/**
 * Collateral price drop (in bps) that would push the position over the
 * liquidation line, assuming collateral and debt are valued 1:1 today as the
 * contract does. Returns 0 when already over the line.
 */
export function bufferToLiquidationBps(ltvBps: number): number {
  if (ltvBps <= 0) return 10_000;
  if (ltvBps >= LIQUIDATION_LTV_BPS) return 0;
  return Math.round(10_000 - (ltvBps * 10_000) / LIQUIDATION_LTV_BPS);
}

/** Formats bps as a percentage string, e.g. 6666 → "66.66%". */
export function formatBps(bps: number, fractionDigits = 2): string {
  return `${(bps / 100).toFixed(fractionDigits)}%`;
}

/** Formats an atomic SEP-41 amount (default 7 decimals) for display. */
export function formatTokenAmount(atomic: bigint, decimals = 7, maxFractionDigits = 4): string {
  const negative = atomic < BigInt(0);
  const abs = negative ? -atomic : atomic;
  const base = BigInt(10) ** BigInt(decimals);
  const whole = abs / base;
  const fraction = (abs % base).toString().padStart(decimals, "0").slice(0, maxFractionDigits).replace(/0+$/, "");
  const wholeStr = whole.toLocaleString("en-US");
  return `${negative ? "-" : ""}${wholeStr}${fraction ? `.${fraction}` : ""}`;
}
