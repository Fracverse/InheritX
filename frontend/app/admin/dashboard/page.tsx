"use client";
import { useEffect, useState } from "react";
import { MetricsCards } from "@/components/admin/metrics/MetricsCards";
import { TVLChart } from "@/components/admin/metrics/TVLChart";
import { AssetDistributionChart } from "@/components/admin/metrics/AssetDistributionChart";
import { EmergencyPausePanel } from "@/components/admin/EmergencyPausePanel";
import { adminAPI, AdminMetrics } from "@/app/lib/api/admin";
import { mockAdminMetrics } from "@/lib/mockAdminUsers";
import { AlertCircle } from "lucide-react";

export default function AdminDashboardPage() {
  const [metrics, setMetrics] = useState<AdminMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [usingMock, setUsingMock] = useState(false);

  useEffect(() => {
    adminAPI
      .getMetrics()
      .then((data) => setMetrics(data))
      .catch(() => {
        setMetrics(mockAdminMetrics);
        setUsingMock(true);
      })
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className="animate-fade-in space-y-6 max-w-5xl">
      <div>
        <h1 className="text-2xl font-semibold text-white">Dashboard</h1>
        <p className="text-sm text-slate-500 mt-1">Platform-wide metrics and asset overview</p>
      </div>

      {usingMock && (
        <div className="rounded-xl border border-yellow-500/20 bg-yellow-500/5 px-4 py-3 flex items-start gap-2">
          <AlertCircle size={14} className="text-yellow-400 mt-0.5 flex-shrink-0" />
          <p className="text-xs text-yellow-400/80">
            Backend unavailable — showing mock data. In production, metrics are fetched from{" "}
            <code className="font-mono">/api/admin/metrics</code>.
          </p>
        </div>
      )}

      <EmergencyPausePanel />

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
  );
}
