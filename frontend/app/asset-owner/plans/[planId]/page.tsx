"use client";

import { useState, useEffect } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { plansAPI, type Plan } from "@/app/lib/api/plans";
import { getPlan } from "@/lib/api/dataSource";
import {
  ArrowLeft,
  FileText,
  Edit3,
  Zap,
  HandCoins,
  Clock,
  TrendingUp,
  Users,
  DollarSign,
  ShieldCheck,
  Copy,
  CheckCircle2,
  AlertCircle,
  Activity,
  Calendar,
  Hash,
  UserPlus,
} from "lucide-react";
import { FamilyInviteModal } from "@/components/plans/FamilyInviteModal";

function StatusBadge({ status }: { status: string }) {
  const map: Record<string, string> = {
    ACTIVE: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
    PENDING: "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
    TRIGGERED: "bg-orange-500/10 text-orange-400 border-orange-500/20",
    COMPLETED: "bg-blue-500/10 text-blue-400 border-blue-500/20",
  };
  const cls = map[status?.toUpperCase()] ?? "bg-slate-500/10 text-slate-400 border-slate-500/20";
  return (
    <span className={`inline-flex items-center px-2.5 py-1 rounded-lg text-xs font-semibold border ${cls}`}>
      {status}
    </span>
  );
}

function InfoRow({ label, value, mono }: { label: string; value: React.ReactNode; mono?: boolean }) {
  return (
    <div className="flex items-start justify-between py-3 border-b border-white/5 last:border-0 gap-4">
      <span className="text-xs text-slate-500 flex-shrink-0 pt-0.5">{label}</span>
      <span className={`text-sm text-right ${mono ? "font-mono text-[#33C5E0]" : "text-slate-200 font-medium"}`}>
        {value}
      </span>
    </div>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <button onClick={copy} className="ml-1 text-slate-500 hover:text-[#33C5E0] transition-colors" title="Copy">
      {copied ? <CheckCircle2 size={12} className="text-emerald-400" /> : <Copy size={12} />}
    </button>
  );
}

