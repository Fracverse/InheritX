"use client";

import { useState, useEffect } from "react";
import { useWallet } from "@/context/WalletContext";
import { plansAPI, type Plan } from "@/app/lib/api/plans";
import inheritanceAPI from "@/app/lib/api/inheritance";
import { formatAddress } from "@/util/address";
import {
  Zap,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Clock,
  Shield,
  Activity,
  Wallet,
  ChevronRight,
  RefreshCw,
  Info,
} from "lucide-react";

type PingStatus = "idle" | "signing" | "submitting" | "success" | "error";

function PingHistoryRow({ ping }: { ping: { pinged_at: string | number; accrued_yield_snapshot?: number } }) {
  const date = typeof ping.pinged_at === "number"
    ? new Date(ping.pinged_at * 1000)
    : new Date(ping.pinged_at);
  return (
    <div className="flex items-center gap-3 py-2.5 border-b border-white/5 last:border-0">
      <div className="w-6 h-6 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center flex-shrink-0">
        <Zap size={10} className="text-emerald-400" />
      </div>
      <div className="flex-1">
        <p className="text-xs font-medium text-slate-300">Ping sent</p>
        <p className="text-[10px] text-slate-500">
          {date.toLocaleDateString("en-GB", { day: "numeric", month: "short", year: "numeric" })} at{" "}
          {date.toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" })}
        </p>
      </div>
      {ping.accrued_yield_snapshot != null && (
        <span className="text-xs text-emerald-400 font-medium">+${Number(ping.accrued_yield_snapshot).toFixed(2)}</span>
      )}
    </div>
  );
}

export default function PingPage() {
  const { isConnected, address, kit, openModal } = useWallet();
  const [plans, setPlans] = useState<Plan[]>([]);
  const [selectedPlan, setSelectedPlan] = useState<Plan | null>(null);
  const [loading, setLoading] = useState(true);
  const [pingStatus, setPingStatus] = useState<PingStatus>("idle");
  const [pingError, setPingError] = useState("");
  const [pingHistory, setPingHistory] = useState<{ pinged_at: string | number; accrued_yield_snapshot?: number }[]>([]);

  useEffect(() => {
    import("@/lib/mockStore").then((mod) => {
      const mockStore = mod.mockStore;
      const mockPlans = mockStore.getPlans().filter((p: any) => p.status?.toUpperCase() === "ACTIVE");
      setPlans(mockPlans);
      if (mockPlans.length > 0) setSelectedPlan(mockPlans[0]);
    });
    fetchPlans();
  }, [isConnected, address]);
  const fetchPlans = async () => {
    const { dataSource } = await import("@/app/lib/api/dataSource");
    const plans = await dataSource.getPlans(address ?? undefined);
    setPlans(plans);
    setLoading(false);
  };

  const handlePing = async () => {
    if (!selectedPlan) return;
    setPingStatus("signing");
    setPingError("");
    try {
      setPingStatus("submitting");
      import("@/lib/mockStore").then((mod) => { const mockStore = mod.mockStore; })
      mockStore.pingPlan(selectedPlan.id);
      
      setPingStatus("success");
      setPingHistory((prev) => [{ pinged_at: Math.floor(Date.now() / 1000), accrued_yield_snapshot: selectedPlan.accrued_yield }, ...prev]);
      setTimeout(() => setPingStatus("idle"), 5000);
    } catch (e) {
      setPingError(e instanceof Error ? e.message : "Ping failed. Please try again.");
      setPingStatus("error");
    }
  };

  const graceDays = selectedPlan?.grace_period_seconds ? Math.round(selectedPlan.grace_period_seconds / 86400) : 0;
  const lastPingTime = selectedPlan?.last_ping
    ? new Date((typeof selectedPlan.last_ping === "number" && selectedPlan.last_ping > 1e10
        ? selectedPlan.last_ping
        : selectedPlan.last_ping * 1000))
    : null;

  return (
    <div className="animate-fade-in space-y-5 max-w-2xl">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-semibold text-white">Proof of Life</h1>
        <p className="text-sm text-slate-500 mt-1">
          Send a cryptographic ping to prove you&apos;re still active and reset the inactivity timer.
        </p>
      </div>

      {/* Info Banner */}
      <div className="rounded-xl border border-[#33C5E0]/20 bg-[#33C5E0]/5 p-4 flex gap-3">
        <Info size={15} className="text-[#33C5E0] mt-0.5 flex-shrink-0" />
        <p className="text-xs text-slate-300 leading-relaxed">
          Each ping resets the inactivity timer on your plan. If the timer expires without a ping, the{" "}
          <span className="text-[#33C5E0] font-medium">grace period</span> begins and beneficiaries may
          initiate a claim. Pings require a wallet signature — no tokens are transferred.
        </p>
      </div>

      {/* Demo Mode Notice */}
      <div className="rounded-xl border border-yellow-500/20 bg-yellow-500/5 p-4 flex gap-3">
        <span className="w-2 h-2 rounded-full bg-yellow-400 animate-pulse mt-1" />
        <p className="text-xs text-yellow-300">
          <strong>Demo Mode Active:</strong> You can select any plan below and send a liveness ping directly without any wallet connection.
        </p>
      </div>

      <>
          {/* Plan Selector */}
          {loading ? (
            <div className="h-32 rounded-2xl bg-white/5 animate-pulse" />
          ) : plans.length === 0 ? (
            <div className="rounded-2xl border border-white/10 bg-white/[0.03] p-8 text-center">
              <Activity size={24} className="text-slate-500 mx-auto mb-2" />
              <p className="text-sm text-slate-400">No active plans found.</p>
            </div>
          ) : (
            <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
              <label className="text-xs text-slate-500 uppercase tracking-wider font-medium block mb-2">
                Select Plan to Ping
              </label>
              <div className="space-y-2">
                {plans.map((p) => (
                  <button
                    key={p.id}
                    onClick={() => setSelectedPlan(p)}
                    className={`w-full flex items-center gap-3 p-3 rounded-xl border transition-all text-left ${
                      selectedPlan?.id === p.id
                        ? "border-[#33C5E0]/40 bg-[#33C5E0]/5"
                        : "border-white/5 bg-white/[0.02] hover:bg-white/5"
                    }`}
                  >
                    <div className={`w-3 h-3 rounded-full border-2 flex-shrink-0 ${
                      selectedPlan?.id === p.id ? "bg-[#33C5E0] border-[#33C5E0]" : "border-slate-600"
                    }`} />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-white">{p.token_address ?? "USDC"} Plan</p>
                      <p className="text-[10px] text-slate-500 font-mono truncate">{p.id}</p>
                    </div>
                    <span className="text-xs text-emerald-400 flex-shrink-0">Active</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Plan Status Card */}
          {selectedPlan && (
            <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
              <h2 className="text-sm font-semibold text-white mb-4">Timer Status</h2>
              <div className="grid grid-cols-2 gap-3 mb-4">
                <div className="p-3 rounded-xl bg-white/[0.03] border border-white/5 text-center">
                  <Clock size={16} className="text-orange-400 mx-auto mb-1" />
                  <p className="text-sm font-semibold text-white">{graceDays}d</p>
                  <p className="text-[10px] text-slate-500">Grace period</p>
                </div>
                <div className="p-3 rounded-xl bg-white/[0.03] border border-white/5 text-center">
                  <Activity size={16} className="text-[#33C5E0] mx-auto mb-1" />
                  <p className="text-sm font-semibold text-white">
                    {lastPingTime
                      ? lastPingTime.toLocaleDateString("en-GB", { day: "numeric", month: "short" })
                      : "Never"}
                  </p>
                  <p className="text-[10px] text-slate-500">Last ping</p>
                </div>
              </div>

              {/* Ping Button */}
              <button
                onClick={handlePing}
                disabled={pingStatus === "signing" || pingStatus === "submitting" || pingStatus === "success"}
                className={`w-full flex items-center justify-center gap-2 py-3.5 rounded-xl font-semibold text-sm transition-all ${
                  pingStatus === "success"
                    ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 cursor-not-allowed"
                    : pingStatus === "error"
                    ? "bg-red-500/10 text-red-400 border border-red-500/20"
                    : "bg-[#33C5E0] text-black hover:bg-[#33C5E0]/90 active:scale-[0.98]"
                } disabled:opacity-60 disabled:cursor-not-allowed`}
              >
                {pingStatus === "signing" && <><Loader2 size={15} className="animate-spin" /> Awaiting Signature…</>}
                {pingStatus === "submitting" && <><Loader2 size={15} className="animate-spin" /> Submitting Ping…</>}
                {pingStatus === "success" && <><CheckCircle2 size={15} /> Ping Sent Successfully!</>}
                {pingStatus === "error" && <><RefreshCw size={15} /> Retry Ping</>}
                {pingStatus === "idle" && <><Zap size={15} /> Send Proof of Life Ping</>}
              </button>

              {pingError && (
                <div className="mt-3 flex items-start gap-2 p-3 rounded-xl bg-red-500/10 border border-red-500/20">
                  <AlertCircle size={13} className="text-red-400 mt-0.5 flex-shrink-0" />
                  <p className="text-xs text-red-300">{pingError}</p>
                </div>
              )}
            </div>
          )}

          {/* Ping History */}
          {pingHistory.length > 0 && (
            <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
              <h2 className="text-sm font-semibold text-white mb-3">
                Recent Pings <span className="text-slate-500 font-normal">(this session)</span>
              </h2>
              <div>
                {pingHistory.map((p, i) => <PingHistoryRow key={i} ping={p} />)}
              </div>
            </div>
          )}
        </>
    </div>
  );
}
