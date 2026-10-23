"use client";

import { useState, useEffect, useCallback } from "react";
import Link from "next/link";
import { useWallet } from "@/context/WalletContext";
import { plansAPI, type Plan } from "@/app/lib/api/plans";
import { getPlans } from "@/lib/api/dataSource";
import { motion, AnimatePresence } from "framer-motion";
import {
  Plus,
  PlusCircle,
  Search,
  Filter,
  Loader2,
  AlertCircle,
  ShieldCheck,
  Clock,
  CheckCircle2,
  XCircle,
  ChevronRight,
  FileText,
  Edit3,
  Zap,
  TrendingUp,
  RefreshCw,
} from "lucide-react";
import { Skeleton } from "@/components/ui/Skeleton";
import { BeneficiaryMFAModal } from "@/components/plans/BeneficiaryMFAModal";
import { GeneticUploadModal } from "@/components/plans/GeneticUploadModal";

// ─── Types ───────────────────────────────────────────────────────────────────

type ViewMode = "all" | "due-for-claim" | "claimed";
type LoadingState = "idle" | "loading" | "error";

// ─── Status badge helper ────────────────────────────────────────────────────

function StatusBadge({ status }: { status: string }) {
  const normalized = (status ?? "").toLowerCase();
  const styles: Record<string, { bg: string; text: string; icon: typeof Clock }> = {
    active: { bg: "bg-[#33C5E014]", text: "text-[#33C5E0]", icon: Clock },
    claimable: { bg: "bg-[#F59E0B14]", text: "text-[#F59E0B]", icon: ShieldCheck },
    claimed: { bg: "bg-[#48BB7814]", text: "text-[#48BB78]", icon: CheckCircle2 },
    completed: { bg: "bg-[#48BB7814]", text: "text-[#48BB78]", icon: CheckCircle2 },
    expired: { bg: "bg-[#F5656514]", text: "text-[#F56565]", icon: XCircle },
    triggered: { bg: "bg-[#F59E0B14]", text: "text-[#F59E0B]", icon: AlertCircle },
    pending: { bg: "bg-[#F59E0B14]", text: "text-[#F59E0B]", icon: Clock },
    paused: { bg: "bg-[#92A5A814]", text: "text-[#92A5A8]", icon: Clock },
    inactive: { bg: "bg-[#92A5A814]", text: "text-[#92A5A8]", icon: Clock },
  };

  const config = styles[normalized] ?? {
    bg: "bg-[#92A5A814]",
    text: "text-[#92A5A8]",
    icon: Clock,
  };
  const Icon = config.icon;

  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-medium ${config.bg} ${config.text}`}
    >
      <Icon size={11} />
      {(status ?? "Unknown").charAt(0).toUpperCase() + (status ?? "").slice(1)}
    </span>
  );
}

// ─── Plan Card ───────────────────────────────────────────────────────────────

interface PlanCardProps {
  plan: Plan;
  onClaim: (plan: Plan) => void;
  onUpload?: (plan: Plan) => void;
}

function PlanCard({ plan, onClaim, onUpload }: PlanCardProps) {
  const isClaimable =
    (plan.status ?? "").toLowerCase() === "claimable" ||
    (plan.status ?? "").toLowerCase() === "triggered";

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.2 }}
      className="bg-[#161E22] border border-[#2A3338] rounded-2xl overflow-hidden hover:border-[#33C5E040] transition-colors"
    >
      {/* Header */}
      <div className="px-5 py-4 border-b border-[#2A3338]">
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            <h3 className="text-sm font-semibold text-white truncate">
              {plan.title}
            </h3>
            {plan.description && (
              <p className="text-xs text-[#92A5A8] mt-1 line-clamp-2">
                {plan.description}
              </p>
            )}
          </div>
          <StatusBadge status={plan.status} />
        </div>
      </div>

      {/* Body */}
      <div className="px-5 py-4 space-y-3">
        {plan.beneficiary_name && (
          <div className="flex items-center justify-between text-xs">
            <span className="text-[#92A5A8]">Beneficiary</span>
            <span className="text-slate-200 font-medium">
              {plan.beneficiary_name}
            </span>
          </div>
        )}
        <div className="flex items-center justify-between text-xs">
          <span className="text-[#92A5A8]">Net Amount</span>
          <span className="text-slate-200 font-mono">
            {plan.net_amount?.toFixed(2) ?? "0.00"} {plan.currency_preference ?? "USD"}
          </span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="text-[#92A5A8]">Created</span>
          <span className="text-slate-200">
            {plan.created_at ? new Date(plan.created_at).toLocaleDateString() : "—"}
          </span>
        </div>
      </div>

      {onUpload && (
        <div className="px-5 pb-4">
          <button type="button" onClick={() => onUpload(plan)}
            className="rounded-lg border border-[#33C5E0] px-3 py-2 text-xs text-[#33C5E0] hover:bg-[#33C5E014]">
            Upload genetic proof
          </button>
        </div>
      )}

      {/* Footer */}
      <div className="px-5 py-3 bg-[#0A0F11] border-t border-[#2A3338] flex items-center justify-between gap-3">
        <Link
          href={`/asset-owner/plans/${plan.id}/edit`}
          className="text-xs text-[#92A5A8] hover:text-white transition-colors flex items-center gap-1"
        >
          <Edit3 size={11} /> Edit
          <ChevronRight size={12} />
        </Link>
        {isClaimable ? (
          <button
            type="button"
            onClick={() => onClaim(plan)}
            className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors"
          >
            <ShieldCheck size={13} />
            Claim with MFA
          </button>
        ) : (plan.status ?? "").toLowerCase() === "claimed" ||
          (plan.status ?? "").toLowerCase() === "completed" ? (
          <span className="inline-flex items-center gap-1.5 px-3 py-1.5 text-[10px] text-[#48BB78] bg-[#48BB7814] rounded-lg">
            <CheckCircle2 size={12} />
            Claimed
          </span>
        ) : (
          <span className="text-[10px] text-[#92A5A8]">
            Not claimable yet
          </span>
        )}
      </div>
    </motion.div>
  );
}

// ─── Main Page ───────────────────────────────────────────────────────────────

export default function PlansPage() {
  const { isConnected, address, openModal } = useWallet();
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loadingState, setLoadingState] = useState<LoadingState>("loading");
  const [errorMessage, setErrorMessage] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [refreshing, setRefreshing] = useState(false);
  const [mfaPlan, setMfaPlan] = useState<Plan | null>(null);
  const [uploadPlan, setUploadPlan] = useState<Plan | null>(null);

  // ─── Fetch plans ──────────────────────────────────────────────────────────

  const fetchPlans = useCallback(async () => {
    setLoadingState("loading");
    setErrorMessage("");
    try {
      let fetchedPlans: Plan[] = [];

      if (viewMode === "due-for-claim") {
        const response = await plansAPI.getDueForClaimPlans(1, 50);
        fetchedPlans = response.data ?? [];
      } else {
        // Try dataSource first, fall back to plansAPI
        try {
          fetchedPlans = await getPlans(address ?? undefined);
        } catch {
          const response = await plansAPI.getDueForClaimPlans(1, 100);
          fetchedPlans = response.data ?? [];
        }
      }

      // Filter for claimed view
      if (viewMode === "claimed") {
        fetchedPlans = fetchedPlans.filter(
          (p) =>
            (p.status ?? "").toLowerCase() === "claimed" ||
            (p.status ?? "").toLowerCase() === "completed"
        );
      }

      setPlans(fetchedPlans);
      setLoadingState("idle");
    } catch (err) {
      setLoadingState("error");
      setErrorMessage(
        err instanceof Error ? err.message : "Failed to load plans."
      );
    } finally {
      setRefreshing(false);
    }
  }, [viewMode, address]);

  useEffect(() => {
    fetchPlans();
  }, [fetchPlans]);

  const handleRefresh = () => {
    setRefreshing(true);
    fetchPlans();
  };

  // ─── Filter plans by search ──────────────────────────────────────────────

  const filteredPlans = plans.filter((plan) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      (plan.title ?? "").toLowerCase().includes(q) ||
      (plan.beneficiary_name ?? "").toLowerCase().includes(q) ||
      (plan.id ?? "").toLowerCase().includes(q)
    );
  });

  // ─── Claim handler ──────────────────────────────────────────────────────

  const handleClaim = (plan: Plan) => {
    setMfaPlan(plan);
  };

  const handleClaimSuccess = (claimedPlan: Plan) => {
    setPlans((prev) =>
      prev.map((p) => (p.id === claimedPlan.id ? claimedPlan : p))
    );
    setMfaPlan(null);
  };

  // ─── Render ──────────────────────────────────────────────────────────────

  return (
    <div className="animate-fade-in space-y-6">
      {/* Page heading */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-foreground">
            Inheritance Plans
          </h1>
          <p className="text-sm text-gray-500 mt-1">
            View and claim your inheritance plans with multi-factor authentication.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleRefresh}
            disabled={refreshing || loadingState === "loading"}
            className="p-2 rounded-xl border border-[#2A3338] text-[#92A5A8] hover:text-white hover:bg-[#1C252A] transition-colors disabled:opacity-40"
            aria-label="Refresh"
          >
            <RefreshCw size={15} className={refreshing ? "animate-spin" : ""} />
          </button>
          <Link
            href="/asset-owner/plans/create"
            className="inline-flex items-center gap-2 px-4 py-2.5 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors w-fit"
          >
            <PlusCircle size={14} />
            Create Plan
          </Link>
        </div>
      </div>

      {/* Filters bar */}
      <div className="flex flex-col sm:flex-row gap-3">
        {/* Search */}
        <div className="relative flex-1">
          <Search
            size={15}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-[#4A5568]"
          />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search by title, beneficiary, or ID…"
            className="w-full bg-[#0A0F11] border border-[#2A3338] rounded-lg pl-9 pr-4 py-2.5 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>

        {/* View mode toggle */}
        <div className="flex items-center gap-1 p-1 bg-[#0A0F11] border border-[#2A3338] rounded-lg">
          {(["all", "due-for-claim", "claimed"] as ViewMode[]).map((mode) => (
            <button
              key={mode}
              type="button"
              onClick={() => setViewMode(mode)}
              className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
                viewMode === mode
                  ? "bg-[#33C5E0] text-black"
                  : "text-[#92A5A8] hover:text-white"
              }`}
            >
              {mode === "all"
                ? "All Plans"
                : mode === "due-for-claim"
                  ? "Due for Claim"
                  : "Claimed"}
            </button>
          ))}
        </div>
      </div>

      {/* Content */}
      {loadingState === "loading" && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className="h-64 rounded-2xl" />
          ))}
        </div>
      )}

      {loadingState === "error" && (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <AlertCircle size={32} className="text-[#F56565]" />
          <p className="text-sm text-[#F56565] mt-3">{errorMessage}</p>
          <button
            onClick={fetchPlans}
            className="mt-4 text-xs text-[#33C5E0] hover:text-cyan-300 transition-colors"
          >
            Try again
          </button>
        </div>
      )}

      {loadingState === "idle" && filteredPlans.length === 0 && (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <div className="flex items-center justify-center w-16 h-16 rounded-full bg-[#2A3338]">
            <FileText size={24} className="text-[#4A5568]" />
          </div>
          <p className="text-sm text-[#92A5A8] mt-4">
            {searchQuery
              ? "No plans match your search."
              : viewMode === "due-for-claim"
                ? "No plans are due for claim right now."
                : viewMode === "claimed"
                  ? "No claimed plans yet."
                  : "No plans found. Create one to get started."}
          </p>
          {!searchQuery && viewMode === "all" && (
            <Link
              href="/asset-owner/plans/create"
              className="mt-4 inline-flex items-center gap-2 px-4 py-2 text-xs font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors"
            >
              <Plus size={14} />
              Create your first plan
            </Link>
          )}
        </div>
      )}

      {loadingState === "idle" && filteredPlans.length > 0 && (
        <motion.div
          layout
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
        >
          <AnimatePresence>
            {filteredPlans.map((plan) => (
              <PlanCard key={plan.id} plan={plan} onClaim={handleClaim}
                onUpload={address && plan.owner_address === address ? setUploadPlan : undefined} />
            ))}
          </AnimatePresence>
        </motion.div>
      )}

      {/* MFA Modal */}
      {uploadPlan && address && uploadPlan.owner_address === address && (
        <GeneticUploadModal key={`${uploadPlan.id}:${address}`} planId={uploadPlan.id}
          planTitle={uploadPlan.title} onClose={() => setUploadPlan(null)} />
      )}
      {mfaPlan && (
        <BeneficiaryMFAModal
          plan={mfaPlan}
          open={!!mfaPlan}
          onClose={() => setMfaPlan(null)}
          onClaimSuccess={handleClaimSuccess}
        />
      )}
    </div>
  );
}
