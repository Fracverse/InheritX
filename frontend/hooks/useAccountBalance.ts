"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useWallet } from "@/context/WalletContext";
import { STELLAR_HORIZON_URL } from "@/app/lib/stellar/network";

/** Gas threshold in XLM: accounts with less than this risk transaction failures. */
export const LOW_GAS_THRESHOLD_XLM = 5.0;

/** Cadence for background balance polling (15 seconds). */
export const BALANCE_POLL_INTERVAL_MS = 15_000;

export interface HorizonBalanceLine {
  asset_type: "native" | "credit_alphanum4" | "credit_alphanum12" | "liquidity_pool_shares";
  balance: string;
  asset_code?: string;
  asset_issuer?: string;
}

export interface HorizonAccountResponse {
  id: string;
  balances: HorizonBalanceLine[];
}

export interface AccountBalanceState {
  /** Numerical XLM balance, or null while initial load is pending. */
  xlmBalance: number | null;
  /** Human-readable formatted XLM string (e.g. "12.45 XLM"). */
  formattedBalance: string;
  /** True during the initial balance resolution. */
  isLoading: boolean;
  /** True while a background or manual re-fetch is in flight. */
  isRefreshing: boolean;
  /** True when xlmBalance is below the 5 XLM gas threshold. */
  isLowGas: boolean;
  /** True if the account is unfunded on the Stellar network (404). */
  isUnfunded: boolean;
  /** Error message if Horizon could not be reached. */
  error: string | null;
  /** Triggers an immediate balance refresh on demand. */
  refresh: () => Promise<void>;
}

/**
 * Hook managing real-time Stellar account balance polling and low-gas detection.
 *
 * Requirements addressed:
 * - Polls balance every 15s while wallet is connected.
 * - Triggers immediate refresh on "stellar:transaction-confirmed" or "stellar:balance-refresh".
 * - Flags isLowGas when XLM < 5.0 to guard against gas exhaustion on testnet.
 */
export function useAccountBalance(): AccountBalanceState {
  const { address, isConnected } = useWallet();
  const [xlmBalance, setXlmBalance] = useState<number | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
  const [isUnfunded, setIsUnfunded] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const isFetchingRef = useRef<boolean>(false);
  const refreshQueuedRef = useRef<boolean>(false);
  const abortControllerRef = useRef<AbortController | null>(null);

  const fetchBalance = useCallback(
    async (isBackground = false) => {
      if (!isConnected || !address) {
        setXlmBalance(null);
        setIsUnfunded(false);
        refreshQueuedRef.current = false;
        setError(null);
        setIsLoading(false);
        setIsRefreshing(false);
        return;
      }

      // Coalesce overlapping triggers into one fresh fetch after the active request.
      if (isFetchingRef.current) {
        refreshQueuedRef.current = true;
        return;
      }
      isFetchingRef.current = true;

      if (isBackground) {
        setIsRefreshing(true);
      } else {
        setIsLoading(true);
      }
      setError(null);

      const controller = new AbortController();
      abortControllerRef.current = controller;

      try {
        // Mock / E2E test bypass mode
        const isMockMode =
          process.env.NEXT_PUBLIC_USE_MOCK_DATA === "true" ||
          address.startsWith("GDE2KZQ4");

        if (isMockMode) {
          const stored = typeof window !== "undefined"
            ? window.localStorage.getItem("inheritx_mock_xlm_balance")
            : null;
          const mockVal = stored !== null ? parseFloat(stored) : 25.5;
          setXlmBalance(mockVal);
          setIsUnfunded(false);
          setIsLoading(false);
          setIsRefreshing(false);
          isFetchingRef.current = false;
          if (refreshQueuedRef.current) {
            refreshQueuedRef.current = false;
            void fetchBalance(true);
          }
          return;
        }

        const horizonUrl = STELLAR_HORIZON_URL.replace(/\/+$/, "");
        const res = await fetch(`${horizonUrl}/accounts/${address}`, {
          signal: controller.signal,
          headers: { Accept: "application/json" },
        });

        // Horizon returns 404 for an account that has not been funded yet
        if (res.status === 404) {
          setXlmBalance(0);
          setIsUnfunded(true);
          return;
        }

        if (!res.ok) {
          throw new Error(`Horizon error (${res.status}): ${res.statusText}`);
        }

        const data: HorizonAccountResponse = await res.json();
        const nativeLine = data.balances?.find((b) => b.asset_type === "native");
        const balance = nativeLine ? parseFloat(nativeLine.balance) : 0;

        setXlmBalance(isNaN(balance) ? 0 : balance);
        setIsUnfunded(false);
      } catch (err: unknown) {
        if ((err as Error)?.name === "AbortError") {
          return;
        }
        const msg = err instanceof Error ? err.message : "Failed to load balance";
        setError(msg);
      } finally {
        isFetchingRef.current = false;
        setIsLoading(false);
        setIsRefreshing(false);
        if (refreshQueuedRef.current) {
          refreshQueuedRef.current = false;
          void fetchBalance(true);
        }
      }
    },
    [address, isConnected]
  );

  // Initial fetch whenever connection or address changes
  useEffect(() => {
    fetchBalance(false);

    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, [fetchBalance]);

  // Periodic polling every 15 seconds
  useEffect(() => {
    if (!isConnected || !address) return;

    const intervalId = setInterval(() => {
      fetchBalance(true);
    }, BALANCE_POLL_INTERVAL_MS);

    return () => {
      clearInterval(intervalId);
    };
  }, [isConnected, address, fetchBalance]);

  // Event-driven refresh: triggers immediately upon transaction confirmation
  useEffect(() => {
    const handleTxConfirmed = () => {
      fetchBalance(true);
    };

    const handleBalanceRefresh = () => {
      fetchBalance(true);
    };

    if (typeof window !== "undefined") {
      window.addEventListener("stellar:transaction-confirmed", handleTxConfirmed);
      window.addEventListener("stellar:balance-refresh", handleBalanceRefresh);
    }

    return () => {
      if (typeof window !== "undefined") {
        window.removeEventListener("stellar:transaction-confirmed", handleTxConfirmed);
        window.removeEventListener("stellar:balance-refresh", handleBalanceRefresh);
      }
    };
  }, [fetchBalance]);

  const isLowGas = xlmBalance !== null && xlmBalance < LOW_GAS_THRESHOLD_XLM;

  const formattedBalance =
    xlmBalance !== null
      ? `${xlmBalance.toLocaleString("en-US", {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        })} XLM`
      : "-- XLM";

  return {
    xlmBalance,
    formattedBalance,
    isLoading,
    isRefreshing,
    isLowGas,
    isUnfunded,
    error,
    refresh: () => fetchBalance(true),
  };
}
