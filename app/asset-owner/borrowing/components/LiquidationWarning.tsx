"use client";

import React from "react";
import { AlertTriangle, Plus } from "lucide-react";
import { Loan } from "@/app/services/loanService";

interface LiquidationWarningProps {
  loan: Loan;
}

export default function LiquidationWarning({ loan }: LiquidationWarningProps) {
  return (
    <div className="bg-red-500/10 border border-red-500/30 rounded-2xl p-4 flex items-center justify-between">
      <div className="flex items-center gap-x-4">
        <div className="bg-red-500/20 p-2 rounded-full text-red-500">
          <AlertTriangle size={24} />
        </div>
        <div>
          <h3 className="text-red-500 font-semibold">Liquidation Risk!</h3>
          <p className="text-sm text-[#92A5A8]">
            Loan <span className="text-[#FCFFFF]">#{loan.id.slice(0, 8)}...</span> has a health factor of <span className="text-red-500 font-bold">{loan.healthFactor.toFixed(2)}</span>. 
            Liquidation threshold is 1.30.
          </p>
        </div>
      </div>
      <button className="bg-red-500 text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-x-2 hover:bg-red-600 transition-colors">
        <Plus size={16} />
        Add Collateral
      </button>
    </div>
  );
}
