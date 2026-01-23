"use client";

import React, { useState } from "react";
import { ChevronDown, Repeat, Clock, ArrowUpRight, Info } from "lucide-react";
import Image from "next/image";
import SwapRateSlippage from "../components/SwapRateSlippage";
import RecentTransactions from "../components/RecentTransactions";

export default function SwapPage() {
  const [swapFrom, setSwapFrom] = useState("ETH");
  const [swapTo, setSwapTo] = useState("USDC");
  const [fromAmount, setFromAmount] = useState("");
  const [toAmount, setToAmount] = useState("");

  const handleSwapSelection = () => {
    setSwapFrom(swapTo);
    setSwapTo(swapFrom);
  };

  return (
    <div className="max-w-6xl mx-auto space-y-8 animate-fade-in">
      {/* Page Title */}
      <div className="flex justify-between items-start">
        <div>
          <h1 className="text-3xl font-bold text-[#FCFFFF] mb-2">Swap</h1>
          <p className="text-[#92A5A8]">
            Seamlessly swap your assets at the best available rate
          </p>
        </div>
        <div className="hidden md:flex flex-col items-center gap-1 group cursor-pointer">
          <span className="text-[10px] text-[#92A5A8] uppercase tracking-wider group-hover:text-[#33C5E0] transition-colors">
            History
          </span>
          <div className="p-2 bg-[#182024] border border-[#2A3338] rounded-full group-hover:border-[#33C5E0] transition-all">
            <Clock
              size={20}
              className="text-[#92A5A8] group-hover:text-[#33C5E0]"
            />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 items-start">
        <div className="space-y-6">
          <div className="relative space-y-4">
           
            <div className="bg-[#182024] border border-[#2A3338] rounded-2xl p-6 transition-all hover:border-[#33C5E01A]">
              <div className="flex justify-between items-center mb-4">
                <span className="text-sm font-medium text-[#92A5A8]">
                  Swap From:
                </span>
                <div className="flex items-center gap-2">
                  <span className="text-sm text-[#92A5A8]">Bal: 0</span>
                  <button className="text-[10px] font-bold text-[#33C5E0] bg-[#33C5E01A] px-2 py-0.5 rounded uppercase hover:bg-[#33C5E033] transition-colors">
                    Max
                  </button>
                </div>
              </div>

              <div className="flex justify-between items-end">
                <button className="flex items-center gap-2 bg-[#1C252A] border border-[#2A3338] px-4 py-2 rounded-full hover:border-[#33C5E0] transition-all group">
                  <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center overflow-hidden">
                    <span className="text-[10px] font-bold">{swapFrom[0]}</span>
                  </div>
                  <span className="font-bold text-[#FCFFFF] uppercase">
                    {swapFrom}
                  </span>
                  <ChevronDown
                    size={16}
                    className="text-[#92A5A8] group-hover:text-[#33C5E0] transition-colors"
                  />
                </button>

                <div className="text-right">
                  <input
                    type="text"
                    placeholder="$ 0.00"
                    className="bg-transparent text-3xl font-bold text-[#FCFFFF] outline-none text-right w-full placeholder:text-[#2A3338]"
                    value={fromAmount}
                    onChange={(e) => setFromAmount(e.target.value)}
                  />
                  <div className="mt-1 text-sm text-[#92A5A8]">≈ $0.00</div>
                </div>
              </div>
            </div>

         
            <div className="absolute left-1/2 -translate-x-1/2 -translate-y-1/2 z-10">
              <button
                onClick={handleSwapSelection}
                className="p-3 bg-[#1C252A] border border-[#2A3338] rounded-xl hover:border-[#33C5E0] hover:scale-110 transition-all text-[#92A5A8] hover:text-[#33C5E0] shadow-2xl"
              >
                <Repeat size={20} className="rotate-90" />
              </button>
            </div>

            <div className="bg-[#182024] border border-[#2A3338] rounded-2xl p-6 transition-all hover:border-[#33C5E01A]">
              <div className="flex justify-between items-center mb-4">
                <span className="text-sm font-medium text-[#92A5A8]">
                  Swap To:
                </span>
                <span className="text-sm text-[#92A5A8]">Bal: 0</span>
              </div>

              <div className="flex justify-between items-end">
                <button className="flex items-center gap-2 bg-[#1C252A] border border-[#2A3338] px-4 py-2 rounded-full hover:border-[#33C5E0] transition-all group">
                  <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center overflow-hidden">
                    <span className="text-[10px] font-bold">{swapTo[0]}</span>
                  </div>
                  <span className="font-bold text-[#FCFFFF] uppercase">
                    {swapTo}
                  </span>
                  <ChevronDown
                    size={16}
                    className="text-[#92A5A8] group-hover:text-[#33C5E0] transition-colors"
                  />
                </button>

                <div className="text-right">
                  <input
                    type="text"
                    placeholder="$ 0.00"
                    className="bg-transparent text-3xl font-bold text-[#FCFFFF] outline-none text-right w-full placeholder:text-[#2A3338]"
                    value={toAmount}
                    onChange={(e) => setToAmount(e.target.value)}
                  />
                  <div className="mt-1 text-sm text-[#92A5A8]">≈ $0.00</div>
                </div>
              </div>
            </div>
          </div>

          <div className="flex justify-between items-center px-2">
            <div className="flex items-center gap-2 text-xs text-[#92A5A8]">
              <Info size={14} />
              <span>Exchange rate updated every 30s</span>
            </div>
            <div className="text-sm font-medium text-[#92A5A8]">
              Gas Fee: <span className="text-[#FCFFFF]">$0.00</span>
            </div>
          </div>

          <button className="w-full bg-[#33C5E0]/10 border border-[#33C5E0]/30 py-4 rounded-2xl flex items-center justify-center gap-2 text-[#33C5E0] font-bold uppercase tracking-widest hover:bg-[#33C5E0] hover:text-[#161E22] transition-all group shadow-[0_0_20px_rgba(51,197,224,0.1)]">
            <Repeat
              size={20}
              className="group-hover:rotate-180 transition-transform duration-500"
            />
            Swap Asset
          </button>
        </div>

 
        <div className="lg:pt-0">
          <SwapRateSlippage />
        </div>
      </div>

   
      <RecentTransactions />
    </div>
  );
}
