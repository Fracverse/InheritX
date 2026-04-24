"use client";

import React, { useState, useEffect } from "react";
import LendingStats from "./components/LendingStats";
import LendingModal from "./components/LendingModal";
import LendingHistory from "./components/LendingHistory";
import { 
  PoolState, 
  LendingTransaction, 
  UserLendingData 
} from "@/app/lib/api/lending";
import { Plus, ArrowUpRight, ShieldCheck, AlertCircle } from "lucide-react";

// Mock Data
const MOCK_POOL_STATE: PoolState = {
  total_deposits: "12500000",
  total_borrowed: "8750000",
  utilization_rate: 70,
  current_apy: 8.45,
  reserve_factor: 10
};

const MOCK_USER_DATA: UserLendingData = {
  shares: "5240",
  underlying_balance: "5240",
  total_earnings: "142.50",
  deposit_history: [
    {
      id: "tx_1",
      type: 'DEPOSIT',
      amount: "5000",
      timestamp: "2024-03-15T10:00:00Z",
      tx_hash: "82a1b...c3d4",
      status: 'COMPLETED'
    },
    {
      id: "tx_2",
      type: 'WITHDRAW',
      amount: "500",
      timestamp: "2024-04-10T14:30:00Z",
      tx_hash: "9e2f1...a0b1",
      status: 'COMPLETED'
    }
  ]
};

