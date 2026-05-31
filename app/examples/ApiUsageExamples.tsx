/**
 * API Usage Examples
 * Demonstrates how to use the API client in various scenarios
 */

"use client";

import React from "react";
import { useApi, useMutation } from "@/app/hooks/useApi";
import { adminAPI } from "@/app/lib/api";
import { Skeleton } from "@/components/ui/Skeleton";
import { ErrorDisplay } from "@/components/ui/ErrorBoundary";

/**
 * Example 1: Simple Data Fetching
 */
export function MetricsDisplay() {
  const { data, loading, error, refetch } = useApi(
    () => adminAPI.getMetrics(),
    { immediate: true }
  );

  if (loading) return <Skeleton className="h-32 w-full" />;
  if (error) return <ErrorDisplay error={error} onRetry={refetch} />;

  return (
    <div className="grid grid-cols-4 gap-4">
      <div className="bg-[#0A0F11] p-4 rounded-lg">
        <div className="text-2xl font-bold text-white">{data?.total_users}</div>
        <div className="text-sm text-[#8899A6]">Total Users</div>
      </div>
    </div>
  );
}

/**
 * Example 2: Auto-refresh Data
 */
export function LiveMetrics() {
  const { data, loading, refetch } = useApi(() => adminAPI.getMetrics());

  React.useEffect(() => {
    const interval = setInterval(refetch, 30000);
    return () => clearInterval(interval);
  }, [refetch]);

  if (loading && !data) return <Skeleton className="h-32 w-full" />;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-white">Live Metrics</h2>
        <button onClick={refetch} className="text-sm text-[#33C5E0]">
          Refresh
        </button>
      </div>
    </div>
  );
}
