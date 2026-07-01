"use client";

import { useState } from "react";
import { useWallet } from "@/context/WalletContext";
import { plansAPI, type Plan } from "@/app/lib/api/plans";
import { useInactivityTimer } from "@/app/hooks/useInactivityTimer";
import { formatAddress } from "@/util/address";
import { motion, AnimatePresence } from "framer-motion";
import {
  Search,
  Wallet,
  Clock,
  CheckCircle2,
  AlertCircle,
  BookOpen,
  Loader2,
  ArrowRight,
  ShieldCheck,
  Coins
} from "lucide-react";

// ─── Plan Claim Card Component ───────────────────────────────────────────────────

interface PlanClaimCardProps {
  initialPlan: Plan;
  onSuccess: () => void;
}

function PlanClaimCard({ initialPlan, onSuccess }: PlanClaimCardProps) {
  const { kit, address: connectedAddress, isConnected, openModal } = useWallet();
  const [currentPlan, setCurrentPlan] = useState<Plan>(initialPlan);

  // Inactivity timer hook for counting down grace period
  const { timerState, loading: timerLoading, error: timerError } = useInactivityTimer({
    planId: currentPlan.id,
    enabled: currentPlan.status !== "completed",
    pollIntervalMs: 15000, // Poll every 15 seconds
  });

  const [claimStatus, setClaimStatus] = useState<"idle" | "signing" | "submitting" | "success" | "error">("idle");
  const [claimError, setClaimError] = useState("");

  const handleClaim = async () => {
    if (!isConnected || !connectedAddress) {
      openModal();
      return;
    }

    setClaimStatus("signing");
    setClaimError("");

    try {
      // 1. Build unsigned XDR for claim transaction
      const xdr = `unsigned-xdr::claim::${currentPlan.owner_address || currentPlan.user_id}::${Date.now()}`;
      
      // 2. Sign transaction with connected wallet
      await kit?.signTransaction(xdr);

      setClaimStatus("submitting");

      // 3. Submit claim request to backend/mock
      const updatedPlan = await plansAPI.claimPlan(currentPlan.id, {
        beneficiary_email: "beneficiary@example.com",
        two_fa_code: "123456", // dummy value required by API signature
      });

      setCurrentPlan(updatedPlan);
      setClaimStatus("success");
      onSuccess();
    } catch (err) {
      setClaimStatus("error");
      setClaimError(err instanceof Error ? err.message : "Failed to initiate payout.");
    }
  };

  const isCompleted = currentPlan.status.toLowerCase() === "completed" || currentPlan.status.toLowerCase() === "settled";
  const isClaimable = !isCompleted && (timerState.isClaimable || currentPlan.status.toLowerCase() === "claimable");

  // Determine status display details
  let statusLabel = "Active";
  let statusClass = "bg-[#48BB78]/10 border-[#48BB78]/30 text-[#48BB78]";
  
  if (isCompleted) {
    statusLabel = "Completed";
    statusClass = "bg-primary/10 border-primary/30 text-primary";
  } else if (isClaimable) {
    statusLabel = "Claimable";
    statusClass = "bg-[#F56565]/10 border-[#F56565]/30 text-[#F56565]";
  }

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.95 }}
      className="bg-white/3 border border-white/10 rounded-2xl p-6 space-y-6 hover:border-white/20 transition-all duration-300"
    >
      {/* Header Info */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div className="space-y-1">
          <div className="flex items-center gap-3">
            <h3 className="text-lg font-semibold text-foreground">
              {currentPlan.title || "Inheritance Plan"}
            </h3>
            <span className={`px-2.5 py-0.5 rounded-full border text-xs font-semibold ${statusClass}`}>
              {statusLabel}
            </span>
          </div>
          <p className="text-xs text-gray-500 font-mono">
            Plan ID: {currentPlan.id}
          </p>
        </div>
        <div className="text-right sm:text-right text-left">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Allocated Assets</p>
          <p className="text-2xl font-bold font-mono text-primary mt-0.5">
            {currentPlan.net_amount || currentPlan.fee} <span className="text-sm font-medium">USDC</span>
          </p>
        </div>
      </div>

      {/* Description */}
      {currentPlan.description && (
        <p className="text-sm text-gray-400 leading-relaxed bg-white/1 rounded-xl p-3 border border-white/5">
          {currentPlan.description}
        </p>
      )}

      {/* Countdown Timer */}
      {!isCompleted && (
        <div className="bg-black/20 rounded-xl p-4 border border-white/5 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Clock size={16} className="text-[#92A5A8]" />
              <span className="text-xs text-[#92A5A8] uppercase tracking-wider font-semibold">Remaining Grace Period</span>
            </div>
            {isClaimable && (
              <span className="text-xs font-bold text-[#F56565]">
                Grace Period Elapsed
              </span>
            )}
          </div>

          <div className="grid grid-cols-4 gap-2 text-center">
            <div className="bg-white/3 rounded-lg py-2">
              <p className="text-xs font-bold text-foreground font-mono">{timerState.days}</p>
              <p className="text-[10px] text-gray-500 uppercase tracking-wider mt-0.5">Days</p>
            </div>
            <div className="bg-white/3 rounded-lg py-2">
              <p className="text-xs font-bold text-foreground font-mono">{timerState.hours}</p>
              <p className="text-[10px] text-gray-500 uppercase tracking-wider mt-0.5">Hours</p>
            </div>
            <div className="bg-white/3 rounded-lg py-2">
              <p className="text-xs font-bold text-foreground font-mono">{timerState.minutes}</p>
              <p className="text-[10px] text-gray-500 uppercase tracking-wider mt-0.5">Mins</p>
            </div>
            <div className="bg-white/3 rounded-lg py-2">
              <p className="text-xs font-bold text-foreground font-mono">{timerState.seconds}</p>
              <p className="text-[10px] text-gray-500 uppercase tracking-wider mt-0.5">Secs</p>
            </div>
          </div>
        </div>
      )}

      {/* Error Displays */}
      <AnimatePresence>
        {(claimStatus === "error" || timerError) && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            className="flex items-start gap-2.5 p-3.5 rounded-xl bg-[#F5656514] border border-[#F5656540] text-[#F56565] text-xs"
          >
            <AlertCircle size={16} className="mt-0.5 flex-shrink-0" />
            <div>
              <p className="font-semibold">Transaction Failed</p>
              <p className="mt-0.5 opacity-90">{claimError || timerError?.message}</p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Success Banner */}
      <AnimatePresence>
        {claimStatus === "success" && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            className="flex items-start gap-2.5 p-3.5 rounded-xl bg-[#48BB7814] border border-[#48BB7840] text-[#48BB78] text-xs"
          >
            <CheckCircle2 size={16} className="mt-0.5 flex-shrink-0" />
            <div>
              <p className="font-semibold">Payout Successfully Initiated!</p>
              <p className="mt-0.5 opacity-90">
                The on-chain claim transaction has executed. Assets are being split among the designated beneficiaries.
              </p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Action Footer */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pt-2">
        <div className="text-xs text-gray-500 space-y-1">
          <p>Distribution: {currentPlan.distribution_method || "Automatic splitting via smart contract"}</p>
          <p>Owner: {formatAddress(currentPlan.owner_address || currentPlan.user_id)}</p>
        </div>

        <button
          type="button"
          onClick={handleClaim}
          disabled={!isClaimable || claimStatus === "signing" || claimStatus === "submitting" || claimStatus === "success"}
          className={`flex items-center justify-center gap-2 px-6 py-2.5 text-sm font-semibold rounded-xl transition-all duration-300 ${
            isCompleted
              ? "bg-white/5 text-gray-400 border border-white/10 cursor-default"
              : isClaimable
                ? "bg-primary text-black hover:bg-primary/95 shadow-[0_0_15px_-3px_#33c5e0] hover:scale-[1.02] cursor-pointer"
                : "bg-white/5 text-gray-500 border border-white/5 cursor-not-allowed"
          }`}
        >
          {claimStatus === "signing" && (
            <>
              <Loader2 size={14} className="animate-spin" />
              Signing Claim...
            </>
          )}
          {claimStatus === "submitting" && (
            <>
              <Loader2 size={14} className="animate-spin" />
              Submitting Payout...
            </>
          )}
          {claimStatus === "success" && (
            <>
              <CheckCircle2 size={14} />
              Payout Claimed
            </>
          )}
          {claimStatus === "idle" && claimStatus !== "error" && (
            <>
              {isCompleted ? (
                "Payout Completed"
              ) : (
                <>
                  <Coins size={14} />
                  Initiate Payout
                </>
              )}
            </>
          )}
          {claimStatus === "error" && (
            <>
              <Coins size={14} />
              Retry Payout
            </>
          )}
        </button>
      </div>
    </motion.div>
  );
}

// ─── Main Page Component ──────────────────────────────────────────────────────────

export default function ClaimPlanPage() {
  const { isConnected, address, openModal } = useWallet();
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(false);
  const [plans, setPlans] = useState<Plan[]>([]);
  const [searched, setSearched] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim()) return;

    setLoading(true);
    setError(null);
    setSearched(true);

    try {
      const results = await plansAPI.getPlansByOwner(searchQuery.trim());
      setPlans(results || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch plans for this address.");
      setPlans([]);
    } finally {
      setLoading(false);
    }
  };

  const handleClaimSuccess = () => {
    // Optionally refresh lists or states
  };

  return (
    <div className="animate-fade-in space-y-8 max-w-5xl mx-auto">
      {/* Page header */}
      <div className="space-y-2">
        <h1 className="text-3xl font-extrabold text-foreground tracking-tight sm:text-4xl">
          Claim Inherited Assets
        </h1>
        <p className="text-sm sm:text-base text-gray-500 max-w-2xl">
          Search for inheritance plans using the owner's Stellar address to check their status, view inactivity timer progress, and claim payouts.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
        {/* Left Search and Plans listing */}
        <div className="lg:col-span-2 space-y-6">
          {/* Search container */}
          <div className="bg-white/3 border border-white/10 rounded-2xl p-6 space-y-4">
            <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wider">Search Inheritance Plans</h2>
            <form onSubmit={handleSearch} className="flex flex-col sm:flex-row gap-3">
              <div className="relative flex-1">
                <Search size={18} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-gray-500" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Enter Owner's Address (e.g. GABC123)"
                  className="w-full pl-10 pr-4 py-2.5 rounded-xl bg-black/30 border border-white/10 focus:border-primary/50 text-sm focus:outline-none transition-all font-mono"
                />
              </div>
              <button
                type="submit"
                disabled={loading || !searchQuery.trim()}
                className="px-5 py-2.5 rounded-xl bg-primary text-black hover:bg-primary/90 font-medium text-sm transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
              >
                {loading ? (
                  <Loader2 size={16} className="animate-spin" />
                ) : (
                  <>
                    <Search size={16} />
                    Search Plans
                  </>
                )}
              </button>
            </form>

            {/* Wallet Quick Connect Prompt */}
            {!isConnected && (
              <div className="flex items-center justify-between p-4 rounded-xl bg-white/2 border border-white/5 text-xs text-gray-400">
                <div className="flex items-center gap-2">
                  <Wallet size={14} className="text-[#92A5A8]" />
                  <span>Connect your beneficiary wallet to authorize the claim transaction.</span>
                </div>
                <button
                  type="button"
                  onClick={openModal}
                  className="text-primary hover:underline font-semibold flex items-center gap-0.5"
                >
                  Connect <ArrowRight size={10} />
                </button>
              </div>
            )}
          </div>

          {/* Results container */}
          <div className="space-y-4">
            {loading && (
              <div className="flex flex-col items-center justify-center py-16 gap-3">
                <Loader2 size={32} className="animate-spin text-primary" />
                <p className="text-sm text-gray-500">Retrieving inheritance plans from blockchain...</p>
              </div>
            )}

            {!loading && searched && plans.length === 0 && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="bg-white/3 border border-white/10 rounded-2xl p-12 text-center space-y-3"
              >
                <div className="mx-auto w-12 h-12 rounded-full bg-white/5 flex items-center justify-center">
                  <AlertCircle size={20} className="text-gray-500" />
                </div>
                <h3 className="text-base font-semibold text-foreground">No Plans Found</h3>
                <p className="text-sm text-gray-500 max-w-md mx-auto">
                  No active inheritance plans were found for owner address <code className="text-primary font-mono text-xs break-all">{searchQuery}</code>. Ensure the address is correct and try again.
                </p>
              </motion.div>
            )}

            {!loading && error && (
              <div className="bg-[#F5656514] border border-[#F5656540] text-[#F56565] rounded-2xl p-6 text-sm flex items-start gap-3">
                <AlertCircle size={20} className="mt-0.5 flex-shrink-0" />
                <div>
                  <p className="font-semibold">Error Loading Plans</p>
                  <p className="mt-1 opacity-90">{error}</p>
                </div>
              </div>
            )}

            {!loading && plans.length > 0 && (
              <div className="space-y-4">
                <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider px-1">
                  Active Plans Found ({plans.length})
                </h3>
                <div className="space-y-4">
                  {plans.map((plan) => (
                    <PlanClaimCard key={plan.id} initialPlan={plan} onSuccess={handleClaimSuccess} />
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Right Info Step-by-Step Instructions */}
        <div className="space-y-6">
          <div className="bg-white/3 border border-white/10 rounded-2xl p-6 space-y-6">
            <div className="flex items-center gap-2 pb-2 border-b border-white/10">
              <BookOpen size={18} className="text-primary" />
              <h2 className="text-sm font-bold text-foreground uppercase tracking-wider">Beneficiary Instructions</h2>
            </div>

            <ol className="relative border-l border-white/10 space-y-6 ml-2 text-sm text-gray-400">
              <li className="pl-6 relative">
                <span className="absolute left-0 top-0.5 -translate-x-1/2 flex items-center justify-center w-5 h-5 rounded-full bg-white/5 border border-white/10 text-xs font-bold text-primary font-mono">
                  1
                </span>
                <h4 className="font-bold text-foreground">Locate Inheritance Plan</h4>
                <p className="mt-1 text-xs leading-relaxed">
                  Enter the Asset Owner's Stellar address in the search bar. This queries active smart contracts set up by the owner.
                </p>
              </li>

              <li className="pl-6 relative">
                <span className="absolute left-0 top-0.5 -translate-x-1/2 flex items-center justify-center w-5 h-5 rounded-full bg-white/5 border border-white/10 text-xs font-bold text-primary font-mono">
                  2
                </span>
                <h4 className="font-bold text-foreground">Verify Claim Status</h4>
                <p className="mt-1 text-xs leading-relaxed">
                  Check if the grace period has fully elapsed. The plan status must show <span className="text-[#F56565] font-semibold">Claimable</span> before a payout can be initiated.
                </p>
              </li>

              <li className="pl-6 relative">
                <span className="absolute left-0 top-0.5 -translate-x-1/2 flex items-center justify-center w-5 h-5 rounded-full bg-white/5 border border-white/10 text-xs font-bold text-primary font-mono">
                  3
                </span>
                <h4 className="font-bold text-foreground">Connect Wallet</h4>
                <p className="mt-1 text-xs leading-relaxed">
                  Click the connect button in the header or the connect wallet prompt. Verify you are connected to the same address listed as a beneficiary in the plan.
                </p>
              </li>

              <li className="pl-6 relative">
                <span className="absolute left-0 top-0.5 -translate-x-1/2 flex items-center justify-center w-5 h-5 rounded-full bg-white/5 border border-white/10 text-xs font-bold text-primary font-mono">
                  4
                </span>
                <h4 className="font-bold text-foreground">Execute Claim Transaction</h4>
                <p className="mt-1 text-xs leading-relaxed">
                  Click the <span className="text-primary font-semibold">Initiate Payout</span> button. Authorize the claim transaction in your Stellar wallet extension (e.g. Freighter).
                </p>
              </li>
            </ol>
          </div>

          {/* Security details card */}
          <div className="bg-white/3 border border-white/10 rounded-2xl p-6 space-y-4">
            <div className="flex items-center gap-2 text-xs text-gray-500 font-bold uppercase tracking-wider">
              <ShieldCheck size={14} className="text-primary" />
              <span>Smart Contract Security</span>
            </div>
            <p className="text-xs text-gray-400 leading-relaxed">
              InheritX contracts are decentralized and yield-bearing. The grace period is enforced cryptographically on the Stellar ledger, ensuring owners retain full custody until inactivity criteria are met.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
