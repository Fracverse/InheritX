"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { useWallet } from "@/context/WalletContext";
import { type Plan } from "@/app/lib/api/plans";
import InactivityTimerCard from "@/components/plans/InactivityTimerCard";
import { mockStore } from "@/lib/mockStore";
import { formatAddress } from "@/util/address";
import {
  TrendingUp,
  FileText,
  Users,
  PlusCircle,
  ArrowRight,
  Zap,
  ShieldCheck,
  ChevronRight,
  Activity,
  AlertCircle,
} from "lucide-react";

function StatCard({
  label,
  value,
  sub,
  icon: Icon,
  color,
  loading,
}: {
  label: string;
  value: string;
  sub?: string;
  icon: React.ElementType;
  color: string;
  loading?: boolean;
}) {
  return (
    <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5 flex gap-4 items-start hover:bg-white/[0.05] transition-colors">
      <div className={`w-10 h-10 rounded-xl flex items-center justify-center flex-shrink-0 ${color}`}>
        <Icon size={18} />
      </div>
      <div className="min-w-0">
        <p className="text-xs text-slate-500 font-medium uppercase tracking-wider">{label}</p>
        {loading ? (
          <div className="mt-1.5 h-6 w-24 rounded-md bg-white/5 animate-pulse" />
        ) : (
          <p className="text-xl font-semibold text-white mt-0.5">{value}</p>
        )}
        {sub && <p className="text-xs text-slate-500 mt-0.5">{sub}</p>}
      </div>
    </div>
  );
}

