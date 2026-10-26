"use client";

import { useState, useEffect, useCallback } from "react";
import Link from "next/link";
import { useWallet } from "@/context/WalletContext";
import { plansAPI, type Plan } from "@/app/lib/api/plans";
import { getPlans } from "@/lib/api/dataSource";
import {
  PlusCircle,
  Search,
  FileText,
  ChevronRight,
  Edit3,
  Zap,
  Filter,
  TrendingUp,
  Users,
  Clock,
  MoreHorizontal,
  RefreshCw,
  Wallet,
} from "lucide-react";

function StatusBadge({ status }: { status: string }) {
  const map: Record<string, string> = {
    ACTIVE: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
    PENDING: "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
    TRIGGERED: "bg-orange-500/10 text-orange-400 border-orange-500/20",
    COMPLETED: "bg-blue-500/10 text-blue-400 border-blue-500/20",
    INACTIVE: "bg-slate-500/10 text-slate-400 border-slate-500/20",
  };
  const cls = map[status?.toUpperCase()] ?? map.INACTIVE;
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium border ${cls}`}>
      {status ?? "Unknown"}
    </span>
  );
}

function PlanCard({ plan }: { plan: Plan }) {
  const benefCount = plan.beneficiaries?.length ?? 0;
  const amount = Number(plan.amount ?? 0);
  const yield$ = plan.accrued_yield ?? 0;

  return (
    <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5 hover:bg-white/[0.05] hover:border-white/[0.12] transition-all group">
      {/* Top Row */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-[#33C5E0]/10 border border-[#33C5E0]/20 flex items-center justify-center flex-shrink-0">
            <FileText size={16} className="text-[#33C5E0]" />
          </div>
          <div>
            <p className="text-sm font-semibold text-white">
              {plan.token_address ?? "USDC"} Plan
            </p>
            <p className="text-[11px] text-slate-500 font-mono mt-0.5">
              #{plan.id?.slice(0, 8)}
            </p>
          </div>
        </div>
        <StatusBadge status={plan.status ?? "ACTIVE"} />
      </div>

      {/* Stats Row */}
      <div className="grid grid-cols-3 gap-3 mb-4">
        <div className="text-center p-2.5 rounded-xl bg-white/[0.03] border border-white/[0.05]">
          <p className="text-xs text-slate-500">Amount</p>
          <p className="text-sm font-semibold text-white mt-0.5">${amount.toLocaleString()}</p>
        </div>
        <div className="text-center p-2.5 rounded-xl bg-white/[0.03] border border-white/[0.05]">
          <p className="text-xs text-slate-500">Yield</p>
          <p className="text-sm font-semibold text-emerald-400 mt-0.5">+${yield$.toFixed(2)}</p>
        </div>
        <div className="text-center p-2.5 rounded-xl bg-white/[0.03] border border-white/[0.05]">
          <p className="text-xs text-slate-500">Heirs</p>
          <p className="text-sm font-semibold text-white mt-0.5">{benefCount}</p>
        </div>
      </div>

      {/* Beneficiary Allocation Bar */}
      {benefCount > 0 && (
        <div className="mb-4">
          <div className="flex gap-0.5 h-1.5 rounded-full overflow-hidden">
            {plan.beneficiaries!.map((b, i) => {
              const pct = (b.allocation_bps ?? 10000 / benefCount) / 100;
              const colors = ["bg-[#33C5E0]", "bg-violet-400", "bg-emerald-400", "bg-orange-400", "bg-pink-400"];
              return (
                <div
                  key={i}
                  className={`${colors[i % colors.length]} h-full`}
                  style={{ width: `${pct}%` }}
                />
              );
            })}
          </div>
          <div className="flex gap-3 mt-1.5">
            {plan.beneficiaries!.slice(0, 3).map((b, i) => {
              const pct = (b.allocation_bps ?? 10000 / benefCount) / 100;
              const colors = ["text-[#33C5E0]", "text-violet-400", "text-emerald-400"];
              return (
                <span key={i} className={`text-[10px] ${colors[i % colors.length]}`}>
                  {b.wallet_address?.slice(0, 6)}… {pct.toFixed(0)}%
                </span>
              );
            })}
            {benefCount > 3 && <span className="text-[10px] text-slate-500">+{benefCount - 3} more</span>}
          </div>
        </div>
      )}

      {/* Grace Period */}
      {plan.grace_period_seconds != null && (
        <div className="flex items-center gap-2 mb-4 text-xs text-slate-500">
          <Clock size={11} />
          <span>Grace period: {Math.round(plan.grace_period_seconds / 86400)}d</span>
          {plan.earn_yield && (
            <>
              <span className="mx-1">·</span>
              <TrendingUp size={11} className="text-emerald-400" />
              <span className="text-emerald-400">{(plan.yield_rate_bps ?? 0) / 100}% APY</span>
            </>
          )}
        </div>
      )}

      {/* Actions */}
      <div className="flex gap-2 pt-3 border-t border-white/5">
        <Link
          href={`/asset-owner/plans/${plan.id}`}
          className="flex-1 flex items-center justify-center gap-1.5 py-2 rounded-xl text-xs font-medium bg-[#33C5E0]/10 text-[#33C5E0] border border-[#33C5E0]/20 hover:bg-[#33C5E0]/20 transition-colors"
        >
          View <ChevronRight size={11} />
        </Link>
        <Link
          href={`/asset-owner/plans/${plan.id}/edit`}
          className="flex-1 flex items-center justify-center gap-1.5 py-2 rounded-xl text-xs font-medium bg-white/5 text-slate-300 border border-white/10 hover:bg-white/10 transition-colors"
        >
          <Edit3 size={11} /> Edit
        </Link>
        <Link
          href="/asset-owner/ping"
          className="px-3 py-2 rounded-xl text-xs font-medium bg-white/5 text-slate-300 border border-white/10 hover:bg-yellow-500/10 hover:text-yellow-400 hover:border-yellow-500/20 transition-colors"
          title="Send Ping"
        >
          <Zap size={11} />
        </Link>
      </div>
    </div>
  );
}

const STATUS_FILTERS = ["All", "Active", "Pending", "Triggered", "Completed"];

export default function PlansPage() {
  const { isConnected, address, openModal } = useWallet();
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState("All");
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPlans = useCallback(async () => {
    try {
      setPlans(await getPlans(address ?? undefined));
      setError(null);
    } catch (err) {
      setPlans([]);
      setError(err instanceof Error ? err.message : "Failed to load plans.");
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [address]);

  useEffect(() => { fetchPlans(); }, [fetchPlans]);

  const handleRefresh = () => { setRefreshing(true); fetchPlans(); };

  const filtered = plans.filter((p) => {
    const matchStatus = statusFilter === "All" || p.status?.toUpperCase() === statusFilter.toUpperCase();
    const matchSearch = !search || p.id?.toLowerCase().includes(search.toLowerCase()) || p.token_address?.toLowerCase().includes(search.toLowerCase());
    return matchStatus && matchSearch;
  });

  return (
    <div className="animate-fade-in space-y-6 max-w-5xl">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-white">My Plans</h1>
          <p className="text-sm text-slate-500 mt-1">
            {loading ? "Loading..." : `${plans.length} plan${plans.length !== 1 ? "s" : ""} found`}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleRefresh}
            disabled={refreshing || loading}
            className="p-2 rounded-xl border border-white/10 text-slate-400 hover:text-white hover:bg-white/5 transition-colors disabled:opacity-40"
            aria-label="Refresh"
          >
            <RefreshCw size={15} className={refreshing ? "animate-spin" : ""} />
          </button>
          <Link
            href="/asset-owner/plans/create"
            className="flex items-center gap-2 px-4 py-2 rounded-xl bg-[#33C5E0] text-black text-sm font-semibold hover:bg-[#33C5E0]/90 transition-colors"
          >
            <PlusCircle size={14} />
            New Plan
          </Link>
        </div>
      </div>

      {/* Filters */}
      {error && <div role="alert" className="rounded-xl border border-red-500/20 bg-red-500/5 p-4 text-xs text-red-300">{error}</div>}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="relative flex-1">
            <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search by plan ID or token…"
              className="w-full pl-9 pr-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-[#33C5E0]/40 transition-colors"
            />
          </div>
          <div className="flex items-center gap-1 p-1 rounded-xl bg-white/5 border border-white/10 flex-shrink-0 flex-wrap">
            {STATUS_FILTERS.map((f) => (
              <button
                key={f}
                onClick={() => setStatusFilter(f)}
                className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                  statusFilter === f
                    ? "bg-[#33C5E0] text-black"
                    : "text-slate-400 hover:text-white"
                }`}
              >
                {f}
              </button>
            ))}
          </div>
        </div>

      {/* Plans Grid */}
      {loading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (
            <div key={i} className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5 h-64 animate-pulse" />
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className="py-20 flex flex-col items-center gap-4 text-center">
          <div className="w-16 h-16 rounded-2xl bg-white/5 flex items-center justify-center">
            <FileText size={24} className="text-slate-600" />
          </div>
          <div>
            <p className="text-sm font-medium text-slate-300">No plans found</p>
            <p className="text-xs text-slate-500 mt-1">
              {search || statusFilter !== "All" ? "Try adjusting your filters." : "Get started by creating your first inheritance plan."}
            </p>
          </div>
          {!search && statusFilter === "All" && (
            <Link
              href="/asset-owner/plans/create"
              className="flex items-center gap-2 px-4 py-2 rounded-xl bg-[#33C5E0] text-black text-sm font-semibold"
            >
              <PlusCircle size={14} /> Create Plan
            </Link>
          )}
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {filtered.map((plan) => (
            <PlanCard key={plan.id} plan={plan} />
          ))}
        </div>
      )}
    </div>
  );
}
