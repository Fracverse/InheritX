"use client";

import React from "react";
import { Loan } from "@/app/services/loanService";
import { CheckCircle2, XCircle } from "lucide-react";

interface LoanHistoryProps {
  loans: Loan[];
}

export default function LoanHistory({ loans }: LoanHistoryProps) {
  if (loans.length === 0) {
    return (
      <div className="bg-[#182024] rounded-3xl p-12 border border-[#2A3338] text-center">
        <p className="text-[#92A5A8]">No loan history found.</p>
      </div>
    );
  }

  return (
    <div className="bg-[#182024] rounded-3xl border border-[#2A3338] overflow-hidden">
      <table className="w-full text-left">
        <thead>
          <tr className="border-b border-[#2A3338] text-[#92A5A8] text-xs uppercase tracking-wider">
            <th className="px-6 py-4 font-medium">Loan ID</th>
            <th className="px-6 py-4 font-medium">Date</th>
            <th className="px-6 py-4 font-medium">Amount</th>
            <th className="px-6 py-4 font-medium">Collateral</th>
            <th className="px-6 py-4 font-medium">Status</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-[#2A3338]">
          {loans.map((loan) => (
            <tr key={loan.id} className="text-[#FCFFFF] hover:bg-[#1C252A] transition-colors">
              <td className="px-6 py-4 text-sm font-medium">#{loan.id.slice(0, 8)}</td>
              <td className="px-6 py-4 text-sm text-[#92A5A8]">{new Date(loan.startTime * 1000).toLocaleDateString()}</td>
              <td className="px-6 py-4 text-sm font-bold">{loan.borrowedAmount} XLM</td>
              <td className="px-6 py-4 text-sm">{loan.collateralAmount} {loan.collateralAsset}</td>
              <td className="px-6 py-4">
                <div className={`inline-flex items-center gap-x-1 px-3 py-1 rounded-full text-[10px] font-bold uppercase ${loan.status === "REPAID" ? "bg-green-500/10 text-green-500" : "bg-red-500/10 text-red-500"}`}>
                  {loan.status === "REPAID" ? (
                    <>
                      <CheckCircle2 size={12} />
                      Repaid
                    </>
                  ) : (
                    <>
                      <XCircle size={12} />
                      Liquidated
                    </>
                  )}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