function PlanStatusBadge({ status }: { status: string }) {
  const map: Record<string, { label: string; className: string }> = {
    ACTIVE: { label: "Active", className: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20" },
    PENDING: { label: "Pending", className: "bg-yellow-500/10 text-yellow-400 border-yellow-500/20" },
    TRIGGERED: { label: "Triggered", className: "bg-orange-500/10 text-orange-400 border-orange-500/20" },
    COMPLETED: { label: "Completed", className: "bg-blue-500/10 text-blue-400 border-blue-500/20" },
  };
  const s = map[status?.toUpperCase()] ?? { label: status, className: "bg-slate-500/10 text-slate-400 border-slate-500/20" };
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium border ${s.className}`}>
      {s.label}
    </span>
  );
}

export default function AssetOwnerPage() {
  const { isConnected, address } = useWallet();
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const mockPlans = mockStore.getPlans();
    setPlans(mockPlans);
    setLoading(false);
  }, [isConnected, address]);

  const activePlans = plans.filter((p) => p.status?.toUpperCase() === "ACTIVE");
  const totalYield = plans.reduce((s, p) => s + (p.accrued_yield ?? 0), 0);
  const totalBeneficiaries = plans.reduce((s, p) => s + (p.beneficiaries?.length ?? 0), 0);

  const quickActions = [
    { label: "Create Plan", href: "/asset-owner/plans/create", icon: PlusCircle, desc: "Set up a new inheritance plan" },
    { label: "Send Ping", href: "/asset-owner/ping", icon: Zap, desc: "Prove you're still active" },
    { label: "Verify KYC", href: "/asset-owner/kyc", icon: ShieldCheck, desc: "Complete identity verification" },
    { label: "View Plans", href: "/asset-owner/plans", icon: FileText, desc: "Manage all your plans" },
  ];

  return (
    <div className="animate-fade-in space-y-6 max-w-5xl">
      {/* Page Title */}
      <div>
        <h1 className="text-2xl font-semibold text-white">Overview</h1>
        <p className="text-sm text-slate-500 mt-1">
          {isConnected && address
            ? `Welcome back, ${formatAddress(address)}`
            : "Connect your wallet to get started"}
        </p>
      </div>

      {/* Demo Mode Notice */}
      <div className="rounded-2xl border border-[#33C5E0]/20 bg-[#33C5E0]/5 p-4 flex items-center gap-3">
        <span className="w-2 h-2 rounded-full bg-[#33C5E0] animate-pulse" />
        <p className="text-xs text-[#33C5E0]">
          <strong>InheritX Demo Mode:</strong> API integrations are decoupled. You can explore, create, edit, and ping plans locally.
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Active Plans" value={loading ? "—" : String(activePlans.length)} icon={FileText} color="bg-[#33C5E0]/10 text-[#33C5E0]" loading={loading} />
        <StatCard label="Total Plans" value={loading ? "—" : String(plans.length)} sub="All statuses" icon={Activity} color="bg-violet-500/10 text-violet-400" loading={loading} />
        <StatCard label="Yield Earned" value={loading ? "—" : `$${totalYield.toFixed(2)}`} sub="Accrued across plans" icon={TrendingUp} color="bg-emerald-500/10 text-emerald-400" loading={loading} />
        <StatCard label="Beneficiaries" value={loading ? "—" : String(totalBeneficiaries)} sub="Across all plans" icon={Users} color="bg-orange-500/10 text-orange-400" loading={loading} />
      </div>

      {/* Proof-of-life controls */}
      {!loading && activePlans.length > 0 && (
        <section className="space-y-3" aria-labelledby="proof-of-life-heading">
          <div className="flex items-end justify-between gap-3">
            <div>
              <h2 id="proof-of-life-heading" className="text-sm font-semibold text-white">
                Proof of Life
              </h2>
              <p className="text-xs text-slate-500 mt-1">
                Sign a ping to reset each plan&apos;s inactivity timer.
              </p>
            </div>
            <Link href="/asset-owner/ping" className="text-xs text-[#33C5E0] hover:underline flex items-center gap-1">
              View all <ChevronRight size={12} />
            </Link>
          </div>
          <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
            {activePlans.map((plan) => (
              <InactivityTimerCard key={plan.id} planId={plan.id} />
            ))}
          </div>
        </section>
      )}

      {/* Main Content Row */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Recent Plans */}
        <div className="lg:col-span-2 rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-semibold text-white">Recent Plans</h2>
            <Link href="/asset-owner/plans" className="text-xs text-[#33C5E0] hover:underline flex items-center gap-1">
              View all <ChevronRight size={12} />
            </Link>
          </div>

          {loading ? (
            <div className="space-y-3">
              {[1, 2, 3].map((i) => (
                <div key={i} className="h-14 rounded-xl bg-white/5 animate-pulse" />
              ))}
            </div>
          ) : plans.length === 0 ? (
            <div className="py-12 flex flex-col items-center gap-3 text-center">
              <div className="w-12 h-12 rounded-2xl bg-white/5 flex items-center justify-center">
                <FileText size={20} className="text-slate-500" />
              </div>
              <p className="text-sm text-slate-500">No plans yet</p>
              <Link
                href="/asset-owner/plans/create"
                className="text-xs text-[#33C5E0] hover:underline"
              >
                Create your first plan →
              </Link>
            </div>
          ) : (
            <div className="space-y-2">
              {plans.slice(0, 5).map((plan) => (
                <Link
                  key={plan.id}
                  href={`/asset-owner/plans/${plan.id}`}
                  className="flex items-center gap-3 p-3 rounded-xl hover:bg-white/5 transition-colors group"
                >
                  <div className="w-9 h-9 rounded-lg bg-[#33C5E0]/10 border border-[#33C5E0]/20 flex items-center justify-center flex-shrink-0">
                    <FileText size={14} className="text-[#33C5E0]" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-white truncate">
                      {plan.token_address ?? "USDC"} Plan
                    </p>
                    <p className="text-xs text-slate-500">
                      {plan.beneficiaries?.length ?? 0} beneficiar{plan.beneficiaries?.length === 1 ? "y" : "ies"} · ${Number(plan.amount ?? 0).toLocaleString()}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <PlanStatusBadge status={plan.status ?? "ACTIVE"} />
                    <ChevronRight size={14} className="text-slate-600 group-hover:text-slate-400 transition-colors" />
                  </div>
                </Link>
              ))}
            </div>
          )}
        </div>

        {/* Quick Actions */}
        <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
          <h2 className="text-sm font-semibold text-white mb-4">Quick Actions</h2>
          <div className="space-y-2">
            {quickActions.map((action) => {
              const Icon = action.icon;
              return (
                <Link
                  key={action.href}
                  href={action.href}
                  className="flex items-center gap-3 p-3 rounded-xl hover:bg-white/5 border border-transparent hover:border-white/[0.08] transition-all group"
                >
                  <div className="w-8 h-8 rounded-lg bg-[#33C5E0]/10 flex items-center justify-center flex-shrink-0">
                    <Icon size={14} className="text-[#33C5E0]" />
                  </div>
                  <div className="min-w-0">
                    <p className="text-sm text-white font-medium">{action.label}</p>
                    <p className="text-xs text-slate-500 truncate">{action.desc}</p>
                  </div>
                  <ArrowRight size={14} className="ml-auto text-slate-600 group-hover:text-[#33C5E0] transition-colors flex-shrink-0" />
                </Link>
              );
            })}
          </div>
        </div>
      </div>

      {/* Inactivity Alert */}
      {activePlans.length > 0 && (
        <div className="rounded-2xl border border-yellow-500/20 bg-yellow-500/5 p-4 flex items-start gap-3">
          <AlertCircle size={16} className="text-yellow-400 mt-0.5 flex-shrink-0" />
          <div>
            <p className="text-sm font-medium text-yellow-300">Proof of Life Required</p>
            <p className="text-xs text-yellow-300/70 mt-0.5">
              You have {activePlans.length} active plan{activePlans.length > 1 ? "s" : ""}. Send a ping periodically to prevent triggering the grace period.
            </p>
            <Link href="/asset-owner/ping" className="inline-flex items-center gap-1 mt-2 text-xs text-yellow-400 font-medium hover:underline">
              Send Ping <Zap size={11} />
            </Link>
          </div>
        </div>
      )}
    </div>
  );
}
