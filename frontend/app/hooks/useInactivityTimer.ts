/**
 * Hook for managing inactivity countdown timer
 * Provides client-side active timer based on blockchain last-ping
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
  isSoonWarning: boolean; // True when <= 24 hours
}

interface UseInactivityTimerOptions {
  planId: string;
  enabled?: boolean;
  pollIntervalMs?: number;
  warningThresholdHours?: number;
}

/**
 * Calculate time remaining from timestamps
 */
function calculateTimeRemaining(
  lastPingTimestamp: number,
  inactivityPeriodDays: number
): {
  days: number;
  hours: number;
  minutes: number;
  seconds: number;
  totalSeconds: number;
  isClaimable: boolean;
} {
  const claimableAt = lastPingTimestamp + inactivityPeriodDays * 24 * 60 * 60 * 1000;
  const now = Date.now();
  const remainingMs = Math.max(0, claimableAt - now);
  const isClaimable = remainingMs === 0;

  const totalSeconds = Math.floor(remainingMs / 1000);
  const days = Math.floor(totalSeconds / (24 * 60 * 60));
  const hours = Math.floor((totalSeconds % (24 * 60 * 60)) / (60 * 60));
  const minutes = Math.floor((totalSeconds % (60 * 60)) / 60);
  const seconds = totalSeconds % 60;

  return { days, hours, minutes, seconds, totalSeconds, isClaimable };
}

export function useInactivityTimer(options: UseInactivityTimerOptions) {
  const {
    planId,
    enabled = true,
    pollIntervalMs = 5000, // Poll every 5 seconds for status
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

  // Fetch initial inactivity status
  const fetchInactivityStatus = useCallback(async () => {
    if (!enabled) return;

    try {
      setLoading(true);
      const status = await plansAPI.getInactivityStatus(planId);
      setInactivityPeriodDays(status.inactivity_period_days);
      setError(null);

      // Update timer state with fetched data
      const remaining = calculateTimeRemaining(
        status.last_ping_timestamp,
        status.inactivity_period_days
      );
      const isSoonWarning = remaining.totalSeconds < warningThresholdHours * 60 * 60;

      setTimerState({
        days: remaining.days,
        hours: remaining.hours,
        minutes: remaining.minutes,
        seconds: remaining.seconds,
        lastPingTimestamp: status.last_ping_timestamp,
        isClaimable: status.is_claimable || remaining.isClaimable,
        isSoonWarning,
      });
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Failed to fetch inactivity status"));
    } finally {
      setLoading(false);
    }
  }, [planId, enabled, warningThresholdHours]);

  // Update timer every second (client-side tick)
  useEffect(() => {
    if (!enabled || !inactivityPeriodDays) return;

    // Start with immediate fetch
    fetchInactivityStatus();

    // Set up polling for fresh data from blockchain
    pollIntervalRef.current = setInterval(fetchInactivityStatus, pollIntervalMs);

    return () => {
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
      }
    };
  }, [enabled, inactivityPeriodDays, pollIntervalMs, fetchInactivityStatus]);

  // Client-side tick (every second) for smooth countdown
  useEffect(() => {
    if (!enabled || !inactivityPeriodDays) return;

    const tick = () => {
      setTimerState((prev) => {
        const remaining = calculateTimeRemaining(prev.lastPingTimestamp, inactivityPeriodDays);
        const isSoonWarning = remaining.totalSeconds < warningThresholdHours * 60 * 60;

        return {
          days: remaining.days,
          hours: remaining.hours,
          minutes: remaining.minutes,
          seconds: remaining.seconds,
          lastPingTimestamp: prev.lastPingTimestamp,
          isClaimable: remaining.isClaimable,
          isSoonWarning,
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

  const ping = useCallback(
    async (signedTransaction?: string) => {
      try {
        setError(null);
        const updated = await plansAPI.pingKeepAlive(planId, signedTransaction);
        // Refetch status to get updated timestamp
        await fetchInactivityStatus();
        return updated;
      } catch (err) {
        const error = err instanceof Error ? err : new Error("Keep-alive ping failed");
        setError(error);
        throw error;
      }
    },
    [planId, fetchInactivityStatus]
  );

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