export default function PlanDetailPage() {
  const params = useParams();
  const router = useRouter();
  const planId = params.planId as string;
  const [plan, setPlan] = useState<Plan | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [familyInviteOpen, setFamilyInviteOpen] = useState(false);

  useEffect(() => {
    if (!planId) return;
    let cancelled = false;
    setLoading(true);
    getPlan(planId)
      .then((nextPlan) => {
        if (cancelled) return;
        if (nextPlan) setPlan(nextPlan);
        else setError("Plan not found");
      })
      .catch((err) => { if (!cancelled) setError(err instanceof Error ? err.message : "Failed to load plan."); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [planId]);

  if (loading) {
    return (
      <div className="animate-fade-in space-y-4 max-w-3xl">
        <div className="h-8 w-40 rounded-xl bg-white/5 animate-pulse" />
        <div className="h-48 rounded-2xl bg-white/5 animate-pulse" />
        <div className="h-64 rounded-2xl bg-white/5 animate-pulse" />
      </div>
    );
  }

  if (error || !plan) {
    return (
      <div className="flex flex-col items-center gap-4 py-20 text-center">
        <AlertCircle size={32} className="text-red-400" />
        <p className="text-sm text-slate-300">{error ?? "Plan not found."}</p>
        <button onClick={() => router.back()} className="text-xs text-[#33C5E0] hover:underline">← Go back</button>
      </div>
    );
  }

  const amount = Number(plan.amount ?? 0);
  const benefCount = plan.beneficiaries?.length ?? 0;
  const graceDays = plan.grace_period_seconds ? Math.round(plan.grace_period_seconds / 86400) : 0;

  return (
    <div className="animate-fade-in space-y-5 max-w-3xl">
      {/* Back + Header */}
      <div className="flex items-center gap-3">
        <button
          onClick={() => router.back()}
          className="p-2 rounded-xl border border-white/10 text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
          aria-label="Go back"
        >
          <ArrowLeft size={15} />
        </button>
        <div className="flex-1">
          <h1 className="text-xl font-semibold text-white flex items-center gap-2">
            {plan.token_address ?? "USDC"} Plan
            <StatusBadge status={plan.status ?? "ACTIVE"} />
          </h1>
          <p className="text-xs text-slate-500 font-mono mt-0.5 flex items-center gap-1">
            {plan.id}
            <CopyButton text={plan.id ?? ""} />
          </p>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => setFamilyInviteOpen(true)}
          className="flex items-center gap-1.5 rounded-xl border border-violet-500/20 bg-violet-500/10 px-3 py-2 text-xs font-medium text-violet-300 transition-colors hover:bg-violet-500/20"
        >
          <UserPlus size={12} /> Invite Family
        </button>
        <Link
          href={`/asset-owner/plans/${plan.id}/edit`}
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-medium bg-[#33C5E0]/10 text-[#33C5E0] border border-[#33C5E0]/20 hover:bg-[#33C5E0]/20 transition-colors"
        >
          <Edit3 size={12} /> Edit Plan
        </Link>
        <Link
          href="/asset-owner/ping"
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-medium bg-yellow-500/10 text-yellow-400 border border-yellow-500/20 hover:bg-yellow-500/20 transition-colors"
        >
          <Zap size={12} /> Send Ping
        </Link>
        <Link
          href="/asset-owner/plans/claim"
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-medium bg-white/5 text-slate-300 border border-white/10 hover:bg-white/10 transition-colors"
        >
          <HandCoins size={12} /> Claim
        </Link>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {[
          { label: "Amount", value: `$${amount.toLocaleString()}`, icon: DollarSign, color: "text-[#33C5E0]" },
          { label: "Yield Earned", value: `+$${(plan.accrued_yield ?? 0).toFixed(2)}`, icon: TrendingUp, color: "text-emerald-400" },
          { label: "Beneficiaries", value: String(benefCount), icon: Users, color: "text-violet-400" },
          { label: "Grace Period", value: `${graceDays}d`, icon: Clock, color: "text-orange-400" },
        ].map(({ label, value, icon: Icon, color }) => (
          <div key={label} className="rounded-xl bg-white/[0.03] border border-white/[0.08] p-4 text-center">
            <Icon size={16} className={`${color} mx-auto mb-1.5`} />
            <p className="text-lg font-semibold text-white">{value}</p>
            <p className="text-[10px] text-slate-500 mt-0.5">{label}</p>
          </div>
        ))}
      </div>

      {/* Plan Details */}
      <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
        <h2 className="text-sm font-semibold text-white mb-2">Plan Details</h2>
        <InfoRow label="Token" value={plan.token_address ?? "USDC"} />
        <InfoRow label="Owner" value={<span className="flex items-center gap-1">{(plan.owner_address ?? "—").slice(0, 20)}…<CopyButton text={plan.owner_address ?? ""} /></span>} mono />
        <InfoRow label="Amount" value={`$${amount.toLocaleString()} USDC`} />
        <InfoRow label="Yield Rate" value={plan.earn_yield ? `${(plan.yield_rate_bps ?? 0) / 100}% APY` : "Disabled"} />
        <InfoRow label="Grace Period" value={`${graceDays} days (${plan.grace_period_seconds?.toLocaleString() ?? 0}s)`} />
        <InfoRow label="Status" value={<StatusBadge status={plan.status ?? "ACTIVE"} />} />
        {plan.created_at && (
          <InfoRow
            label="Created"
            value={new Date(plan.created_at).toLocaleDateString("en-GB", { day: "numeric", month: "short", year: "numeric" })}
          />
        )}
      </div>

      {/* Beneficiaries */}
      <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
        <h2 className="text-sm font-semibold text-white mb-4">
          Beneficiaries <span className="text-slate-600 font-normal">({benefCount})</span>
        </h2>
        {benefCount === 0 ? (
          <p className="text-xs text-slate-500 py-4 text-center">No beneficiaries configured.</p>
        ) : (
          <div className="space-y-2">
            {plan.beneficiaries!.map((b, i) => {
              const pct = (b.allocation_bps ?? 0) / 100;
              const colors = ["bg-[#33C5E0]", "bg-violet-400", "bg-emerald-400", "bg-orange-400", "bg-pink-400"];
              return (
                <div key={i} className="flex items-center gap-3 p-3 rounded-xl bg-white/[0.03] border border-white/5">
                  <div className={`w-2 h-8 rounded-full ${colors[i % colors.length]} opacity-80 flex-shrink-0`} />
                  <div className="flex-1 min-w-0">
                    <p className="text-xs font-mono text-[#33C5E0] truncate flex items-center gap-1">
                      {b.wallet_address}
                      <CopyButton text={b.wallet_address ?? ""} />
                    </p>
                    {b.fiat_anchor_info && (
                      <p className="text-[10px] text-slate-500 mt-0.5">Fiat: {b.fiat_anchor_info}</p>
                    )}
                  </div>
                  <div className="text-right flex-shrink-0">
                    <p className="text-sm font-semibold text-white">{pct.toFixed(1)}%</p>
                    <p className="text-[10px] text-slate-500">{b.allocation_bps ?? 0} bps</p>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Allocation Visual */}
      {benefCount > 0 && (
        <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
          <h2 className="text-sm font-semibold text-white mb-3">Allocation Breakdown</h2>
          <div className="flex gap-1 h-4 rounded-full overflow-hidden mb-3">
            {plan.beneficiaries!.map((b, i) => {
              const pct = (b.allocation_bps ?? 0) / 100;
              const colors = ["bg-[#33C5E0]", "bg-violet-400", "bg-emerald-400", "bg-orange-400", "bg-pink-400"];
              return <div key={i} className={`${colors[i % colors.length]} h-full`} style={{ width: `${pct}%` }} />;
            })}
          </div>
          <div className="flex flex-wrap gap-3">
            {plan.beneficiaries!.map((b, i) => {
              const pct = (b.allocation_bps ?? 0) / 100;
              const colors = ["text-[#33C5E0]", "text-violet-400", "text-emerald-400", "text-orange-400", "text-pink-400"];
              return (
                <div key={i} className="flex items-center gap-1.5">
                  <span className={`w-2 h-2 rounded-full ${colors[i % colors.length].replace("text-", "bg-")}`} />
                  <span className={`text-xs ${colors[i % colors.length]}`}>
                    {b.wallet_address?.slice(0, 8)}… ({pct.toFixed(0)}%)
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}
      <FamilyInviteModal
        planId={plan.id}
        open={familyInviteOpen}
        onClose={() => setFamilyInviteOpen(false)}
      />
    </div>
  );
}
