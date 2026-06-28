"use client";

import { ReactNode } from "react";
import { motion } from "framer-motion";
import { AlertCircle, Lock } from "lucide-react";
import { useKYC } from "@/context/KYCContext";

interface KYCRequiredGuardProps {
  children: ReactNode;
  fallback?: ReactNode;
}

export function KYCRequiredGuard({ children, fallback }: KYCRequiredGuardProps) {
  const { canCreatePlan, openKYCModal, kycStatus, isLoading } = useKYC();

  if (isLoading) {
    return (
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="p-6 rounded-lg bg-black/20 border border-[#2A3338] flex items-center justify-center"
      >
        <div className="text-center">
          <div className="w-8 h-8 rounded-full border-2 border-[#2A3338] border-t-[#33C5E0] animate-spin mx-auto mb-3" />
          <p className="text-sm text-[#92A5A8]">Loading KYC status...</p>
        </div>
      </motion.div>
    );
  }

  if (canCreatePlan) {
    return <>{children}</>;
  }

  if (fallback) {
    return <>{fallback}</>;
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      className="p-6 rounded-lg bg-[#ED893614] border border-[#ED893640] space-y-3"
    >
      <div className="flex items-start gap-3">
        <Lock size={20} className="text-[#ED8936] mt-0.5 flex-shrink-0" />
        <div className="flex-1">
          <h3 className="text-sm font-semibold text-[#ED8936] mb-1">KYC Verification Required</h3>
          <p className="text-xs text-[#92A5A8] mb-3">
            {kycStatus === "pending"
              ? "Complete KYC verification to create and manage plans."
              : kycStatus === "submitted"
                ? "Your KYC is under review. You'll be able to create plans once approved."
                : kycStatus === "rejected"
                  ? "Your KYC was rejected. Please resubmit with correct information."
                  : "Complete your KYC verification to proceed."}
          </p>
          <button
            onClick={openKYCModal}
            className="inline-flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-[#ED8936] hover:bg-[#ED8936]/90 rounded-lg transition-colors"
          >
            <AlertCircle size={14} />
            {kycStatus === "rejected" ? "Resubmit KYC" : "Start KYC Verification"}
          </button>
        </div>
      </div>
    </motion.div>
  );
}

export default KYCRequiredGuard;
