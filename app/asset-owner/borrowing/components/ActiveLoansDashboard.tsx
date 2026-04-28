"use client";

import React, { useState } from "react";
import { 
  ShieldCheck, 
  ArrowRightLeft,
} from "lucide-react";
import { Loan } from "@/app/services/loanService";
import RepaymentModal from "./RepaymentModal";

interface ActiveLoansDashboardProps {
  loans: Loan[];
  onRefresh: () => void;
}

export default function ActiveLoansDashboard({ loans, onRefresh }: ActiveLoansDashboardProps) {
  const [selectedLoan, setSelectedLoan] = useState<Loan | null>(null);

  const getHealthColor = (hf: number) => {
    if (hf >= 1.5) return "text-green-500";
    if (hf >= 1.3) return "text-yellow-500";
    return "text-red-500";
  };

  const getHealthBg = (hf: number) => {
    if (hf >= 1.5) return "bg-green-500/10 border-green-500/20";
    if (hf >= 1.3) return "bg-yellow-500/10 border-yellow-500/20";
    return "bg-red-500/10 border-red-500/20";
  };

  if (loans.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-[400px] text-center bg-[#182024] rounded-3xl p-8 border border-[#2A3338]">
        <ShieldCheck className="text-[#33C5E0] mb-4" size={48} />
        <h2 className="text-xl text-[#FCFFFF] font-semibold mb-2">No Active Loans</h2>
        <p className="text-[#92A5A8] max-w-md">
          You don&apos;t have any active loans at the moment. Click &quot;New Loan&quot; to get started.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Stats Summary */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
        <div className="bg-[#182024] p-6 rounded-2xl border border-[#2A3338]">
          <p className="text-[#92A5A8] text-sm mb-1">Total Borrowed</p>
          <h3 className="text-2xl font-bold text-[#FCFFFF]">
            {loans.reduce((acc, l) => acc + parseFloat(l.borrowedAmount), 0).toFixed(2)} XLM
          </h3>
        </div>
        <div className="bg-[#182024] p-6 rounded-2xl border border-[#2A3338]">
          <p className="text-[#92A5A8] text-sm mb-1">Total Collateral</p>
          <h3 className="text-2xl font-bold text-[#FCFFFF]">
            {loans.reduce((acc, l) => acc + parseFloat(l.collateralAmount), 0).toFixed(2)} XLM
          </h3>
        </div>
        <div className="bg-[#182024] p-6 rounded-2xl border border-[#2A3338]">
          <p className="text-[#92A5A8] text-sm mb-1">Avg. Health Factor</p>
          <h3 className={`text-2xl font-bold ${getHealthColor(loans.reduce((acc, l) => acc + l.healthFactor, 0) / loans.length)}`}>
            {(loans.reduce((acc, l) => acc + l.healthFactor, 0) / loans.length).toFixed(2)}
          </h3>
        </div>
      </div>

      {/* Loan List */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {loans.map((loan) => (
          <div key={loan.id} className="bg-[#182024] rounded-3xl p-6 border border-[#2A3338] hover:border-[#33C5E03D] transition-all group">
            <div className="flex justify-between items-start mb-6">
              <div className="flex items-center gap-x-3">
                <div className="bg-[#33C5E014] p-3 rounded-2xl text-[#33C5E0]">
                  <ArrowRightLeft size={24} />
                </div>
                <div>
                  <h4 className="text-[#FCFFFF] font-semibold">Loan #{loan.id.slice(0, 8)}</h4>
                  <p className="text-xs text-[#92A5A8]">Active since {new Date(loan.startTime * 1000).toLocaleDateString()}</p>
                </div>
              </div>
              {/* Loan NFT Representation */}
              <div className="bg-gradient-to-br from-[#33C5E020] to-[#33C5E005] border border-[#33C5E03D] rounded-xl p-2 flex items-center gap-2">
                <div className="w-10 h-10 bg-[#33C5E014] rounded-lg flex items-center justify-center text-[#33C5E0] text-[10px] font-bold">
                  NFT
                </div>
                <div className="text-[10px] text-[#33C5E0] font-medium leading-none">
                  Minted<br/>#{loan.id.slice(-4)}
                </div>
              </div>
              <div className={`px-3 py-1 rounded-full border text-xs font-medium ${getHealthBg(loan.healthFactor)} ${getHealthColor(loan.healthFactor)}`}>
                HF: {loan.healthFactor.toFixed(2)}
              </div>
            </div>

            <div className="grid grid-cols-2 gap-y-4 mb-6">
              <div>
                <p className="text-[#92A5A8] text-xs uppercase tracking-wider mb-1">Borrowed</p>
                <p className="text-[#FCFFFF] font-medium">{loan.borrowedAmount} XLM</p>
              </div>
              <div>
                <p className="text-[#92A5A8] text-xs uppercase tracking-wider mb-1">Collateral</p>
                <p className="text-[#FCFFFF] font-medium">{loan.collateralAmount} {loan.collateralAsset}</p>
              </div>
              <div>
                <p className="text-[#92A5A8] text-xs uppercase tracking-wider mb-1">Interest</p>
                <p className="text-[#FCFFFF] font-medium">{loan.interestRate}% APR</p>
              </div>
              <div>
                <p className="text-[#92A5A8] text-xs uppercase tracking-wider mb-1">Repayment</p>
                <p className="text-[#FCFFFF] font-medium">{loan.repaymentAmount} XLM</p>
              </div>
            </div>

            <div className="flex gap-x-3">
              <button 
                onClick={() => setSelectedLoan(loan)}
                className="flex-1 bg-[#33C5E0] text-[#161E22] py-3 rounded-xl font-semibold hover:bg-[#2bb2cc] transition-colors flex items-center justify-center gap-x-2"
              >
                Repay Loan
              </button>
              <button className="px-4 py-3 bg-[#1C252A] text-[#92A5A8] rounded-xl hover:text-[#FCFFFF] transition-colors">
                Details
              </button>
            </div>
          </div>
        ))}
      </div>

      {selectedLoan && (
        <RepaymentModal 
          loan={selectedLoan} 
          onClose={() => setSelectedLoan(null)} 
          onSuccess={() => {
            setSelectedLoan(null);
            onRefresh();
          }}
        />
      )}
    </div>
  );
}
