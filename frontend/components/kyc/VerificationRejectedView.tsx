"use client";

import { motion } from "framer-motion";
import { XCircle, AlertCircle } from "lucide-react";
import { useKYC } from "@/context/KYCContext";
import type { KYCResponse } from "@/app/lib/api/kyc";

interface VerificationRejectedViewProps {
  response: KYCResponse;
}

export function VerificationRejectedView({ response }: VerificationRejectedViewProps) {
  const { openKYCModal } = useKYC();

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
        <XCircle size={64} className="mx-auto text-[#F56565]" />
      </motion.div>

      <h2 className="text-2xl font-bold text-[#F56565]">Verification Rejected</h2>

      <p className="text-sm text-[#92A5A8] max-w-sm mx-auto">
        Unfortunately, your KYC verification was not approved. You can resubmit with
        corrected information.
      </p>

      {response.rejection_reason && (
        <motion.div
          initial={{ opacity: 0, y: -8 }}
          animate={{ opacity: 1, y: 0 }}
          className="p-4 rounded-lg bg-[#F5656514] border border-[#F5656540] space-y-2"
        >
          <div className="flex items-start gap-2">
            <AlertCircle size={16} className="mt-0.5 text-[#F56565] flex-shrink-0" />
            <div className="text-left">
              <p className="text-xs font-semibold text-[#F56565]">Reason for rejection:</p>
              <p className="text-xs text-[#92A5A8] mt-1">{response.rejection_reason}</p>
            </div>
          </div>
        </motion.div>
      )}

      <div className="pt-4 bg-black/20 rounded-lg p-4 space-y-2 text-left">
        <p className="text-xs font-semibold text-slate-200">Common reasons for rejection:</p>
        <ul className="text-xs text-[#92A5A8] space-y-1 list-inside list-disc">
          <li>Documents are unclear or partially visible</li>
          <li>Document has expired</li>
          <li>Personal information doesn't match documents</li>
          <li>Selfie doesn't match ID document</li>
        </ul>
      </div>

      <button
        onClick={openKYCModal}
        className="w-full px-4 py-2 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors"
      >
        Try Again
      </button>
    </motion.div>
  );
}
