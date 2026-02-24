"use client";

import React, { useState } from "react";
import { ChevronDown, Repeat, Clock, Info, Search, X } from "lucide-react";
import SwapRateSlippage from "../components/SwapRateSlippage";
import RecentTransactions from "../components/RecentTransactions";
import tokensData from "./tokens.json";

interface Token {
  id: string;
  symbol: string;
  name: string;
  icon: string;
}

export default function SwapPage() {
  const [tokens] = useState<Token[]>(tokensData);
  const [swapFrom, setSwapFrom] = useState<Token>(tokens[0]); 
  const [swapTo, setSwapTo] = useState<Token>(tokens[1]); 
  const [fromAmount, setFromAmount] = useState("");
  const [toAmount, setToAmount] = useState("");

  const [isTokenSelectorOpen, setIsTokenSelectorOpen] = useState(false);
  const [selectingFor, setSelectingFor] = useState<"from" | "to">("from");
  const [searchQuery, setSearchQuery] = useState("");

  const handleSwapSelection = () => {
    const temp = swapFrom;
    setSwapFrom(swapTo);
    setSwapTo(temp);
  };

  const openTokenSelector = (type: "from" | "to") => {
    setSelectingFor(type);
    setIsTokenSelectorOpen(true);
    setSearchQuery("");
  };

  const selectToken = (token: Token) => {
    if (selectingFor === "from") {
      if (token.id === swapTo.id) {
        setSwapTo(swapFrom);
      }
      setSwapFrom(token);
    } else {
      if (token.id === swapFrom.id) {
        setSwapFrom(swapTo);
      }
      setSwapTo(token);
    }
    setIsTokenSelectorOpen(false);
  };

  const filteredTokens = tokens.filter(
    (t) =>
      t.symbol.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.name.toLowerCase().includes(searchQuery.toLowerCase()),
  );

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
            {/* Swap From */}
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
                <button
                  onClick={() => openTokenSelector("from")}
                  className="flex items-center gap-2 bg-[#1C252A] border border-[#2A3338] px-4 py-2 rounded-full hover:border-[#33C5E0] transition-all group"
                >
                  <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center overflow-hidden relative">
                    <span className="text-[10px] font-bold z-0">
                      {swapFrom.symbol[0]}
                    </span>
                    {/* Fallback to text if icon not found, though here we assume relative path works or Alt text handles it */}
                  </div>
                  <span className="font-bold text-[#FCFFFF] uppercase">
                    {swapFrom.symbol}
                  </span>
                  <ChevronDown
                    size={16}
                    className="text-[#92A5A8] group-hover:text-[#33C5E0] transition-colors"
                  />
                </button>

                <div className="text-right">
                  <input
                    type="text"
                    placeholder="0.00"
                    className="bg-transparent text-3xl font-bold text-[#FCFFFF] outline-none text-right w-full placeholder:text-[#2A3338]"
                    value={fromAmount}
                    onChange={(e) => setFromAmount(e.target.value)}
                  />
                  <div className="mt-1 text-sm text-[#92A5A8]">≈ $0.00</div>
                </div>
              </div>
            </div>

            {/* Swap Arrow */}
            <div className="absolute left-1/2 -translate-x-1/2 -translate-y-1/2 z-10">
              <button
                onClick={handleSwapSelection}
                className="p-3 bg-[#1C252A] border border-[#2A3338] rounded-xl hover:border-[#33C5E0] hover:scale-110 transition-all text-[#92A5A8] hover:text-[#33C5E0] shadow-2xl"
              >
                <Repeat size={20} className="rotate-90" />
              </button>
            </div>

            {/* Swap To */}
            <div className="bg-[#182024] border border-[#2A3338] rounded-2xl p-6 transition-all hover:border-[#33C5E01A]">
              <div className="flex justify-between items-center mb-4">
                <span className="text-sm font-medium text-[#92A5A8]">
                  Swap To:
                </span>
                <span className="text-sm text-[#92A5A8]">Bal: 0</span>
              </div>

              <div className="flex justify-between items-end">
                <button
                  onClick={() => openTokenSelector("to")}
                  className="flex items-center gap-2 bg-[#1C252A] border border-[#2A3338] px-4 py-2 rounded-full hover:border-[#33C5E0] transition-all group"
                >
                  <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center overflow-hidden relative">
                    <span className="text-[10px] font-bold z-0">
                      {swapTo.symbol[0]}
                    </span>
                  </div>
                  <span className="font-bold text-[#FCFFFF] uppercase">
                    {swapTo.symbol}
                  </span>
                  <ChevronDown
                    size={16}
                    className="text-[#92A5A8] group-hover:text-[#33C5E0] transition-colors"
                  />
                </button>

                <div className="text-right">
                  <input
                    type="text"
                    placeholder="0.00"
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

      {isTokenSelectorOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
          <div className="bg-[#161E22] border border-[#2A3338] w-full max-w-md rounded-3xl overflow-hidden shadow-2xl animate-scale-up">
            <div className="p-6 border-b border-[#2A3338] flex justify-between items-center">
              <h2 className="text-xl font-bold text-[#FCFFFF]">Select Token</h2>
              <button
                onClick={() => setIsTokenSelectorOpen(false)}
                className="p-2 hover:bg-[#1C252A] rounded-full text-[#92A5A8] hover:text-[#FCFFFF] transition-all"
              >
                <X size={20} />
              </button>
            </div>

            <div className="p-4">
              <div className="relative mb-6">
                <Search
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-[#92A5A8]"
                  size={18}
                />
                <input
                  type="text"
                  placeholder="Search by name or symbol"
                  className="w-full bg-[#182024] border border-[#2A3338] rounded-xl py-3 pl-10 pr-4 text-[#FCFFFF] outline-none focus:border-[#33C5E0] transition-all"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                />
              </div>

              <div className="space-y-1 max-h-[400px] overflow-y-auto custom-scrollbar">
                {filteredTokens.length > 0 ? (
                  filteredTokens.map((token) => (
                    <button
                      key={token.id}
                      onClick={() => selectToken(token)}
                      className="w-full flex items-center justify-between p-4 rounded-2xl hover:bg-[#1C252A] transition-all group"
                    >
                      <div className="flex items-center gap-4">
                        <div className="w-10 h-10 rounded-full bg-[#182024] border border-[#2A3338] flex items-center justify-center font-bold text-[#FCFFFF] group-hover:border-[#33C5E0]">
                          {token.symbol[0]}
                        </div>
                        <div className="text-left">
                          <div className="font-bold text-[#FCFFFF] group-hover:text-[#33C5E0] transition-colors">
                            {token.symbol}
                          </div>
                          <div className="text-xs text-[#92A5A8]">
                            {token.name}
                          </div>
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="text-sm font-medium text-[#FCFFFF]">
                          0
                        </div>
                        <div className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
                          Balance
                        </div>
                      </div>
                    </button>
                  ))
                ) : (
                  <div className="py-10 text-center text-[#92A5A8]">
                    No tokens found matching &quot;{searchQuery}&quot;
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
