/**
 * Hook for managing inactivity countdown timer
 */

import { useState, useEffect, useCallback, useRef } from "react";
import { plansAPI } from "@/app/lib/api/plans";

export interface InactivityTimerState {
  days: number;
  hours: number;
  minutes: number;
  seconds: number;
  lastPingTimestamp: number;
  isClaimable: boolean;
  isSoonWarning: boolean;
}

interface UseInactivityTimerOptions {
  planId: string;
  enabled?: boolean;
  pollIntervalMs?: number;
  warningThresholdHours?: number;
}

function calculateTimeRemaining(
  lastPingTimestamp: number,
  inactivityPeriodDays: number
) {
  const claimableAt =
    lastPingTimestamp + inactivityPeriodDays * 24 * 60 * 60 * 1000;

  const remainingMs = Math.max(0, claimableAt - Date.now());
  const isClaimable = remainingMs === 0;

  const totalSeconds = Math.floor(remainingMs / 1000);

  return {
    days: Math.floor(totalSeconds / (24 * 60 * 60)),
    hours: Math.floor((totalSeconds % (24 * 60 * 60)) / (60 * 60)),
    minutes: Math.floor((totalSeconds % (60 * 60)) / 60),
    seconds: totalSeconds % 60,
    totalSeconds,
    isClaimable,
  };
}

export function useInactivityTimer(options: UseInactivityTimerOptions) {
  const {
    planId,
    enabled = true,
    pollIntervalMs = 5000,
    warningThresholdHours = 24,
  } = options;

  const [timerState, setTimerState] = useState<InactivityTimerState>({
    days: 0,
    hours: 0,
    minutes: 0,
    seconds: 0,
    lastPingTimestamp: Date.now(),
    isClaimable: false,
    isSoonWarning: false,
  });

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [inactivityPeriodDays, setInactivityPeriodDays] = useState<number | null>(null);

  const tickIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const fetchInactivityStatus = useCallback(async () => {
    if (!enabled) return;

    try {
      setLoading(true);

      const status = await plansAPI.getInactivityStatus(planId);

      setInactivityPeriodDays(status.inactivity_period_days);
      setError(null);

      const remaining = calculateTimeRemaining(
        status.last_ping_timestamp,
        status.inactivity_period_days
      );

      setTimerState({
        days: remaining.days,
        hours: remaining.hours,
        minutes: remaining.minutes,
        seconds: remaining.seconds,
        lastPingTimestamp: status.last_ping_timestamp,
        isClaimable: status.is_claimable || remaining.isClaimable,
        isSoonWarning:
          remaining.totalSeconds <
          warningThresholdHours * 60 * 60,
      });
    } catch (err) {
      setError(
        err instanceof Error
          ? err
          : new Error("Failed to fetch inactivity status")
      );
    } finally {
      setLoading(false);
    }
  }, [planId, enabled, warningThresholdHours]);

  // Initial fetch + polling (FIXED)
  useEffect(() => {
    if (!enabled) return;

    const init = async () => {
      await fetchInactivityStatus();
    };

    init();

    pollIntervalRef.current = setInterval(() => {
      fetchInactivityStatus();
    }, pollIntervalMs);

    return () => {
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
      }
    };
  }, [enabled, pollIntervalMs, fetchInactivityStatus]);

  // Local countdown ticker
  useEffect(() => {
    if (!enabled || inactivityPeriodDays === null) return;

    const tick = () => {
      setTimerState((prev) => {
        const remaining = calculateTimeRemaining(
          prev.lastPingTimestamp,
          inactivityPeriodDays
        );

        return {
          days: remaining.days,
          hours: remaining.hours,
          minutes: remaining.minutes,
          seconds: remaining.seconds,
          lastPingTimestamp: prev.lastPingTimestamp,
          isClaimable: remaining.isClaimable,
          isSoonWarning:
            remaining.totalSeconds <
            warningThresholdHours * 60 * 60,
        };
      });
    };

    tickIntervalRef.current = setInterval(tick, 1000);

    return () => {
      if (tickIntervalRef.current) {
        clearInterval(tickIntervalRef.current);
      }
    };
  }, [enabled, inactivityPeriodDays, warningThresholdHours]);

  const ping = useCallback(async (signedTransaction?: string) => {
    try {
      setError(null);

      const updated = await plansAPI.pingKeepAlive(
        planId,
        signedTransaction
      );

      await fetchInactivityStatus();
      return updated;
    } catch (err) {
      const error =
        err instanceof Error
          ? err
          : new Error("Keep-alive ping failed");

      setError(error);
      throw error;
    }
  }, [planId, fetchInactivityStatus]);

  const refetch = useCallback(async () => {
    await fetchInactivityStatus();
  }, [fetchInactivityStatus]);

  return {
    timerState,
    loading,
    error,
    ping,
    refetch,
  };
}