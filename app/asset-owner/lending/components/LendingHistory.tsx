"use client";

import React from "react";
import { History, ExternalLink, ArrowDownRight, ArrowUpRight } from "lucide-react";
import { LendingTransaction } from "@/app/lib/api/lending";

interface Props {
  history: LendingTransaction[];
}

export default function LendingHistory({ history }: Props) {
  return (
    <div className="bg-[#182024] rounded-[32px] border border-[#1C252A] overflow-hidden">
      <div className="p-8 border-b border-[#1C252A] flex justify-between items-center">
        <h2 className="text-xl font-bold text-[#FCFFFF] flex items-center gap-3">
          <History size={20} className="text-[#33C5E0]" />
          Transaction History
        </h2>
        <button className="text-xs text-[#33C5E0] hover:underline uppercase font-bold tracking-widest">
          View All
        </button>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left">
          <thead>
            <tr className="text-[#92A5A8] text-[10px] uppercase font-bold tracking-widest border-b border-[#1C252A]">
              <th className="px-8 py-4">Action</th>
              <th className="px-8 py-4">Amount</th>
              <th className="px-8 py-4">Date</th>
              <th className="px-8 py-4">Status</th>
              <th className="px-8 py-4">Transaction</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-[#1C252A]">
            {history.map((tx) => (
              <tr key={tx.id} className="hover:bg-[#1C252A]/30 transition-colors group">
                <td className="px-8 py-5">
                  <div className="flex items-center gap-3">
                    <div className={`p-2 rounded-lg ${tx.type === 'DEPOSIT' ? 'bg-green-500/10 text-green-500' : 'bg-red-500/10 text-red-500'}`}>
                      {tx.type === 'DEPOSIT' ? <ArrowDownRight size={14} /> : <ArrowUpRight size={14} />}
                    </div>
                    <span className="text-sm font-bold text-[#FCFFFF] uppercase">{tx.type}</span>
                  </div>
                </td>
                <td className="px-8 py-5">
                  <span className={`text-sm font-black ${tx.type === 'DEPOSIT' ? 'text-green-500' : 'text-red-500'}`}>
                    {tx.type === 'DEPOSIT' ? '+' : '-'}{Number(tx.amount).toLocaleString()} USDC
                  </span>
                </td>
                <td className="px-8 py-5">
                  <div className="text-sm text-[#FCFFFF]">{new Date(tx.timestamp).toLocaleDateString()}</div>
                  <div className="text-[10px] text-[#92A5A8]">{new Date(tx.timestamp).toLocaleTimeString()}</div>
                </td>
                <td className="px-8 py-5">
                  <span className={`px-2 py-1 rounded text-[10px] font-bold uppercase ${
                    tx.status === 'COMPLETED' ? 'bg-green-500/10 text-green-500' :
                    tx.status === 'FAILED' ? 'bg-red-500/10 text-red-500' :
                    'bg-yellow-500/10 text-yellow-500'
                  }`}>
                    {tx.status}
                  </span>
                </td>
                <td className="px-8 py-5">
                  <a 
                    href={`https://stellar.expert/explorer/public/tx/${tx.tx_hash}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-2 text-xs text-[#92A5A8] hover:text-[#33C5E0] transition-colors"
                  >
                    {tx.tx_hash.slice(0, 8)}...
                    <ExternalLink size={12} />
                  </a>
                </td>
              </tr>
            ))}

            {history.length === 0 && (
              <tr>
                <td colSpan={5} className="px-8 py-12 text-center text-[#92A5A8]">
                  No lending activity found.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
