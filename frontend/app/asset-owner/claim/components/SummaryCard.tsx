"use client";

import React from "react";
import { ArrowUpRight } from "lucide-react";

interface SummaryCardProps {
  count: number;
  label: string;
  buttonText: string;
  buttonTextMobile?: string;
  onClick?: () => void;
  disabled?: boolean;
}

export default function SummaryCard({
  count,
  label,
  buttonText,
  buttonTextMobile,
  onClick,
  disabled = false,
}: SummaryCardProps) {
  return (
    <div className="bg-[#1C252A] rounded-xl md:rounded-2xl p-3 md:p-6">
      <div className="text-3xl md:text-5xl font-semibold text-[#FCFFFF] mb-1 md:mb-2">
        {count}
      </div>
      <div className="text-xs md:text-sm text-[#92A5A8] mb-3 md:mb-4 leading-tight">
        {label}
      </div>
      <button
        onClick={onClick}
        disabled={disabled}
        className={`w-full py-4 md:py-5 rounded-3xl font-medium flex items-center justify-center gap-1 md:gap-1 transition-colors text-[10px] md:text-sm ${
          disabled
            ? "bg-[#33C5E014] borer border-[#33C5E03D] text-[#33C5E0] cursor-not-allowed"
            : "bg-[#33C5E0] text-[#161E22] hover:bg-[#2AB8D3]"
        }`}
      >
        <span className="hidden md:inline">{buttonText}</span>
        <span className="md:hidden">{buttonTextMobile || buttonText}</span>
        <ArrowUpRight size={12} className="md:w-4 md:h-4" />
      </button>
    </div>
  );
}
