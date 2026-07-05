"use client";

import { motion } from "framer-motion";
import { Clock, RefreshCw } from "lucide-react";
import { useKYC } from "@/context/KYCContext";
import type { KYCResponse } from "@/app/lib/api/kyc";

interface VerificationPendingViewProps {
  response: KYCResponse;
}

export function VerificationPendingView({ response }: VerificationPendingViewProps) {
  const { refreshKYCStatus } = useKYC();

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      className="p-6 text-center space-y-4"
    >
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ delay: 0.2, type: "spring" }}
      >
        <motion.div animate={{ rotate: 360 }} transition={{ duration: 2, repeat: Infinity }}>
          <Clock size={64} className="mx-auto text-[#33C5E0]" />
        </motion.div>
      </motion.div>

      <h2 className="text-2xl font-bold text-[#33C5E0]">Under Review</h2>

      <p className="text-sm text-[#92A5A8] max-w-sm mx-auto">
        Your KYC verification is being reviewed. This typically takes 24-48 hours. You'll
        receive an email notification once the review is complete.
      </p>

      <div className="pt-4 bg-black/20 rounded-lg p-4 space-y-3">
        <div className="space-y-2 text-left">
          <p className="text-xs font-semibold text-slate-200">What happens next:</p>
          <ol className="text-xs text-[#92A5A8] space-y-1 list-inside list-decimal">
            <li>Our team reviews your documents</li>
            <li>We verify your information</li>
            <li>You receive approval or rejection</li>
          </ol>
        </div>
      </div>

      <button
        onClick={refreshKYCStatus}
        className="flex items-center justify-center gap-2 mx-auto px-4 py-2 text-sm text-[#33C5E0] hover:text-cyan-300 transition-colors"
      >
        <RefreshCw size={14} />
        Check Status
      </button>
    </motion.div>
  );
}
