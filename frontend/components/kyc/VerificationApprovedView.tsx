"use client";

import { motion } from "framer-motion";
import { CheckCircle, Calendar } from "lucide-react";
import type { KYCResponse } from "@/app/lib/api/kyc";

interface VerificationApprovedViewProps {
  response: KYCResponse;
}

export function VerificationApprovedView({ response }: VerificationApprovedViewProps) {
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
        <CheckCircle size={64} className="mx-auto text-[#48BB78]" />
      </motion.div>

      <h2 className="text-2xl font-bold text-[#48BB78]">Verification Approved</h2>

      <p className="text-sm text-[#92A5A8] max-w-sm mx-auto">
        Your KYC verification has been approved. You can now create and manage inheritance
        plans with full features.
      </p>

      {response.approved_at && (
        <div className="flex items-center justify-center gap-2 text-xs text-[#92A5A8]">
          <Calendar size={14} />
          <span>
            Approved on{" "}
            {new Date(response.approved_at).toLocaleDateString(undefined, {
              month: "short",
              day: "numeric",
              year: "numeric",
            })}
          </span>
        </div>
      )}

      <div className="pt-4 bg-black/20 rounded-lg p-4 space-y-2 text-xs text-[#92A5A8]">
        <p>
          <span className="font-semibold text-slate-200">Wallet:</span>
          <br />
          <span className="font-mono">{response.wallet_address}</span>
        </p>
        {response.provider_reference && (
          <p>
            <span className="font-semibold text-slate-200">Reference:</span>
            <br />
            <span className="font-mono text-[10px]">{response.provider_reference}</span>
          </p>
        )}
      </div>
    </motion.div>
  );
}
