"use client";

import React, { useState } from "react";
import { X, CheckCircle2, AlertCircle, Loader2 } from "lucide-react";
import { loanService, Loan } from "@/app/services/loanService";

interface RepaymentModalProps {
  loan: Loan;
  onClose: () => void;
  onSuccess: () => void;
}

export default function RepaymentModal({ loan, onClose, onSuccess }: RepaymentModalProps) {
  const [isRepaying, setIsRepaying] = useState(false);
  const [step, setStep] = useState<"confirm" | "success" | "error">("confirm");
  const [error, setError] = useState("");

  const handleRepay = async () => {
    setIsRepaying(true);
    try {
      const result = await loanService.repayLoan(loan.id);
      if (result.success) {
        setStep("success");
      } else {
        setError(result.error || "Failed to repay loan");
        setStep("error");
      }
    } catch (err) {
      setError("An unexpected error occurred");
      setStep("error");
    } finally {
      setIsRepaying(false);
    }
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center p-6 bg-[#161E22CC] backdrop-blur-sm">
      <div className="bg-[#182024] w-full max-w-md rounded-[32px] border border-[#2A3338] shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-[#2A3338]">
          <h2 className="text-xl font-semibold text-[#FCFFFF]">Repay Loan</h2>
          <button onClick={onClose} className="text-[#92A5A8] hover:text-[#FCFFFF]">
            <X size={24} />
          </button>
        </div>

        <div className="p-8">
          {step === "confirm" && (
            <div className="space-y-6">
              <div className="bg-[#1C252A] p-6 rounded-2xl space-y-4">
                <div className="flex justify-between">
                  <span className="text-[#92A5A8]">Loan ID</span>
                  <span className="text-[#FCFFFF]">#{loan.id.slice(0, 8)}...</span>
                </div>
                <div className="flex justify-between font-semibold">
                  <span className="text-[#92A5A8]">Total Repayment</span>
                  <span className="text-[#33C5E0]">{loan.repaymentAmount} XLM</span>
                </div>
                <div className="pt-4 border-t border-[#2A3338] text-xs text-[#92A5A8]">
                  This includes the principal amount of {loan.borrowedAmount} XLM plus accrued interest.
                </div>
              </div>

              <div className="flex items-start gap-x-3 text-sm bg-yellow-500/10 p-4 rounded-xl text-yellow-500">
                <AlertCircle size={20} className="shrink-0" />
                <p>Ensure you have enough balance to cover the repayment. Collateral will be released immediately after success.</p>
              </div>

              <button 
                onClick={handleRepay}
                disabled={isRepaying}
                className="w-full bg-[#33C5E0] text-[#161E22] py-4 rounded-full font-bold flex items-center justify-center gap-x-2 disabled:opacity-50"
              >
                {isRepaying ? (
                  <>
                    <Loader2 className="animate-spin" size={20} />
                    Processing...
                  </>
                ) : (
                  "Confirm Repayment"
                )}
              </button>
            </div>
          )}

          {step === "success" && (
            <div className="text-center space-y-6 py-4">
              <div className="bg-green-500/20 w-20 h-20 rounded-full flex items-center justify-center mx-auto text-green-500">
                <CheckCircle2 size={48} />
              </div>
              <div>
                <h3 className="text-2xl font-bold text-[#FCFFFF] mb-2">Success!</h3>
                <p className="text-[#92A5A8]">
                  Loan repaid and collateral released.
                </p>
              </div>
              <button 
                onClick={onSuccess}
                className="w-full bg-[#1C252A] text-[#FCFFFF] py-4 rounded-full font-bold"
              >
                Back to Dashboard
              </button>
            </div>
          )}

          {step === "error" && (
            <div className="text-center space-y-6 py-4">
              <div className="bg-red-500/20 w-20 h-20 rounded-full flex items-center justify-center mx-auto text-red-500">
                <AlertCircle size={48} />
              </div>
              <div>
                <h3 className="text-2xl font-bold text-[#FCFFFF] mb-2">Failed</h3>
                <p className="text-red-400">
                  {error}
                </p>
              </div>
              <button 
                onClick={() => setStep("confirm")}
                className="w-full bg-[#1C252A] text-[#FCFFFF] py-4 rounded-full font-bold"
              >
                Try Again
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
