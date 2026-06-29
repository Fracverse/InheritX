"use client";
import { useEffect, useState } from "react";
import { DashboardLayout } from "@/components/dashboard/DashboardLayout";
import { MetricsCards } from "@/components/admin/metrics/MetricsCards";
import { TVLChart } from "@/components/admin/metrics/TVLChart";
import { AssetDistributionChart } from "@/components/admin/metrics/AssetDistributionChart";
import { adminAPI, AdminMetrics } from "@/app/lib/api/admin";
import { mockAdminMetrics } from "@/lib/mockAdminUsers";

export default function AdminDashboardPage() {
  const [metrics, setMetrics] = useState<AdminMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [usingMock, setUsingMock] = useState(false);

  useEffect(() => {
    // Production: fetches from /api/admin/metrics (backend/src/api.rs → AdminAPI.getMetrics)
    // Development fallback: uses mockAdminMetrics derived from mockUsers when backend is unavailable
    adminAPI
      .getMetrics()
      .then((data) => {
        setMetrics(data);
      })
      .catch(() => {
        setMetrics(mockAdminMetrics);
        setUsingMock(true);
      })
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-semibold text-foreground">Admin Dashboard</h1>
          <p className="text-sm text-foreground/50 mt-1">Platform-wide metrics and asset overview</p>
        </div>

        {usingMock && (
          <div className="rounded-lg border border-yellow-500/20 bg-yellow-500/5 px-4 py-2.5 text-xs text-yellow-400/80">
            Backend unavailable — showing mock data derived from local user records. In production, metrics are fetched from{" "}
            <code className="font-mono">/api/admin/metrics</code>.
          </div>
        )}

        <MetricsCards metrics={metrics} loading={loading} />

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2">
            <TVLChart />
          </div>
          <div className="lg:col-span-1">
            <AssetDistributionChart />
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}