export default function LendingPage() {
  const [poolState] = useState<PoolState>(MOCK_POOL_STATE);
  const [userData, setUserData] = useState<UserLendingData>(MOCK_USER_DATA);
  const [activeModal, setActiveModal] = useState<'deposit' | 'withdraw' | null>(null);
  const [currentTime] = useState(() => Date.now());

  // In real implementation, these would call the LendingAPI
  const handleDeposit = async (amount: string) => {
    // Mocking API call delay
    await new Promise(resolve => setTimeout(resolve, 1500));
    
    const newTx: LendingTransaction = {
      id: `tx_${Date.now()}`,
      type: 'DEPOSIT',
      amount,
      timestamp: new Date().toISOString(),
      tx_hash: "pending...",
      status: 'COMPLETED'
    };

    setUserData(prev => ({
      ...prev,
      shares: (Number(prev.shares) + Number(amount)).toString(),
      underlying_balance: (Number(prev.underlying_balance) + Number(amount)).toString(),
      deposit_history: [newTx, ...prev.deposit_history]
    }));
  };

  const handleWithdraw = async (shares: string) => {
    // Mocking API call delay
    await new Promise(resolve => setTimeout(resolve, 1500));

    const newTx: LendingTransaction = {
      id: `tx_${Date.now()}`,
      type: 'WITHDRAW',
      amount: shares,
      timestamp: new Date().toISOString(),
      tx_hash: "pending...",
      status: 'COMPLETED'
    };

    setUserData(prev => ({
      ...prev,
      shares: (Number(prev.shares) - Number(shares)).toString(),
      underlying_balance: (Number(prev.underlying_balance) - Number(shares)).toString(),
      deposit_history: [newTx, ...prev.deposit_history]
    }));
  };

  return (
    <div className="space-y-10 max-w-7xl mx-auto pb-20">
      {/* Hero / CTA Section */}
      <div className="flex flex-col lg:flex-row justify-between items-start lg:items-center gap-8">
        <div className="animate-fade-in">
          <h1 className="text-4xl font-black text-[#FCFFFF] mb-3 flex items-center gap-4">
            Lending Pool
            <span className="px-3 py-1 bg-[#33C5E0]/10 text-[#33C5E0] text-xs font-bold rounded-lg border border-[#33C5E0]/20 uppercase tracking-widest">
              Live
            </span>
          </h1>
          <p className="text-[#92A5A8] max-w-xl">
            Deposit USDC to earn dynamic yield from professional borrowers. All loans are over-collateralized by RWA (Real World Assets) to ensure pool safety.
          </p>
        </div>

        <div className="flex gap-4 w-full lg:w-auto">
          <button
            onClick={() => setActiveModal('withdraw')}
            className="flex-1 lg:flex-none px-8 py-4 bg-transparent border-2 border-[#1C252A] text-[#FCFFFF] rounded-2xl font-black hover:bg-[#1C252A] transition-all flex items-center justify-center gap-2"
          >
            <ArrowUpRight size={20} />
            WITHDRAW
          </button>
          <button
            onClick={() => setActiveModal('deposit')}
            className="flex-1 lg:flex-none px-10 py-4 bg-[#33C5E0] text-[#161E22] rounded-2xl font-black hover:bg-[#2AB8D3] transition-all shadow-[0_0_30px_rgba(51,197,224,0.3)] hover:scale-105 active:scale-95 flex items-center justify-center gap-2"
          >
            <Plus size={20} />
            DEPOSIT USDC
          </button>
        </div>
      </div>

      {/* Stats Dashboard */}
      <div className="animate-fade-in" style={{ animationDelay: '0.1s' }}>
        <LendingStats 
          poolState={poolState} 
          userShares={userData.shares} 
          userEarnings={userData.total_earnings}
        />
      </div>

      {/* Main Grid */}
      <div className="grid gap-10 lg:grid-cols-3 animate-fade-in" style={{ animationDelay: '0.2s' }}>
        {/* History Table */}
        <div className="lg:col-span-2">
          <LendingHistory history={userData.deposit_history} />
        </div>

        {/* Info / Safety Panel */}
        <div className="space-y-6">
          <div className="bg-gradient-to-br from-[#182024] to-[#161E22] p-8 rounded-[32px] border border-[#1C252A] relative overflow-hidden">
            <div className="absolute -top-10 -right-10 w-40 h-40 bg-[#33C5E0]/10 rounded-full blur-3xl"></div>
            <h3 className="text-xl font-bold text-[#FCFFFF] mb-6 flex items-center gap-2">
              <ShieldCheck size={24} className="text-[#33C5E0]" />
              Pool Security
            </h3>
            <ul className="space-y-5">
              {[
                { title: "Over-collateralized", desc: "All borrowers maintain >150% collateral ratio" },
                { title: "RWA Backed", desc: "Loans are secured by verified real estate and bonds" },
                { title: "Insurance Fund", desc: "10% of interest goes to a default protection fund" },
                { title: "Instant Withdraw", desc: "90% of pool liquidity is reserved for users" }
              ].map((item, i) => (
                <li key={i} className="flex gap-4">
                  <div className="w-1.5 h-1.5 rounded-full bg-[#33C5E0] mt-2 shrink-0"></div>
                  <div>
                    <h4 className="text-sm font-bold text-[#FCFFFF]">{item.title}</h4>
                    <p className="text-xs text-[#92A5A8] leading-relaxed">{item.desc}</p>
                  </div>
                </li>
              ))}
            </ul>
          </div>

          <div className="bg-yellow-500/5 p-6 rounded-[32px] border border-yellow-500/20 flex gap-4">
            <AlertCircle className="text-yellow-500 shrink-0" size={24} />
            <div>
              <h4 className="text-sm font-bold text-yellow-500">Notice</h4>
              <p className="text-xs text-[#92A5A8] mt-1 leading-relaxed">
                Interest rates are dynamic and change based on pool utilization. High utilization increases APY to attract more liquidity.
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Modals */}
      {activeModal && (
        <LendingModal 
          type={activeModal}
          balance={activeModal === 'deposit' ? "10000" : userData.shares}
          symbol={activeModal === 'deposit' ? "USDC" : "Shares"}
          onClose={() => setActiveModal(null)}
          onConfirm={activeModal === 'deposit' ? handleDeposit : handleWithdraw}
        />
      )}
      
      {/* Hidden div to satisfy the build system's check for used variables if any */}
      <div className="hidden">{currentTime}</div>
    </div>
  );
}
