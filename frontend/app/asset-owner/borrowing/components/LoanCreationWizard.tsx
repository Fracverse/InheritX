"use client";

import React, { useState, useEffect, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { 
  ChevronRight, 
  ChevronLeft, 
  Wallet, 
  Clock, 
  Search,
  CheckCircle2,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { useWallet } from "@/context/WalletContext";
import { loanService, LoanSimulation } from "@/app/services/loanService";

interface LoanCreationWizardProps {
  onSuccess: () => void;
}

const STEPS = [
  "Select Collateral",
  "Loan Amount",
  "Duration",
  "Review Terms",
  "Approval",
  "Finalize"
];

const ASSETS = [
  { id: "XLM", name: "Stellar Lumens", icon: "🚀", balance: "1500.00" },
  { id: "USDC", name: "USD Coin", icon: "💵", balance: "500.00" },
  { id: "YBX", name: "YieldBlox", icon: "📊", balance: "2500.00" }
];

export default function LoanCreationWizard({ onSuccess }: LoanCreationWizardProps) {
  const { address } = useWallet();
  const [currentStep, setCurrentStep] = useState(0);
  const [formData, setFormData] = useState({
    collateralAsset: "XLM",
    collateralAmount: "",
    borrowAmount: "",
    duration: 30
  });
  const [simulation, setSimulation] = useState<LoanSimulation | null>(null);
  const [isSimulating, setIsSimulating] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState("");

  const nextStep = () => setCurrentStep(prev => Math.min(prev + 1, STEPS.length - 1));
  const prevStep = () => setCurrentStep(prev => Math.max(prev - 1, 0));

  const handleSimulate = useCallback(async () => {
    setIsSimulating(true);
    setError("");
    const result = await loanService.simulateLoan(formData);
    if (result) {
      setSimulation(result);
    } else {
      setError("Failed to simulate loan terms. Please check your inputs.");
    }
    setIsSimulating(false);
  }, [formData]);

  useEffect(() => {
    if (currentStep === 3) {
      handleSimulate();
    }
  }, [currentStep, handleSimulate]);

  const handleFinalize = async () => {
    setIsProcessing(true);
    setError("");
    try {
      if (!address) throw new Error("Wallet not connected");
      const result = await loanService.createLoan({
        ...formData,
        address
      });
      if (result.success) {
        onSuccess();
      } else {
        setError(result.error || "Failed to create loan");
      }
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : "An unexpected error occurred";
      setError(errorMessage);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="bg-[#182024] rounded-[40px] border border-[#2A3338] overflow-hidden shadow-2xl max-w-4xl mx-auto">
      {/* Wizard Header / Progress */}
      <div className="p-8 border-b border-[#2A3338]">
        <div className="flex justify-between items-center mb-8">
          <h2 className="text-xl font-semibold text-[#FCFFFF]">Create New Loan</h2>
          <span className="text-sm text-[#92A5A8]">Step {currentStep + 1} of {STEPS.length}</span>
        </div>
        <div className="flex gap-2">
          {STEPS.map((step, idx) => (
            <div 
              key={step}
              className={`h-1.5 flex-1 rounded-full transition-all duration-300 ${idx <= currentStep ? "bg-[#33C5E0]" : "bg-[#2A3338]"}`}
            />
          ))}
        </div>
      </div>

      {/* Step Content */}
      <div className="p-8 min-h-[400px]">
        <AnimatePresence mode="wait">
          <motion.div
            key={currentStep}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            transition={{ duration: 0.2 }}
          >
            {currentStep === 0 && (
              <div className="space-y-6">
                <div className="flex justify-between items-end">
                  <h3 className="text-lg font-medium text-[#FCFFFF]">Select Collateral Asset</h3>
                  <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-[#92A5A8]" size={16} />
                    <input 
                      type="text" 
                      placeholder="Search assets..." 
                      className="bg-[#1C252A] border border-[#2A3338] rounded-xl pl-10 pr-4 py-2 text-sm focus:outline-none focus:border-[#33C5E0]"
                    />
                  </div>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  {ASSETS.map(asset => (
                    <button
                      key={asset.id}
                      onClick={() => setFormData({ ...formData, collateralAsset: asset.id })}
                      className={`p-6 rounded-3xl border-2 transition-all text-left ${formData.collateralAsset === asset.id ? "border-[#33C5E0] bg-[#33C5E014]" : "border-[#2A3338] bg-[#1C252A] hover:border-[#33C5E03D]"}`}
                    >
                      <div className="text-3xl mb-4">{asset.icon}</div>
                      <div className="text-[#FCFFFF] font-bold">{asset.id}</div>
                      <div className="text-[#92A5A8] text-sm">{asset.name}</div>
                      <div className="mt-4 pt-4 border-t border-[#2A3338] flex justify-between items-center">
                        <span className="text-[10px] text-[#92A5A8] uppercase">Balance</span>
                        <span className="text-xs text-[#FCFFFF] font-medium">{asset.balance}</span>
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {currentStep === 1 && (
              <div className="space-y-8 max-w-lg mx-auto py-8">
                <div className="text-center">
                  <h3 className="text-xl font-medium text-[#FCFFFF] mb-2">How much do you want to borrow?</h3>
                  <p className="text-[#92A5A8]">Your loan is backed by your {formData.collateralAsset} collateral.</p>
                </div>
                <div className="space-y-4">
                  <div className="bg-[#1C252A] p-6 rounded-3xl border border-[#2A3338]">
                    <div className="flex justify-between mb-2">
                      <label className="text-sm text-[#92A5A8]">Borrow Amount (XLM)</label>
                      <span className="text-xs text-[#33C5E0]">Max: 1000 XLM</span>
                    </div>
                    <input 
                      type="number"
                      value={formData.borrowAmount}
                      onChange={(e) => setFormData({ ...formData, borrowAmount: e.target.value })}
                      placeholder="0.00"
                      className="bg-transparent text-3xl font-bold text-[#FCFFFF] focus:outline-none w-full"
                    />
                  </div>
                  <div className="bg-[#1C252A] p-6 rounded-3xl border border-[#2A3338]">
                    <div className="flex justify-between mb-2">
                      <label className="text-sm text-[#92A5A8]">Collateral to Lock ({formData.collateralAsset})</label>
                      <span className="text-xs text-[#92A5A8]">Available: {ASSETS.find(a => a.id === formData.collateralAsset)?.balance}</span>
                    </div>
                    <input 
                      type="number"
                      value={formData.collateralAmount}
                      onChange={(e) => setFormData({ ...formData, collateralAmount: e.target.value })}
                      placeholder="0.00"
                      className="bg-transparent text-3xl font-bold text-[#FCFFFF] focus:outline-none w-full"
                    />
                  </div>
                </div>
              </div>
            )}

            {currentStep === 2 && (
              <div className="space-y-8 max-w-lg mx-auto py-8 text-center">
                <div>
                  <h3 className="text-xl font-medium text-[#FCFFFF] mb-2">Set Loan Duration</h3>
                  <p className="text-[#92A5A8]">Choose how long you want to keep the loan.</p>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  {[7, 30, 60, 90].map(days => (
                    <button
                      key={days}
                      onClick={() => setFormData({ ...formData, duration: days })}
                      className={`p-6 rounded-2xl border-2 transition-all flex flex-col items-center ${formData.duration === days ? "border-[#33C5E0] bg-[#33C5E014]" : "border-[#2A3338] bg-[#1C252A] hover:border-[#33C5E03D]"}`}
                    >
                      <Clock className={formData.duration === days ? "text-[#33C5E0]" : "text-[#92A5A8]"} size={32} />
                      <span className="mt-2 font-bold text-lg text-[#FCFFFF]">{days} Days</span>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {currentStep === 3 && (
              <div className="space-y-6 py-4">
                <h3 className="text-lg font-medium text-[#FCFFFF]">Review Loan Terms</h3>
                {isSimulating ? (
                  <div className="flex flex-col items-center justify-center py-20 space-y-4">
                    <Loader2 className="animate-spin text-[#33C5E0]" size={48} />
                    <p className="text-[#92A5A8]">Simulating terms based on current market rates...</p>
                  </div>
                ) : error ? (
                  <div className="bg-red-500/10 border border-red-500/30 p-6 rounded-2xl text-center">
                    <AlertCircle className="text-red-500 mx-auto mb-4" size={48} />
                    <p className="text-[#FCFFFF] font-semibold mb-2">Simulation Failed</p>
                    <p className="text-red-400 text-sm mb-6">{error}</p>
                    <button onClick={handleSimulate} className="bg-red-500/20 text-red-500 px-6 py-2 rounded-xl text-sm font-bold">Retry</button>
                  </div>
                ) : simulation && (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="bg-[#1C252A] p-8 rounded-[32px] border border-[#2A3338] space-y-6">
                      <div className="flex justify-between">
                        <span className="text-[#92A5A8]">Interest Rate (APR)</span>
                        <span className="text-[#33C5E0] font-bold">{simulation.interestRate}%</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-[#92A5A8]">Estimated Health Factor</span>
                        <span className={`font-bold ${simulation.healthFactor >= 1.5 ? "text-green-500" : "text-yellow-500"}`}>
                          {simulation.healthFactor.toFixed(2)}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-[#92A5A8]">Liquidation Price</span>
                        <span className="text-red-500 font-bold">${simulation.liquidationPrice.toFixed(4)} / XLM</span>
                      </div>
                    </div>
                    <div className="bg-[#1C252A] p-8 rounded-[32px] border border-[#2A3338] space-y-6">
                      <div className="flex justify-between">
                        <span className="text-[#92A5A8]">Total Borrowed</span>
                        <span className="text-[#FCFFFF] font-bold">{simulation.borrowedAmount} XLM</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-[#92A5A8]">Collateral Locked</span>
                        <span className="text-[#FCFFFF] font-bold">{formData.collateralAmount} {formData.collateralAsset}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-[#92A5A8]">Repayment Date</span>
                        <span className="text-[#FCFFFF] font-bold">
                          {new Date(Date.now() + formData.duration * 24 * 60 * 60 * 1000).toLocaleDateString()}
                        </span>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}

            {currentStep === 4 && (
              <div className="flex flex-col items-center justify-center py-12 space-y-8 text-center max-w-md mx-auto">
                <div className="bg-[#33C5E014] w-24 h-24 rounded-full flex items-center justify-center text-[#33C5E0]">
                  <Wallet size={48} />
                </div>
                <div>
                  <h3 className="text-2xl font-bold text-[#FCFFFF] mb-2">Approve Collateral</h3>
                  <p className="text-[#92A5A8]">
                    Before creating the loan, you need to grant the smart contract permission to lock your {formData.collateralAmount} {formData.collateralAsset}.
                  </p>
                </div>
                <button 
                  onClick={nextStep}
                  className="w-full bg-[#33C5E0] text-[#161E22] py-4 rounded-full font-bold hover:bg-[#2bb2cc] transition-colors"
                >
                  Approve Transaction
                </button>
              </div>
            )}

            {currentStep === 5 && (
              <div className="flex flex-col items-center justify-center py-12 space-y-8 text-center max-w-md mx-auto">
                <div className="bg-green-500/10 w-24 h-24 rounded-full flex items-center justify-center text-green-500">
                  <CheckCircle2 size={48} />
                </div>
                <div>
                  <h3 className="text-2xl font-bold text-[#FCFFFF] mb-2">Ready to Finalize</h3>
                  <p className="text-[#92A5A8]">
                    Everything is set. Confirm to create your loan and receive the assets in your wallet.
                  </p>
                </div>
                {error && <p className="text-red-500 text-sm">{error}</p>}
                <button 
                  onClick={handleFinalize}
                  disabled={isProcessing}
                  className="w-full bg-[#33C5E0] text-[#161E22] py-4 rounded-full font-bold hover:bg-[#2bb2cc] transition-colors flex items-center justify-center gap-x-2"
                >
                  {isProcessing ? (
                    <>
                      <Loader2 className="animate-spin" size={20} />
                      Creating Loan...
                    </>
                  ) : (
                    "Confirm & Create Loan"
                  )}
                </button>
              </div>
            )}
          </motion.div>
        </AnimatePresence>
      </div>

      {/* Wizard Footer Buttons */}
      <div className="p-8 bg-[#1C252A] flex justify-between">
        <button 
          onClick={prevStep}
          disabled={currentStep === 0 || isProcessing}
          className={`px-6 py-3 rounded-xl flex items-center gap-x-2 transition-all ${currentStep === 0 || isProcessing ? "opacity-0 pointer-events-none" : "text-[#92A5A8] hover:text-[#FCFFFF]"}`}
        >
          <ChevronLeft size={20} />
          Back
        </button>
        {currentStep < 4 && (
          <button 
            onClick={nextStep}
            disabled={
              (currentStep === 1 && (!formData.borrowAmount || !formData.collateralAmount)) ||
              (currentStep === 3 && (!simulation || isSimulating))
            }
            className={`px-8 py-3 bg-[#33C5E0] text-[#161E22] rounded-xl font-bold flex items-center gap-x-2 transition-all hover:bg-[#2bb2cc] disabled:opacity-50 disabled:cursor-not-allowed`}
          >
            Next
            <ChevronRight size={20} />
          </button>
        )}
      </div>
    </div>
  );
}
