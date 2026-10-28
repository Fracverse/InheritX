"use client";

import { useState, useEffect } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, FileText, ArrowDownToLine, Zap, Trash2, ShieldCheck, DollarSign, TrendingUp, Users, Clock, AlertCircle } from "lucide-react";

export default function AdminPlanDetailPage() {
  const params = useParams();
  const router = useRouter();
  const planId = params.planId as string;
  const [loading, setLoading] = useState(true);

  // Simulated plan detail data for admin view
  const [plan, setPlan] = useState<any>(null);

  useEffect(() => {
    if (!planId) return;
    // Mock fetch for administrative detail inspection
    setTimeout(() => {
      setPlan({
        id: planId,
        owner: "GDRT7YV3XP9P6HJK82Y8K2M192SDFPWER9A",
        token: "USDC",
        amount: 32000,
        yieldRateBps: 450,
        status: "ACTIVE",
        gracePeriodSeconds: 2592000, // 30 days
        earnYield: true,
        lastPing: "2026-07-14T10:00:00Z",
        createdAt: "2025-11-15T08:30:00Z",
        beneficiaries: [
          { address: "GDX74T...3P2X", allocationBps: 6000, fiatAnchor: "anchor-ngn" },
          { address: "GBN23P...8K9T", allocationBps: 4000, fiatAnchor: "anchor-kes" },
        ],
      });
      setLoading(false);
    }, 400);
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

  const handleDownloadPDF = () => {
    window.open(`/api/plans/${planId}/report`, "_blank");
  };

  return (
    <div className="animate-fade-in space-y-6 max-w-3xl">
      <div className="flex items-center gap-3">
        <button
          onClick={() => router.back()}
          className="p-2 rounded-xl border border-white/10 text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
          aria-label="Back"
        >
          <ArrowLeft size={15} />
        </button>
        <div>
          <h1 className="text-xl font-semibold text-white">Plan Inspection</h1>
          <p className="text-xs text-slate-500 font-mono mt-0.5">{planId}</p>
        </div>
      </div>

      {/* Admin Action Ribbon */}
      <div className="flex gap-2">
        <button
          onClick={handleDownloadPDF}
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-semibold bg-violet-500 text-white hover:bg-violet-400 transition-colors"
        >
          <ArrowDownToLine size={12} /> Download PDF Report
        </button>
      </div>

      {/* Metrics Row */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <div className="rounded-xl bg-white/[0.02] border border-white/5 p-4 text-center">
          <DollarSign size={16} className="text-violet-400 mx-auto mb-1.5" />
          <p className="text-lg font-bold text-white">${plan.amount.toLocaleString()}</p>
          <p className="text-[10px] text-slate-500 mt-0.5">Deposit Amount</p>
        </div>
        <div className="rounded-xl bg-white/[0.02] border border-white/5 p-4 text-center">
          <TrendingUp size={16} className="text-emerald-400 mx-auto mb-1.5" />
          <p className="text-lg font-bold text-emerald-400">{(plan.yieldRateBps / 100).toFixed(2)}%</p>
          <p className="text-[10px] text-slate-500 mt-0.5">Yield APY</p>
        </div>
        <div className="rounded-xl bg-white/[0.02] border border-white/5 p-4 text-center">
          <Users size={16} className="text-violet-400 mx-auto mb-1.5" />
          <p className="text-lg font-bold text-white">{plan.beneficiaries.length}</p>
          <p className="text-[10px] text-slate-500 mt-0.5">Beneficiaries</p>
        </div>
        <div className="rounded-xl bg-white/[0.02] border border-white/5 p-4 text-center">
          <Clock size={16} className="text-orange-400 mx-auto mb-1.5" />
          <p className="text-lg font-bold text-white">{Math.round(plan.gracePeriodSeconds / 86400)}d</p>
          <p className="text-[10px] text-slate-500 mt-0.5">Grace Period</p>
        </div>
      </div>

      {/* Plan metadata summary */}
      <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5 space-y-3">
        <h2 className="text-sm font-semibold text-white">General Parameters</h2>
        <div className="grid grid-cols-2 gap-4 text-xs">
          <div>
            <p className="text-slate-500">Plan Status</p>
            <span className="inline-block mt-1 px-2.5 py-0.5 rounded-md bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-semibold">
              {plan.status}
            </span>
          </div>
          <div>
            <p className="text-slate-500">Token Contract</p>
            <p className="text-slate-200 mt-1 font-mono">{plan.token}</p>
          </div>
          <div>
            <p className="text-slate-500">Owner Wallet</p>
            <p className="text-slate-200 mt-1 font-mono text-[11px] truncate">{plan.owner}</p>
          </div>
          <div>
            <p className="text-slate-500">Last Proof of Life (Ping)</p>
            <p className="text-slate-200 mt-1">{new Date(plan.lastPing).toLocaleString()}</p>
          </div>
        </div>
      </div>

      {/* Heirs list */}
      <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
        <h2 className="text-sm font-semibold text-white mb-4">Configured Heirs</h2>
        <div className="space-y-2">
          {plan.beneficiaries.map((b: any, i: number) => (
            <div key={i} className="flex items-center justify-between p-3 rounded-xl bg-white/[0.02] border border-white/5 text-xs">
              <span className="font-mono text-slate-400">{b.address}</span>
              <div className="flex gap-4 items-center">
                <span className="text-slate-500">Anchor: {b.fiatAnchor}</span>
                <span className="font-bold text-violet-400">{(b.allocationBps / 100).toFixed(0)}%</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
