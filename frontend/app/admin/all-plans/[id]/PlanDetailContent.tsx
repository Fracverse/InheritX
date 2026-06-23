"use client";

import React, { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { 
  ArrowLeft, 
  AlertTriangle, 
  CheckCircle, 
  Clock, 
  HelpCircle, 
  Shield, 
  TrendingUp, 
  Coins, 
  Users, 
  Layers,
  ArrowRight,
  Info,
  DollarSign
} from "lucide-react";
import { plansAPI } from "@/app/lib/api/plans";
import { useWallet } from "@/context/WalletContext";

interface PlanDetailContentProps {
  id: string;
}

interface TriggerInfo {
  timestamp: string | null;
  freeze_status: "PENDING" | "PROCESSING" | "FROZEN";
  recall_progress: number;
  settlement_status: "PENDING" | "PROCESSING" | "LIQUIDATED" | "SETTLED";
  outstanding_loans: Array<{ pool: string; amount: string; status: string }>;
}

export default function PlanDetailContent({ id }: PlanDetailContentProps) {
  const router = useRouter();
  const { isConnected, connect, address } = useWallet();
  const [plan, setPlan] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Trigger states
  const [triggerInfo, setTriggerInfo] = useState<TriggerInfo | null>(null);
  const [isConfirmModalOpen, setIsConfirmModalOpen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [recallSimulationActive, setRecallSimulationActive] = useState(false);
  const [simulationLogs, setSimulationLogs] = useState<string[]>([]);

  // Load plan detail and trigger info
  useEffect(() => {
    async function loadData() {
      try {
        setIsLoading(true);
        const data = await plansAPI.getPlan(id);
        setPlan(data);
        
        const info = await plansAPI.getTriggerInfo(id);
        if (info && info.data) {
          setTriggerInfo(info.data);
        }
      } catch (err: any) {
        console.error("Failed to load plan:", err);
        // Fallback to static mock plan if API errors or plan not found
        setPlan({
          id,
          name: id === "1" ? "Testnet testing" : "Family Trust Plan",
          description: "Just testing out inheritx dapp for secure asset distribution",
          owner: "GABC123OwnerAddress",
          amount: "15,000 XLM & 4,500 USDC",
          assetsList: [
            { asset: "USDC", amount: "4,500" },
            { asset: "XLM", amount: "15,000" }
          ],
          beneficiaries: [
            { name: "John Doe", email: "john@doe.com", relationship: "Son", share: 60 },
            { name: "Jane Doe", email: "jane@doe.com", relationship: "Daughter", share: 40 }
          ],
          status: "ACTIVE",
          transferDate: "2026-12-01"
        });
      } finally {
        setIsLoading(false);
      }
    }
    loadData();
  }, [id]);

  // Polling helper or mock updates
  const refreshTriggerInfo = async () => {
    try {
      const info = await plansAPI.getTriggerInfo(id);
      if (info && info.data) {
        setTriggerInfo(info.data);
      }
    } catch (err) {
      console.error(err);
    }
  };

  const handleTriggerInheritance = async () => {
    setIsSubmitting(true);
    addLog("Initiating inheritance trigger transaction on Stellar network...");
    try {
      await plansAPI.triggerPlan(id);
      // Update local plan status to triggered
      setPlan((prev: any) => ({ ...prev, status: "triggered" }));
      await refreshTriggerInfo();
      setIsConfirmModalOpen(false);
      addLog("SUCCESS: Inheritance execution triggered on Soroban contract.");
    } catch (err: any) {
      setError(err.message || "Failed to trigger inheritance");
      addLog("ERROR: Transaction signature rejected or failed.");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleFreezeLoans = async () => {
    setIsSubmitting(true);
    addLog("Sending Soroban request to freeze active lending pool borrows for this plan...");
    try {
      await plansAPI.freezeLoans(id);
      await refreshTriggerInfo();
      addLog("SUCCESS: Soroban borrowing capability disabled. All active positions frozen.");
    } catch (err: any) {
      setError(err.message || "Failed to freeze loans");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleRecallLoans = async () => {
    setIsSubmitting(true);
    setRecallSimulationActive(true);
    addLog("Sending auto-recall queries to Soroban lending pool...");
    try {
      // Simulate gradual recall progress
      let currentProgress = 0;
      const interval = setInterval(async () => {
        currentProgress += 25;
        addLog(`Recalling loans: ${currentProgress}% reclaimed...`);
        
        if (currentProgress >= 100) {
          clearInterval(interval);
          await plansAPI.recallLoans(id);
          await refreshTriggerInfo();
          addLog("SUCCESS: Loans recalled successfully back to collateral settlement reserve.");
          setRecallSimulationActive(false);
          setIsSubmitting(false);
        }
      }, 800);
    } catch (err: any) {
      setError(err.message || "Failed to recall loans");
      setRecallSimulationActive(false);
      setIsSubmitting(false);
    }
  };

  const handleLiquidateSettle = async () => {
    setIsSubmitting(true);
    addLog("Executing liquidation fallback on remaining outstanding debt...");
    try {
      await plansAPI.liquidateAndSettle(id);
      setPlan((prev: any) => ({ ...prev, status: "claimable" }));
      await refreshTriggerInfo();
      addLog("SUCCESS: Collateral seized, outstanding debt cleared. Plan is now fully claimable by beneficiaries.");
    } catch (err: any) {
      setError(err.message || "Failed to liquidate and settle");
    } finally {
      setIsSubmitting(false);
    }
  };

  const addLog = (message: string) => {
    setSimulationLogs(prev => [`[${new Date().toLocaleTimeString()}] ${message}`, ...prev]);
  };

  if (isLoading) {
    return (
      <div className="min-h-[70vh] flex flex-col items-center justify-center">
        <div className="w-12 h-12 border-4 border-gray-800 border-t-[#33C5E0] rounded-full animate-spin mb-4"></div>
        <p className="text-gray-400">Loading plan detail metrics...</p>
      </div>
    );
  }

  if (!plan) {
    return (
      <div className="bg-red-500/10 border border-red-500/20 text-red-400 p-6 rounded-2xl">
        <h3 className="font-semibold text-lg">Plan not found</h3>
        <p className="mt-1">The plan with ID {id} could not be resolved.</p>
      </div>
    );
  }

  // Calculate status flags
  const statusUpper = plan.status ? plan.status.toUpperCase() : "ACTIVE";
  const isTriggered = statusUpper === "TRIGGERED" || statusUpper === "CLAIMABLE" || statusUpper === "EXECUTED";
  const isClaimable = statusUpper === "CLAIMABLE" || statusUpper === "EXECUTED";

  return (
    <div className="space-y-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-6 border-b border-gray-900">
        <div className="flex items-center gap-4">
          <Link
            href="/admin/all-plans"
            className="w-10 h-10 rounded-full bg-[#182024] border border-gray-800 flex items-center justify-center text-[#92A5A8] hover:text-white hover:bg-gray-800 transition-all duration-200"
          >
            <ArrowLeft size={20} />
          </Link>
          <div>
            <div className="flex items-center gap-3 flex-wrap">
              <h1 className="text-2xl font-bold text-[#FCFFFF]">{plan.name}</h1>
              <span className={`px-3 py-1 rounded-full text-xs font-semibold border ${
                isClaimable 
                  ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20" 
                  : isTriggered 
                    ? "bg-amber-500/10 text-amber-400 border-amber-500/20 animate-pulse" 
                    : "bg-cyan-500/10 text-cyan-400 border-cyan-500/20"
              }`}>
                {statusUpper}
              </span>
            </div>
            <p className="text-sm text-[#92A5A8] mt-1">{plan.description}</p>
          </div>
        </div>

        {/* Wallet Connection Status */}
        <div className="flex items-center gap-3 self-start sm:self-center">
          {isConnected ? (
            <div className="flex items-center gap-2 px-4 py-2 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-lg text-sm font-medium">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-ping"></span>
              <span className="font-mono text-xs">Admin Wallet: {address?.slice(0, 6)}...{address?.slice(-6)}</span>
            </div>
          ) : (
            <button
              onClick={() => connect("mock_wallet")}
              className="px-5 py-2.5 bg-[#33C5E0] hover:bg-[#2AB8D3] text-black font-semibold rounded-lg text-sm transition-all shadow-md shadow-[#33C5E0]/20"
            >
              Connect Admin Wallet
            </button>
          )}
        </div>
      </div>

      {/* Main Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        
        {/* Left Column: Plan Details & Allocation Preview */}
        <div className="lg:col-span-1 space-y-6">
          {/* Plan Overview Card */}
          <div className="bg-[#182024]/50 border border-gray-900 rounded-2xl p-6 backdrop-blur-sm">
            <h3 className="text-md font-semibold text-[#FCFFFF] mb-4 flex items-center gap-2">
              <Coins className="w-5 h-5 text-[#33C5E0]" />
              Plan Valuation & Assets
            </h3>
            <div className="space-y-4">
              <div className="bg-gray-950/40 p-4 rounded-xl border border-gray-900">
                <p className="text-xs text-[#92A5A8]">Total Plan Value</p>
                <p className="text-2xl font-bold text-white mt-1">{plan.amount}</p>
              </div>

              <div>
                <label className="text-xs text-[#92A5A8] block mb-2">Locked Soroban Reserves</label>
                <div className="space-y-2">
                  {(plan.assetsList || [
                    { asset: "USDC", amount: "4,500" },
                    { asset: "XLM", amount: "15,000" }
                  ]).map((ast: any, i: number) => (
                    <div key={i} className="flex justify-between items-center text-sm py-1.5 border-b border-gray-900/50">
                      <span className="text-[#FCFFFF] font-medium">{ast.asset}</span>
                      <span className="font-mono text-[#92A5A8]">{ast.amount} {ast.asset}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* Beneficiary Distribution Preview Card */}
          <div className="bg-[#182024]/50 border border-gray-900 rounded-2xl p-6 backdrop-blur-sm">
            <h3 className="text-md font-semibold text-[#FCFFFF] mb-4 flex items-center gap-2">
              <Users className="w-5 h-5 text-[#33C5E0]" />
              Beneficiary Share Preview
            </h3>
            <div className="space-y-3">
              {(plan.beneficiaries || [
                { name: "John Doe", email: "john@doe.com", relationship: "Son", share: 60 },
                { name: "Jane Doe", email: "jane@doe.com", relationship: "Daughter", share: 40 }
              ]).map((ben: any, i: number) => (
                <div key={i} className="p-3 bg-gray-950/30 rounded-xl border border-gray-900">
                  <div className="flex justify-between items-start">
                    <div>
                      <p className="text-sm font-semibold text-white">{ben.name}</p>
                      <p className="text-xs text-[#92A5A8]">{ben.relationship} &bull; {ben.email}</p>
                    </div>
                    <span className="text-xs bg-[#33C5E0]/10 text-[#33C5E0] px-2 py-0.5 rounded font-mono font-semibold">
                      {ben.share}%
                    </span>
                  </div>
                  <div className="mt-2.5 pt-2.5 border-t border-gray-900 flex justify-between items-center text-xs">
                    <span className="text-[#92A5A8]">USDC Share</span>
                    <span className="font-mono text-[#FCFFFF] font-medium">
                      {(4500 * ben.share / 100).toLocaleString()} USDC
                    </span>
                  </div>
                  <div className="mt-1 flex justify-between items-center text-xs">
                    <span className="text-[#92A5A8]">XLM Share</span>
                    <span className="font-mono text-[#FCFFFF] font-medium">
                      {(15000 * ben.share / 100).toLocaleString()} XLM
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Right Column: Trigger Controls & Live Timeline */}
        <div className="lg:col-span-2 space-y-6">
          
          {/* Main Dashboard Widget */}
          {!isTriggered ? (
            /* PRE-TRIGGER INTERFACE */
            <div className="bg-linear-to-b from-[#182024]/80 to-gray-950/80 border border-gray-800 rounded-3xl p-8 shadow-xl shadow-cyan-950/10">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 bg-amber-500/10 rounded-2xl flex items-center justify-center text-amber-500 shrink-0">
                  <AlertTriangle className="w-6 h-6 animate-pulse" />
                </div>
                <div>
                  <h2 className="text-lg font-bold text-white">Trigger Inheritance Execution</h2>
                  <p className="text-[#92A5A8] text-sm mt-1">
                    This administrative bypass option will manually authorize the Soroban Smart Contract to initiate the liquidation, pool recalls, and distribution protocol for this plan.
                  </p>
                </div>
              </div>

              {/* Outstanding Loan Warning */}
              <div className="my-6 bg-red-950/20 border border-red-900/30 rounded-2xl p-5">
                <div className="flex gap-3">
                  <Shield className="w-5 h-5 text-red-400 shrink-0 mt-0.5" />
                  <div>
                    <h4 className="text-red-400 text-sm font-semibold">Active Lending Pool Exposure Detected</h4>
                    <p className="text-gray-400 text-xs mt-1">
                      This plan has outstanding loans locked as collateral in the Soroban lending pools. Triggering inheritance will execute automated loan freeze and recovery strategies.
                    </p>
                  </div>
                </div>
              </div>

              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setIsConfirmModalOpen(true)}
                  disabled={!isConnected}
                  className={`px-6 py-3 font-semibold text-sm rounded-xl transition-all flex items-center gap-2 ${
                    isConnected
                      ? "bg-[#33C5E0] hover:bg-[#2AB8D3] text-black shadow-lg shadow-[#33C5E0]/10 active:scale-95"
                      : "bg-gray-800 text-gray-500 cursor-not-allowed border border-gray-900"
                  }`}
                >
                  Trigger Inheritance
                  <ArrowRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          ) : (
            /* POST-TRIGGER DASHBOARD */
            <div className="space-y-6">
              
              {/* Stepper Timeline & Progress Widget */}
              <div className="bg-[#182024]/50 border border-gray-900 rounded-2xl p-6 backdrop-blur-sm">
                <h3 className="text-md font-semibold text-[#FCFFFF] mb-6 flex items-center gap-2">
                  <Layers className="w-5 h-5 text-[#33C5E0]" />
                  Inheritance Execution Progress Timeline
                </h3>

                {/* Timeline Stepper Component */}
                <div className="relative pl-6 space-y-8 before:absolute before:left-[11px] before:top-2 before:bottom-2 before:w-0.5 before:bg-gray-800">
                  
                  {/* Step 1: Triggered */}
                  <div className="relative">
                    <span className="absolute -left-[21px] top-0.5 w-6 h-6 rounded-full bg-emerald-500 border border-gray-950 flex items-center justify-center text-xs text-black">
                      <CheckCircle className="w-4 h-4 text-black" />
                    </span>
                    <div className="pl-4">
                      <h4 className="text-sm font-semibold text-white">Inheritance Execution Triggered</h4>
                      <p className="text-xs text-[#92A5A8] mt-0.5">
                        Authorized by Admin/Guardian at {triggerInfo?.timestamp ? new Date(triggerInfo.timestamp).toLocaleString() : "Recently"}
                      </p>
                    </div>
                  </div>

                  {/* Step 2: Loan Freeze */}
                  <div className="relative">
                    <span className={`absolute -left-[21px] top-0.5 w-6 h-6 rounded-full border border-gray-950 flex items-center justify-center text-xs ${
                      triggerInfo?.freeze_status === "FROZEN" 
                        ? "bg-emerald-500 text-black" 
                        : "bg-[#1C252A] text-[#92A5A8]"
                    }`}>
                      {triggerInfo?.freeze_status === "FROZEN" ? (
                        <CheckCircle className="w-4 h-4 text-black" />
                      ) : (
                        <Clock className="w-4 h-4" />
                      )}
                    </span>
                    <div className="pl-4">
                      <div className="flex items-center justify-between">
                        <h4 className="text-sm font-semibold text-white">Freezing Active Lending Positions</h4>
                        {triggerInfo?.freeze_status === "PENDING" && (
                          <button
                            onClick={handleFreezeLoans}
                            disabled={isSubmitting}
                            className="text-xs text-[#33C5E0] hover:underline"
                          >
                            Freeze Loans
                          </button>
                        )}
                      </div>
                      <p className="text-xs text-[#92A5A8] mt-0.5">
                        Status: <span className="font-semibold text-white">{triggerInfo?.freeze_status}</span>
                      </p>
                    </div>
                  </div>

                  {/* Step 3: Recall Loans */}
                  <div className="relative">
                    <span className={`absolute -left-[21px] top-0.5 w-6 h-6 rounded-full border border-gray-950 flex items-center justify-center text-xs ${
                      triggerInfo?.recall_progress === 100 
                        ? "bg-emerald-500 text-black" 
                        : triggerInfo?.freeze_status === "FROZEN" 
                          ? "bg-amber-500/20 text-amber-400 animate-pulse border border-amber-500/30"
                          : "bg-[#1C252A] text-[#92A5A8]"
                    }`}>
                      {triggerInfo?.recall_progress === 100 ? (
                        <CheckCircle className="w-4 h-4 text-black" />
                      ) : (
                        <Clock className="w-4 h-4" />
                      )}
                    </span>
                    <div className="pl-4">
                      <div className="flex items-center justify-between">
                        <h4 className="text-sm font-semibold text-white">Recalling Capital from Pools</h4>
                        {triggerInfo?.freeze_status === "FROZEN" && triggerInfo?.recall_progress < 100 && (
                          <button
                            onClick={handleRecallLoans}
                            disabled={isSubmitting}
                            className="text-xs text-[#33C5E0] hover:underline"
                          >
                            Recall Capital
                          </button>
                        )}
                      </div>
                      
                      {/* Recall Progress Bar */}
                      <div className="mt-2 space-y-1 max-w-md">
                        <div className="w-full h-1.5 bg-gray-950 rounded-full overflow-hidden border border-gray-900">
                          <div 
                            className="h-full bg-gradient-to-r from-cyan-500 to-indigo-500 transition-all duration-300"
                            style={{ width: `${triggerInfo?.recall_progress || 0}%` }}
                          ></div>
                        </div>
                        <div className="flex justify-between text-[10px] text-[#92A5A8]">
                          <span>Progress</span>
                          <span>{triggerInfo?.recall_progress || 0}% Complete</span>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Step 4: Settle & Liquidate Fallback */}
                  <div className="relative">
                    <span className={`absolute -left-[21px] top-0.5 w-6 h-6 rounded-full border border-gray-950 flex items-center justify-center text-xs ${
                      triggerInfo?.settlement_status === "SETTLED" 
                        ? "bg-emerald-500 text-black" 
                        : triggerInfo?.recall_progress === 100 
                          ? "bg-amber-500/20 text-amber-400 animate-pulse border border-amber-500/30"
                          : "bg-[#1C252A] text-[#92A5A8]"
                    }`}>
                      {triggerInfo?.settlement_status === "SETTLED" ? (
                        <CheckCircle className="w-4 h-4 text-black" />
                      ) : (
                        <Clock className="w-4 h-4" />
                      )}
                    </span>
                    <div className="pl-4">
                      <div className="flex items-center justify-between">
                        <h4 className="text-sm font-semibold text-white">Collateral Settlement & Liquidation</h4>
                        {triggerInfo?.recall_progress === 100 && triggerInfo?.settlement_status !== "SETTLED" && (
                          <button
                            onClick={handleLiquidateSettle}
                            disabled={isSubmitting}
                            className="text-xs text-[#33C5E0] hover:underline"
                          >
                            Execute Settle
                          </button>
                        )}
                      </div>
                      <p className="text-xs text-[#92A5A8] mt-0.5">
                        Status: <span className="font-semibold text-white">{triggerInfo?.settlement_status}</span>
                      </p>
                    </div>
                  </div>

                  {/* Step 5: Claimable */}
                  <div className="relative">
                    <span className={`absolute -left-[21px] top-0.5 w-6 h-6 rounded-full border border-gray-950 flex items-center justify-center text-xs ${
                      isClaimable 
                        ? "bg-emerald-500 text-black" 
                        : "bg-[#1C252A] text-[#92A5A8]"
                    }`}>
                      {isClaimable ? (
                        <CheckCircle className="w-4 h-4 text-black" />
                      ) : (
                        <Clock className="w-4 h-4" />
                      )}
                    </span>
                    <div className="pl-4">
                      <h4 className="text-sm font-semibold text-white">Available for Beneficiary Claims</h4>
                      <p className="text-xs text-[#92A5A8] mt-0.5">
                        Time restrictions bypassed. Beneficiaries can execute immediate cryptographic claim payouts.
                      </p>
                    </div>
                  </div>

                </div>
              </div>

              {/* Recall Progress Tracker Panel */}
              {triggerInfo?.freeze_status === "FROZEN" && triggerInfo?.recall_progress < 100 && (
                <div className="bg-[#182024]/50 border border-gray-900 rounded-2xl p-6 backdrop-blur-sm">
                  <h3 className="text-sm font-semibold text-white mb-4 flex items-center gap-2">
                    <Coins className="w-4 h-4 text-[#33C5E0]" />
                    Outstanding Soroban Loan Recall Strategy
                  </h3>
                  <div className="space-y-3">
                    {triggerInfo?.outstanding_loans.map((loan, idx) => (
                      <div key={idx} className="flex justify-between items-center p-3 bg-gray-950/40 rounded-xl border border-gray-900">
                        <div>
                          <p className="text-xs font-semibold text-white">{loan.pool}</p>
                          <p className="text-[10px] text-[#92A5A8]">LTV Exposure: 62%</p>
                        </div>
                        <div className="text-right">
                          <p className="text-xs font-mono text-white font-medium">{loan.amount}</p>
                          <span className={`text-[10px] px-1.5 py-0.5 rounded font-medium ${
                            loan.status === "Frozen" 
                              ? "bg-blue-500/10 text-blue-400"
                              : loan.status === "Recalled"
                                ? "bg-emerald-500/10 text-emerald-400"
                                : "bg-amber-500/10 text-amber-400"
                          }`}>
                            {loan.status}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Liquidation Fallback Panel */}
              {triggerInfo?.recall_progress === 100 && triggerInfo?.settlement_status !== "SETTLED" && (
                <div className="bg-red-950/10 border border-red-900/20 rounded-2xl p-6">
                  <div className="flex gap-4">
                    <AlertTriangle className="w-8 h-8 text-red-500 shrink-0" />
                    <div className="flex-1 space-y-4">
                      <div>
                        <h4 className="text-md font-bold text-red-400">Recall Threshold Breached: Auto-Liquidation Fallback Needed</h4>
                        <p className="text-gray-400 text-sm mt-1">
                          Some lending pool capital could not be directly recalled due to locked borrowing positions. Admin approval required to seize outstanding collateral reserves and trigger market settlement.
                        </p>
                      </div>

                      <div className="grid grid-cols-2 md:grid-cols-3 gap-4 bg-gray-950/40 p-4 rounded-xl border border-gray-900">
                        <div>
                          <span className="text-[10px] text-[#92A5A8] uppercase block">Outstanding Debt</span>
                          <span className="text-sm font-mono text-white font-semibold">1,200 USDC</span>
                        </div>
                        <div>
                          <span className="text-[10px] text-[#92A5A8] uppercase block">Collateral locked</span>
                          <span className="text-sm font-mono text-white font-semibold">3,500 XLM</span>
                        </div>
                        <div>
                          <span className="text-[10px] text-[#92A5A8] uppercase block">Liquidation Price (XLM)</span>
                          <span className="text-sm font-mono text-emerald-400 font-semibold">$0.142</span>
                        </div>
                      </div>

                      <div className="flex justify-end pt-2">
                        <button
                          onClick={handleLiquidateSettle}
                          disabled={isSubmitting}
                          className="px-5 py-2.5 bg-red-600 hover:bg-red-500 text-white font-semibold rounded-lg text-sm transition-all shadow-md shadow-red-950/20 active:scale-95"
                        >
                          Execute Auto-Liquidation Fallback
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Execution Action Logs */}
              <div className="bg-[#182024]/50 border border-gray-900 rounded-2xl p-6 backdrop-blur-sm">
                <h3 className="text-xs font-semibold text-white uppercase tracking-wider mb-3">Soroban execution logs</h3>
                <div className="bg-gray-950 p-4 rounded-xl border border-gray-900 h-40 overflow-y-auto font-mono text-[11px] text-emerald-400/90 space-y-1 scrollbar-thin">
                  {simulationLogs.length > 0 ? (
                    simulationLogs.map((log, i) => <div key={i}>{log}</div>)
                  ) : (
                    <div className="text-gray-600">Waiting for trigger status events...</div>
                  )}
                </div>
              </div>

            </div>
          )}

        </div>
      </div>

      {/* TRIGGER CONFIRMATION MODAL */}
      {isConfirmModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-fade-in">
          <div className="bg-[#161E22] border border-gray-800 rounded-3xl max-w-xl w-full overflow-hidden shadow-2xl animate-scale-in">
            <div className="p-6 md:p-8 space-y-6">
              
              {/* Modal Header */}
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="text-xl font-bold text-white">Confirm Inheritance Execution</h3>
                  <p className="text-xs text-[#92A5A8] mt-1">Review plan settlement strategy before authorization.</p>
                </div>
                <button 
                  onClick={() => setIsConfirmModalOpen(false)}
                  className="text-gray-500 hover:text-white transition-colors"
                >
                  &times;
                </button>
              </div>

              {/* Plan Assets summary */}
              <div className="bg-gray-950/40 p-4 rounded-xl border border-gray-900 space-y-3">
                <h4 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider">Plan Assets Overview</h4>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-400">Total Valuation</span>
                  <span className="font-semibold text-white">{plan.amount}</span>
                </div>
                <div className="flex justify-between text-sm border-t border-gray-900 pt-2">
                  <span className="text-gray-400">Soroban Contract ID</span>
                  <span className="font-mono text-xs text-[#92A5A8]">CABC...SorobanPlan{id}</span>
                </div>
              </div>

              {/* Outstanding Loans strategy preview */}
              <div className="bg-amber-500/5 border border-amber-500/10 p-4 rounded-xl space-y-3">
                <div className="flex gap-2">
                  <AlertTriangle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
                  <div>
                    <h5 className="text-xs font-semibold text-amber-500 uppercase tracking-wider">Soroban Loan Exposure strategy</h5>
                    <p className="text-gray-400 text-xs mt-1">
                      Outstanding loans will be recalled through the following process:
                    </p>
                  </div>
                </div>
                <ol className="list-decimal list-inside text-[11px] text-gray-400 space-y-1.5 pl-1.5">
                  <li>Freeze borrowing accounts to prevent further debt drawdown</li>
                  <li>Perform auto-recall sequence from Soroban lending pools</li>
                  <li>Execute collateral seizure & liquidation in case of pool liquidity shortfalls</li>
                </ol>
              </div>

              {/* Beneficiary Shares breakdown */}
              <div className="space-y-2">
                <h4 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider">Beneficiary Claims Allocation</h4>
                <div className="space-y-2">
                  {(plan.beneficiaries || [
                    { name: "John Doe", email: "john@doe.com", relationship: "Son", share: 60 },
                    { name: "Jane Doe", email: "jane@doe.com", relationship: "Daughter", share: 40 }
                  ]).map((ben: any, i: number) => (
                    <div key={i} className="flex justify-between text-xs py-1.5 border-b border-gray-900">
                      <span className="text-gray-300">{ben.name} ({ben.relationship})</span>
                      <span className="font-mono text-white font-medium">{ben.share}% Share</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Action Buttons */}
              <div className="flex justify-end gap-3 pt-4 border-t border-gray-900">
                <button
                  onClick={() => setIsConfirmModalOpen(false)}
                  className="px-5 py-2.5 bg-[#182024] hover:bg-gray-800 text-[#92A5A8] hover:text-white rounded-lg text-sm font-semibold transition-all border border-gray-800"
                >
                  Cancel
                </button>
                <button
                  onClick={handleTriggerInheritance}
                  disabled={isSubmitting}
                  className="px-6 py-2.5 bg-[#33C5E0] hover:bg-[#2AB8D3] text-black rounded-lg text-sm font-semibold transition-all flex items-center gap-2 active:scale-95 shadow-md shadow-[#33C5E0]/10"
                >
                  {isSubmitting ? "Executing on Soroban..." : "Confirm & Trigger"}
                </button>
              </div>

            </div>
          </div>
        </div>
      )}
    </div>
  );
}
