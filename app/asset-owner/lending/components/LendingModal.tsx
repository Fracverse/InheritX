"use client";

import React, { useState } from "react";
import { X, ArrowDownCircle, ArrowUpCircle, Info, Loader2 } from "lucide-react";

interface Props {
  type: 'deposit' | 'withdraw';
  onClose: () => void;
  onConfirm: (amount: string) => Promise<void>;
  balance: string;
  symbol: string;
}

export default function LendingModal({ type, onClose, onConfirm, balance, symbol }: Props) {
  const [amount, setAmount] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isDeposit = type === 'deposit';

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!amount || Number(amount) <= 0) return;
    
    setLoading(true);
    setError(null);
    try {
      await onConfirm(amount);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Transaction failed");
    } finally {
      setLoading(false);
    }
  };

  const setMax = () => {
    setAmount(balance);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
      <div className="bg-[#161E22] w-full max-w-md rounded-[32px] border border-[#2A3338] shadow-2xl overflow-hidden">
        <div className="p-6 border-b border-[#2A3338] flex justify-between items-center bg-[#182024]">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-xl ${isDeposit ? 'bg-[#33C5E0]/10 text-[#33C5E0]' : 'bg-[#8B5CF6]/10 text-[#8B5CF6]'}`}>
              {isDeposit ? <ArrowDownCircle size={20} /> : <ArrowUpCircle size={20} />}
            </div>
            <h2 className="text-xl font-bold text-[#FCFFFF]">
              {isDeposit ? 'Deposit' : 'Withdraw'} {symbol}
            </h2>
          </div>
          <button onClick={onClose} className="p-2 text-[#92A5A8] hover:text-[#FCFFFF] transition-colors">
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-8 space-y-6">
          <div className="space-y-4">
            <div className="flex justify-between text-xs font-bold text-[#92A5A8] uppercase tracking-wider">
              <span>Amount</span>
              <span className="flex items-center gap-2">
                Available: {Number(balance).toLocaleString()} {symbol}
                <button 
                  type="button" 
                  onClick={setMax}
                  className="text-[#33C5E0] hover:underline"
                >
                  MAX
                </button>
              </span>
            </div>
            
            <div className="relative">
              <input
                type="number"
                step="any"
                required
                className="w-full bg-[#1C252A] border border-[#2A3338] rounded-2xl px-6 py-5 text-2xl font-black text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-all"
                placeholder="0.00"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
              <div className="absolute right-6 top-1/2 -translate-y-1/2 font-bold text-[#92A5A8]">
                {symbol}
              </div>
            </div>

            {error && (
              <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-xs text-red-500 text-center">
                {error}
              </div>
            )}

            <div className="bg-[#1C252A] p-4 rounded-2xl border border-[#2A3338] space-y-2">
              <div className="flex justify-between text-xs">
                <span className="text-[#92A5A8]">Transaction Fee</span>
                <span className="text-[#FCFFFF] font-medium">~0.0001 XLM</span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-[#92A5A8]">Pool Health impact</span>
                <span className="text-green-500 font-medium">None</span>
              </div>
              <div className="pt-2 border-t border-[#2A3338] flex justify-between text-xs font-bold">
                <span className="text-[#92A5A8]">Expected {isDeposit ? 'Shares' : 'USDC'}</span>
                <span className="text-[#33C5E0]">{amount || "0.00"}</span>
              </div>
            </div>
          </div>

          <button
            type="submit"
            disabled={loading || !amount || Number(amount) <= 0}
            className={`w-full py-5 rounded-2xl font-black text-lg transition-all flex items-center justify-center gap-2 ${
              isDeposit 
                ? 'bg-[#33C5E0] text-[#161E22] hover:bg-[#2AB8D3] shadow-[0_0_20px_rgba(51,197,224,0.3)]' 
                : 'bg-transparent border-2 border-[#8B5CF6] text-[#8B5CF6] hover:bg-[#8B5CF6]/10'
            } disabled:opacity-50 disabled:cursor-not-allowed`}
          >
            {loading ? <Loader2 className="animate-spin" /> : (isDeposit ? 'CONFIRM DEPOSIT' : 'CONFIRM WITHDRAWAL')}
          </button>

          <div className="flex items-start gap-2 text-[10px] text-[#92A5A8] leading-tight">
            <Info size={12} className="flex-shrink-0" />
            <p>
              Your {isDeposit ? 'USDC will be converted to lending shares' : 'shares will be redeemed for USDC'} at the current pool exchange rate.
              {isDeposit && " Deposits contribute to pool liquidity and earn interest."}
            </p>
          </div>
        </form>
      </div>
    </div>
  );
}
