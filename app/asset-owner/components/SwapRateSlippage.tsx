import React from "react";
import Image from "next/image";
import { TrendingUp, TrendingDown } from "lucide-react";

const AssetRateItem = ({
  asset,
  value,
  change,
  isUp,
  icon,
}: {
  asset: string;
  value: string;
  change: string;
  isUp: boolean;
  icon: string;
}) => (
  <div className="bg-[#182024] border border-[#2A3338] rounded-xl p-4 flex flex-col gap-3 min-w-[160px]">
    <div className="flex justify-between items-center">
      <div className="flex items-center gap-2">
        <div className="w-6 h-6 rounded-full overflow-hidden bg-white/10 flex items-center justify-center">
          {/* Simple placeholder for icons, real ones would use Image from next */}
          <span className="text-[10px] font-bold">{asset[0]}</span>
        </div>
        <span className="text-sm font-bold text-[#FCFFFF] uppercase">
          {asset}
        </span>
      </div>
      <div
        className={`flex items-center gap-1 text-xs ${isUp ? "text-green-400" : "text-red-400"}`}
      >
        {isUp ? <TrendingUp size={12} /> : <TrendingDown size={12} />}
        {change}
      </div>
    </div>
    <div className="text-sm text-[#92A5A8]">
      1 {asset} = {value}
    </div>
  </div>
);

const SwapRateSlippage = () => {
  const assets = [
    { asset: "XLM", value: "0.12 USDC", change: "0.24%", isUp: true, icon: "" },
    { asset: "USDC", value: "8.33 XLM", change: "0.15%", isUp: true, icon: "" },
    {
      asset: "AQUA",
      value: "0.004 USDC",
      change: "1.12%",
      isUp: false,
      icon: "",
    },
  ];

  return (
    <div className="w-full flex flex-col gap-6">
      <div className="w-full bg-[#1A2329] border border-[#2A3338] rounded-xl p-3 text-center">
        <span className="text-xs font-bold text-[#33C5E0] uppercase tracking-wider">
          Asset Rate Slippage
        </span>
      </div>

      <div className="flex gap-4 overflow-x-auto pb-2 scrollbar-hide">
        {assets.map((item, index) => (
          <AssetRateItem key={index} {...item} />
        ))}
      </div>

      <div className="text-center">
        <p className="text-xs text-[#33C5E0] leading-relaxed">
          You Will Receive At Least 1790 USDC (If The Price Doesn&apos;t Move
          More Than 0.5%).
        </p>
      </div>
    </div>
  );
};

export default SwapRateSlippage;
