"use client";

import { useState, useMemo, useEffect } from "react";
import Link from "next/link";
import { FileText, Search, ArrowDownToLine, Eye, RefreshCw, Trash2, ArrowUpRight } from "lucide-react";
import { adminAPI } from "@/app/lib/api/admin";
import { describeAdminError } from "@/lib/adminErrors";
import { AdminErrorBanner } from "@/components/admin/AdminErrorBanner";
import type { AdminPlan } from "@/lib/adminTypes";

export default function AdminPlansPage() {
  const [plans, setPlans] = useState<AdminPlan[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [terminatingIds, setTerminatingIds] = useState<Set<string>>(new Set());
  const [reloadKey, setReloadKey] = useState(0);
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState("ALL");

  useEffect(() => {
    let cancelled = false;
    adminAPI
      .listPlans()
      .then((data) => {
        if (!cancelled) setPlans(data);
      })
      .catch((err) => {
        if (!cancelled) setLoadError(describeAdminError(err, "Loading plans"));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [reloadKey]);

  const retryLoad = () => {
    setLoadError(null);
    setLoading(true);
    setReloadKey((k) => k + 1);
  };

  const filtered = useMemo(() => {
    return plans.filter((p) => {
      const matchesSearch =
        p.id.toLowerCase().includes(search.toLowerCase()) ||
        p.owner.toLowerCase().includes(search.toLowerCase());
      const matchesStatus = statusFilter === "ALL" || p.status === statusFilter;
      return matchesSearch && matchesStatus;
    });
  }, [plans, search, statusFilter]);

  const handleDownloadPDF = (planId: string) => {
    // Standard PDF download mock trigger pointing to backend report API stub
    window.open(`/api/plans/${planId}/report`, "_blank");
  };

  const handleTerminatePlan = async (planId: string) => {
    if (!confirm(`Are you sure you want to terminate plan ${planId}?`)) return;

    setActionError(null);
    setTerminatingIds((prev) => new Set(prev).add(planId));
    try {
      // The plan row is replaced with the server's response; nothing is assumed locally.
      const updated = await adminAPI.terminatePlan(planId);
      setPlans((prev) => prev.map((p) => (p.id === planId ? updated : p)));
    } catch (err) {
      setActionError(describeAdminError(err, `Terminating plan ${planId}`));
    } finally {
      setTerminatingIds((prev) => {
        const next = new Set(prev);
        next.delete(planId);
        return next;
      });
    }
  };

  return (
    <div className="animate-fade-in space-y-6 max-w-5xl">
      <div>
        <h1 className="text-2xl font-semibold text-white">Plans Oversight</h1>
        <p className="text-sm text-slate-500 mt-1">
          Monitor and manage all inheritance plans deployed on InheritX.
        </p>
      </div>

      {loadError && <AdminErrorBanner message={loadError} onRetry={retryLoad} />}
      {actionError && (
        <AdminErrorBanner message={actionError} onDismiss={() => setActionError(null)} />
      )}

      {/* Filter bar */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="relative flex-1">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by plan ID or owner address…"
            className="w-full pl-9 pr-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-violet-500/50 transition-colors"
          />
        </div>
        <div className="flex items-center gap-1 p-1 rounded-xl bg-white/5 border border-white/10 flex-shrink-0">
          {["ALL", "ACTIVE", "TRIGGERED", "COMPLETED"].map((f) => (
            <button
              key={f}
              onClick={() => setStatusFilter(f)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                statusFilter === f ? "bg-violet-500 text-white" : "text-slate-400 hover:text-white"
              }`}
            >
              {f}
            </button>
          ))}
        </div>
      </div>

      {/* Table */}
      {!loadError && (
        <div className="bg-white/[0.02] border border-white/5 rounded-2xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left text-xs">
              <thead>
                <tr className="border-b border-white/5 bg-white/[0.02] text-slate-400 font-medium uppercase tracking-wider">
                  <th className="p-4">Plan ID</th>
                  <th className="p-4">Owner Address</th>
                  <th className="p-4">Total Value</th>
                  <th className="p-4">Yield APY</th>
                  <th className="p-4">Heirs</th>
                  <th className="p-4">Status</th>
                  <th className="p-4">Last Active</th>
                  <th className="p-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5">
                {filtered.map((plan) => (
                  <tr key={plan.id} className="hover:bg-white/[0.01] transition-colors text-slate-300">
                    <td className="p-4 font-mono font-medium text-violet-400">
                      <Link href={`/admin/plans/${plan.id}`} className="hover:underline flex items-center gap-1">
                        {plan.id}
                        <ArrowUpRight size={10} />
                      </Link>
                    </td>
                    <td className="p-4 font-mono text-slate-400">{plan.owner}</td>
                    <td className="p-4 font-semibold text-white">
                      ${plan.amount.toLocaleString()} <span className="text-[10px] text-slate-500">{plan.token}</span>
                    </td>
                    <td className="p-4 text-emerald-400 font-medium">{(plan.yieldRateBps / 100).toFixed(2)}%</td>
                    <td className="p-4">{plan.heirs}</td>
                    <td className="p-4">
                      <span
                        className={`inline-flex items-center px-2 py-0.5 rounded-md text-[10px] font-semibold border ${
                          plan.status === "ACTIVE"
                            ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
                            : plan.status === "TRIGGERED"
                            ? "bg-orange-500/10 text-orange-400 border-orange-500/20"
                            : "bg-slate-500/10 text-slate-400 border-slate-500/20"
                        }`}
                      >
                        {plan.status}
                      </span>
                    </td>
                    <td className="p-4 text-slate-400">
                      {new Date(plan.lastPing).toLocaleDateString()}
                    </td>
                    <td className="p-4 text-right flex justify-end gap-2">
                      <button
                        onClick={() => handleDownloadPDF(plan.id)}
                        className="p-1.5 rounded-lg border border-white/10 hover:border-violet-500/30 hover:bg-violet-500/10 text-slate-400 hover:text-violet-300 transition-colors"
                        title="Download PDF Report"
                      >
                        <ArrowDownToLine size={13} />
                      </button>
                      <button
                        onClick={() => handleTerminatePlan(plan.id)}
                        disabled={plan.status === "COMPLETED" || terminatingIds.has(plan.id)}
                        className="p-1.5 rounded-lg border border-white/10 hover:border-red-500/30 hover:bg-red-500/10 text-slate-400 hover:text-red-400 transition-colors disabled:opacity-30 disabled:hover:bg-transparent disabled:hover:border-white/10"
                        title="Force Complete / Terminate Plan"
                      >
                        <Trash2 size={13} />
                      </button>
                    </td>
                  </tr>
                ))}
                {loading && (
                  <tr>
                    <td colSpan={8} role="status" className="p-8 text-center text-slate-500">
                      Loading plans…
                    </td>
                  </tr>
                )}
                {!loading && filtered.length === 0 && (
                  <tr>
                    <td colSpan={8} className="p-8 text-center text-slate-500">
                      No matching plans found.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
