"use client";

import React from "react";
import { TrendingUp, Users, PieChart, Wallet, Info } from "lucide-react";
import { PoolState } from "@/app/lib/api/lending";

interface Props {
  poolState: PoolState;
  userShares: string;
  userEarnings: string;
}

export default function LendingStats({ poolState, userShares, userEarnings }: Props) {
  return (
    <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
      {/* Pool APY */}
      <div className="bg-[#182024] p-6 rounded-3xl border border-[#1C252A] relative overflow-hidden group">
        <div className="absolute top-0 right-0 p-4 opacity-5 group-hover:opacity-10 transition-opacity">
          <TrendingUp size={80} className="text-[#33C5E0]" />
        </div>
        <div className="flex items-center gap-2 text-[#92A5A8] mb-4">
          <TrendingUp size={16} className="text-[#33C5E0]" />
          <span className="text-xs font-bold uppercase tracking-wider">Current APY</span>
        </div>
        <div className="text-4xl font-black text-[#FCFFFF] mb-1">
          {poolState.current_apy.toFixed(2)}%
        </div>
        <div className="flex items-center gap-1 text-[10px] text-green-500 font-bold">
          <span className="bg-green-500/10 px-1.5 py-0.5 rounded">↑ 0.5% today</span>
        </div>
      </div>

      {/* Total Liquidity */}
      <div className="bg-[#182024] p-6 rounded-3xl border border-[#1C252A] relative overflow-hidden group">
        <div className="absolute top-0 right-0 p-4 opacity-5 group-hover:opacity-10 transition-opacity">
          <PieChart size={80} className="text-[#8B5CF6]" />
        </div>
        <div className="flex items-center gap-2 text-[#92A5A8] mb-4">
          <Users size={16} className="text-[#8B5CF6]" />
          <span className="text-xs font-bold uppercase tracking-wider">Total Deposits</span>
        </div>
        <div className="text-3xl font-black text-[#FCFFFF] mb-1">
          ${Number(poolState.total_deposits).toLocaleString()}
        </div>
        <div className="text-[10px] text-[#92A5A8]">
          Utilization: <span className="text-[#FCFFFF] font-bold">{poolState.utilization_rate}%</span>
        </div>
      </div>

      {/* User Balance */}
      <div className="bg-[#182024] p-6 rounded-3xl border border-[#1C252A] relative overflow-hidden group">
        <div className="absolute top-0 right-0 p-4 opacity-5 group-hover:opacity-10 transition-opacity">
          <Wallet size={80} className="text-[#33C5E0]" />
        </div>
        <div className="flex items-center gap-2 text-[#92A5A8] mb-4">
          <Wallet size={16} className="text-[#33C5E0]" />
          <span className="text-xs font-bold uppercase tracking-wider">Your Position</span>
        </div>
        <div className="text-3xl font-black text-[#FCFFFF] mb-1">
          {Number(userShares).toLocaleString()} <span className="text-sm font-normal text-[#92A5A8]">Shares</span>
        </div>
        <div className="flex items-center gap-1 text-[10px] text-[#92A5A8]">
          <Info size={10} />
          <span>Approx. value in USDC</span>
        </div>
      </div>

      {/* User Earnings */}
      <div className="bg-[#182024] p-6 rounded-3xl border border-[#1C252A] relative overflow-hidden group border-b-4 border-b-[#33C5E0]/30">
        <div className="absolute top-0 right-0 p-4 opacity-5 group-hover:opacity-10 transition-opacity">
          <TrendingUp size={80} className="text-green-500" />
        </div>
        <div className="flex items-center gap-2 text-[#92A5A8] mb-4">
          <TrendingUp size={16} className="text-green-500" />
          <span className="text-xs font-bold uppercase tracking-wider">Total Earnings</span>
        </div>
        <div className="text-3xl font-black text-green-500 mb-1">
          +${Number(userEarnings).toLocaleString()}
        </div>
        <div className="text-[10px] text-[#92A5A8]">
          Auto-compounding daily
        </div>
      </div>
    </div>
  );
}
