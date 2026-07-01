import { useMemo } from 'react';

export interface YieldDataPoint {
  month: number;
  year: number;
  elapsedDays: number;
  principal: number;
  accruedYield: number;
  total: number;
  yieldRate: number;
}

export interface YieldCalculationResult {
  principal: number;
  years: number;
  ratePercentage: number;
  rateBps: number;
  dataPoints: YieldDataPoint[];
  finalAccruedYield: number;
  finalTotal: number;
}

/**
 * Hook that calculates yield projections using simple interest formula.
 * Formula (from backend): accrued = principal * (rate_bps / 10000) * (elapsed_secs / seconds_per_year)
 *
 * @param principal - Amount to calculate yield on
 * @param years - Time horizon in years (1-50)
 * @param rateBps - Annual yield rate in basis points (e.g., 500 = 5% APY)
 * @returns Calculated yield data points and summary
 */
export function useYieldCalculations(
  principal: number,
  years: number,
  rateBps: number,
): YieldCalculationResult {
  return useMemo(() => {
    const rate = rateBps / 10_000;
    const ratePercentage = rateBps / 100;

    // Generate data points for each month
    const dataPoints: YieldDataPoint[] = [];
    const totalMonths = Math.ceil(years * 12);

    for (let month = 0; month <= totalMonths; month++) {
      const currentYears = month / 12;
      
      // Skip if beyond requested period
      if (currentYears > years) break;
      
      // Calculate accrued yield using simple interest formula
      const accruedYield = principal * rate * currentYears;
      const total = principal + accruedYield;

      dataPoints.push({
        month,
        year: Math.floor(currentYears),
        elapsedDays: Math.round(currentYears * 365.25),
        principal,
        accruedYield: Math.round(accruedYield * 100) / 100,
        total: Math.round(total * 100) / 100,
        yieldRate: rate,
      });
    }

    // Ensure we have the final year point
    const finalAccruedYield = principal * rate * years;
    const finalTotal = principal + finalAccruedYield;

    return {
      principal,
      years,
      ratePercentage,
      rateBps,
      dataPoints,
      finalAccruedYield: Math.round(finalAccruedYield * 100) / 100,
      finalTotal: Math.round(finalTotal * 100) / 100,
    };
  }, [principal, years, rateBps]);
}

/**
 * Calculates yield for multiple token types
 * Note: Use directly in components that need multi-token comparisons
 */
export function calculateMultipleTokenYields(
  principal: number,
  years: number,
  tokenRates: Record<string, number>, // token name -> rate in bps
): Record<string, YieldCalculationResult> {
  const result: Record<string, YieldCalculationResult> = {};
  
  Object.entries(tokenRates).forEach(([token, rateBps]) => {
    const rate = rateBps / 10_000;
    const ratePercentage = rateBps / 100;
    const dataPoints = [];
    const totalMonths = Math.ceil(years * 12);

    for (let month = 0; month <= totalMonths; month++) {
      const currentYears = month / 12;
      if (currentYears > years) break;
      
      const accruedYield = principal * rate * currentYears;
      const total = principal + accruedYield;

      dataPoints.push({
        month,
        year: Math.floor(currentYears),
        elapsedDays: Math.round(currentYears * 365.25),
        principal,
        accruedYield: Math.round(accruedYield * 100) / 100,
        total: Math.round(total * 100) / 100,
        yieldRate: rate,
      });
    }

    const finalAccruedYield = principal * rate * years;
    const finalTotal = principal + finalAccruedYield;

    result[token] = {
      principal,
      years,
      ratePercentage,
      rateBps,
      dataPoints,
      finalAccruedYield: Math.round(finalAccruedYield * 100) / 100,
      finalTotal: Math.round(finalTotal * 100) / 100,
    };
  });

  return result;
}
