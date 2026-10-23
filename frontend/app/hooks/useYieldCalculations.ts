import { useEffect, useMemo, useState } from 'react';

const DEFAULT_LIVE_RATE_BPS = 200;
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

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

function parseRateToBps(value: unknown): number {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return Math.max(0, Math.round(value));
  }

  if (typeof value === 'string' && value.trim().length > 0) {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return Math.max(0, Math.round(parsed));
    }
  }

  return 0;
}

export async function fetchLiveYieldRateBps(): Promise<number> {
  const endpoints = [
    `${API_BASE_URL}/api/yield/calculate?amount=1&elapsed_secs=31536000`,
    `${API_BASE_URL}/api/lending/current-rate`,
  ];

  for (const endpoint of endpoints) {
    try {
      const response = await fetch(endpoint, {
        method: 'GET',
        headers: { Accept: 'application/json' },
      });

      if (!response.ok) {
        continue;
      }

      const data = (await response.json()) as {
        apy?: number | string;
        rate?: number | string;
        rate_bps?: number | string;
        yield_rate_bps?: number | string;
      };

      const rawValue =
        data.yield_rate_bps ?? data.rate_bps ?? data.apy ?? data.rate ?? DEFAULT_LIVE_RATE_BPS;
      const normalized = Number(rawValue);

      if (!Number.isFinite(normalized)) {
        continue;
      }

      if (normalized > 100) {
        return Math.max(0, Math.round(normalized));
      }

      if (normalized > 1) {
        return Math.max(0, Math.round(normalized * 100));
      }

      return Math.max(0, Math.round(normalized * 10_000));
    } catch {
      // Try the next endpoint in the fallback chain.
    }
  }

  return DEFAULT_LIVE_RATE_BPS;
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
  const [resolvedRateBps, setResolvedRateBps] = useState(() => {
    return Number.isFinite(rateBps) && rateBps > 0 ? rateBps : DEFAULT_LIVE_RATE_BPS;
  });

  useEffect(() => {
    if (Number.isFinite(rateBps) && rateBps > 0) {
      setResolvedRateBps(rateBps);
      return;
    }

    let isMounted = true;

    fetchLiveYieldRateBps()
      .then((liveRateBps) => {
        if (isMounted) {
          setResolvedRateBps(liveRateBps);
        }
      })
      .catch(() => {
        if (isMounted) {
          setResolvedRateBps(DEFAULT_LIVE_RATE_BPS);
        }
      });

    return () => {
      isMounted = false;
    };
  }, [rateBps]);

  return useMemo(() => {
    const safeRateBps = Math.max(0, parseRateToBps(resolvedRateBps));
    const rate = safeRateBps / 10_000;
    const ratePercentage = safeRateBps / 100;

    const dataPoints: YieldDataPoint[] = [];
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

    return {
      principal,
      years,
      ratePercentage,
      rateBps: safeRateBps,
      dataPoints,
      finalAccruedYield: Math.round(finalAccruedYield * 100) / 100,
      finalTotal: Math.round(finalTotal * 100) / 100,
    };
  }, [principal, years, resolvedRateBps]);
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
