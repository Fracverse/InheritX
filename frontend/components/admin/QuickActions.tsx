"use client";
import { useState } from "react";
import { AdminUser } from "@/lib/adminTypes";

interface QuickActionsProps {
  user: AdminUser;
  onApproveKYC: (userId: string) => Promise<void>;
  onTerminatePlan: (userId: string) => Promise<void>;
  onSuspendUser: (userId: string) => Promise<void>;
}

export function QuickActions({
  user,
  onApproveKYC,
  onTerminatePlan,
  onSuspendUser,
}: QuickActionsProps) {
  const [loadingAction, setLoadingAction] = useState<string | null>(null);

  const handle = async (action: string, fn: () => Promise<void>) => {
    setLoadingAction(action);
    try {
      await fn();
    } finally {
      setLoadingAction(null);
    }
  };

  return (
    <div className="flex items-center gap-2">
      {/* Approve KYC — only show if pending */}
      {user.kycStatus === "pending" && (
        <button
          onClick={() => handle("kyc", () => onApproveKYC(user.id))}
          disabled={loadingAction !== null}
          className="px-2.5 py-1 text-xs font-medium rounded-md bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 hover:bg-emerald-500/20 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {loadingAction === "kyc" ? "..." : "Approve KYC"}
        </button>
      )}

      {/* Terminate Plan — only show if active plans exist */}
      {user.activePlansCount > 0 && (
        <button
          onClick={() => handle("terminate", () => onTerminatePlan(user.id))}
          disabled={loadingAction !== null}
          className="px-2.5 py-1 text-xs font-medium rounded-md bg-orange-500/10 text-orange-400 border border-orange-500/20 hover:bg-orange-500/20 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {loadingAction === "terminate" ? "..." : "Terminate Plan"}
        </button>
      )}

      {/* Suspend / Unsuspend */}
      <button
        onClick={() => handle("suspend", () => onSuspendUser(user.id))}
        disabled={loadingAction !== null}
        className={`px-2.5 py-1 text-xs font-medium rounded-md border disabled:opacity-50 disabled:cursor-not-allowed transition-colors ${
          user.status === "active"
            ? "bg-red-500/10 text-red-400 border-red-500/20 hover:bg-red-500/20"
            : "bg-primary/10 text-primary border-primary/20 hover:bg-primary/20"
        }`}
      >
        {loadingAction === "suspend"
          ? "..."
          : user.status === "active"
          ? "Suspend"
          : "Unsuspend"}
      </button>
    </div>
  );
}
