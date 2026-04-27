"use client";

import React, { useState, useEffect, useCallback } from "react";
import { 
  Plus, 
  LayoutDashboard, 
  History,
  Info,
} from "lucide-react";
import { useWallet } from "@/context/WalletContext";
import { loanService, Loan } from "@/app/services/loanService";
import ActiveLoansDashboard from "./components/ActiveLoansDashboard";
import LoanCreationWizard from "./components/LoanCreationWizard";
import LiquidationWarning from "./components/LiquidationWarning";
import LoanHistory from "./components/LoanHistory";

export default function BorrowingPage() {
  const { address, isConnected } = useWallet();
  const [activeTab, setActiveTab] = useState<"dashboard" | "create" | "history">("dashboard");
  const [loans, setLoans] = useState<Loan[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  useEffect(() => {
    let isMounted = true;

    const loadLoans = async () => {
      if (!address || !isConnected) {
        if (isMounted) setIsLoading(false);
        return;
      }

      if (isMounted) setIsLoading(true);
      try {
        const userLoans = await loanService.getUserLoans(address);
        if (isMounted) setLoans(userLoans);
      } catch (error) {
        console.error("Error fetching loans:", error);
      } finally {
        if (isMounted) setIsLoading(false);
      }
    };

    loadLoans();

    return () => {
      isMounted = false;
    };
  }, [isConnected, address]);

  const fetchLoans = useCallback(async () => {
    if (address) {
      const userLoans = await loanService.getUserLoans(address);
      setLoans(userLoans);
    }
  }, [address]);

  const unhealthyLoans = loans.filter(l => l.status === "ACTIVE" && l.healthFactor < 1.3);

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-medium text-[#FCFFFF]">Borrowing</h1>
          <p className="text-sm text-[#92A5A8]">
            Borrow assets against your collateral with ease.
          </p>
        </div>
        <div className="flex gap-x-3">
          <button 
            onClick={() => setActiveTab("dashboard")}
            className={`px-4 py-2 rounded-lg flex items-center gap-x-2 transition-all ${activeTab === "dashboard" ? "bg-[#33C5E0] text-[#161E22]" : "bg-[#182024] text-[#92A5A8] hover:text-[#FCFFFF]"}`}
          >
            <LayoutDashboard size={18} />
            Dashboard
          </button>
          <button 
            onClick={() => setActiveTab("create")}
            className={`px-4 py-2 rounded-lg flex items-center gap-x-2 transition-all ${activeTab === "create" ? "bg-[#33C5E0] text-[#161E22]" : "bg-[#182024] text-[#92A5A8] hover:text-[#FCFFFF]"}`}
          >
            <Plus size={18} />
            New Loan
          </button>
          <button 
            onClick={() => setActiveTab("history")}
            className={`px-4 py-2 rounded-lg flex items-center gap-x-2 transition-all ${activeTab === "history" ? "bg-[#33C5E0] text-[#161E22]" : "bg-[#182024] text-[#92A5A8] hover:text-[#FCFFFF]"}`}
          >
            <History size={18} />
            History
          </button>
        </div>
      </div>

      {/* Liquidation Warnings */}
      {unhealthyLoans.length > 0 && (
        <div className="space-y-4">
          {unhealthyLoans.map(loan => (
            <LiquidationWarning key={loan.id} loan={loan} />
          ))}
        </div>
      )}

      {/* Main Content */}
      <div className="min-h-[500px]">
        {!isConnected ? (
          <div className="flex flex-col items-center justify-center h-[400px] text-center bg-[#182024] rounded-3xl p-8 border border-[#2A3338]">
            <Info className="text-[#33C5E0] mb-4" size={48} />
            <h2 className="text-xl text-[#FCFFFF] font-semibold mb-2">Connect Your Wallet</h2>
            <p className="text-[#92A5A8] max-w-md">
              Please connect your wallet to view your active loans and start borrowing.
            </p>
          </div>
        ) : isLoading ? (
          <div className="flex items-center justify-center h-[400px]">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-[#33C5E0]"></div>
          </div>
        ) : activeTab === "dashboard" ? (
          <ActiveLoansDashboard loans={loans.filter(l => l.status === "ACTIVE")} onRefresh={fetchLoans} />
        ) : activeTab === "create" ? (
          <LoanCreationWizard onSuccess={() => {
            setActiveTab("dashboard");
            fetchLoans();
          }} />
        ) : (
          <LoanHistory loans={loans.filter(l => l.status !== "ACTIVE")} />
        )}
      </div>
    </div>
  );
}
