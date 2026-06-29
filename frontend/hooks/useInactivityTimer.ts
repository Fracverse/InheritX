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
  const MS_PER_DAY = 24 * 60 * 60 * 1000;
  const MS_PER_HOUR = 60 * 60 * 1000;
  const MS_PER_MIN = 60 * 1000;

  const claimableAt =
    lastPingTimestamp + inactivityPeriodDays * MS_PER_DAY;

  const remainingMs = Math.max(0, claimableAt - Date.now());

  const totalSeconds = Math.floor(remainingMs / 1000);

  return {
    days: Math.floor(remainingMs / MS_PER_DAY),
    hours: Math.floor((remainingMs % MS_PER_DAY) / MS_PER_HOUR),
    minutes: Math.floor((remainingMs % MS_PER_HOUR) / MS_PER_MIN),
    seconds: Math.floor((remainingMs % MS_PER_MIN) / 1000),
    totalSeconds,
    isClaimable: remainingMs === 0,
  };
}


export function useInactivityTimer({
  planId,
  enabled = true,
  pollIntervalMs = 5000,
  warningThresholdHours = 24,
}: UseInactivityTimerOptions) {


  const [timerState, setTimerState] =
    useState<InactivityTimerState>({
      days: 0,
      hours: 0,
      minutes: 0,
      seconds: 0,
      lastPingTimestamp: Date.now(),
      isClaimable: false,
      isSoonWarning: false,
    });


  const [loading, setLoading] = useState(true);

  const [error, setError] =
    useState<Error | null>(null);


  const [inactivityPeriodDays, setInactivityPeriodDays] =
    useState<number | null>(null);


  const tickRef = useRef<NodeJS.Timeout | null>(null);
  const pollRef = useRef<NodeJS.Timeout | null>(null);



  const fetchInactivityStatus = useCallback(async () => {

    if (!enabled) return;


    try {

      setLoading(true);


      const status =
        await plansAPI.getInactivityStatus(planId);


      setInactivityPeriodDays(
        status.inactivity_period_days
      );


      const remaining =
        calculateTimeRemaining(
          status.last_ping_timestamp,
          status.inactivity_period_days
        );


      setTimerState({
        days: remaining.days,
        hours: remaining.hours,
        minutes: remaining.minutes,
        seconds: remaining.seconds,
        lastPingTimestamp:
          status.last_ping_timestamp,
        isClaimable:
          status.is_claimable || remaining.isClaimable,
        isSoonWarning:
          remaining.totalSeconds <=
          warningThresholdHours * 3600,
      });


      setError(null);


    } catch (err) {

      setError(
        err instanceof Error
          ? err
          : new Error("Failed to fetch inactivity status")
      );

    } finally {

      setLoading(false);

    }

  }, [
    enabled,
    planId,
    warningThresholdHours
  ]);



  // Initial fetch + polling
  useEffect(() => {

    if (!enabled) return;


    fetchInactivityStatus();


    pollRef.current =
      setInterval(
        fetchInactivityStatus,
        pollIntervalMs
      );


    return () => {

      if (pollRef.current) {
        clearInterval(pollRef.current);
      }

    };


  }, [
    enabled,
    fetchInactivityStatus,
    pollIntervalMs
  ]);




  // countdown tick
  useEffect(() => {

    if (!enabled || !inactivityPeriodDays)
      return;


    tickRef.current =
      setInterval(() => {

        setTimerState(prev => {

          const remaining =
            calculateTimeRemaining(
              prev.lastPingTimestamp,
              inactivityPeriodDays
            );


          return {
            ...prev,
            days: remaining.days,
            hours: remaining.hours,
            minutes: remaining.minutes,
            seconds: remaining.seconds,
            isClaimable:
              remaining.isClaimable,
            isSoonWarning:
              remaining.totalSeconds <=
              warningThresholdHours * 3600,
          };

        });

      },1000);



    return () => {

      if (tickRef.current) {
        clearInterval(tickRef.current);
      }

    };


  }, [
    enabled,
    inactivityPeriodDays,
    warningThresholdHours
  ]);





  const ping = useCallback(
    async (signedTransaction?: string) => {

      try {

        setError(null);


        const response =
          await plansAPI.pingKeepAlive(
            planId,
            signedTransaction
          );


        await fetchInactivityStatus();


        return response;


      } catch(err){

        const error =
          err instanceof Error
            ? err
            : new Error("Keep alive failed");


        setError(error);

        throw error;
      }

    },
    [
      planId,
      fetchInactivityStatus
    ]
  );



  const refetch = useCallback(
    () => fetchInactivityStatus(),
    [fetchInactivityStatus]
  );



  return {
    timerState,
    loading,
    error,
    ping,
    refetch,
  };
}
