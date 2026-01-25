"use client";

import React from "react";
import { ArrowLeft } from "lucide-react";

interface PlanSummaryData {
  planName: string;
  description: string;
  beneficiary: string;
  beneficiaryEmail: string;
  walletAddress: string;
  executeOn: string;
  assets: string[];
}

interface PlanSummaryProps {
  data: PlanSummaryData;
  onBack: () => void;
  onWithdraw?: () => void;
}

export default function PlanSummary({ data, onBack, onWithdraw }: PlanSummaryProps) {
  const expandableSections = ["Assets", "Rules & Conditions", "Legal Settings", "Notes"];

  return (
    <div className="max-w-4xl mx-auto animate-fade-in w-full">
      {/* Header - Mobile Stacked, Desktop Side by Side */}
      <div className="mb-4 md:mb-6 flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div className="flex items-center gap-3 md:gap-4">
          <button
            onClick={onBack}
            className="w-10 h-10 rounded-full bg-[#33C5E0] flex items-center justify-center text-[#161E22] hover:bg-[#2AB8D3] transition-colors flex-shrink-0"
          >
            <ArrowLeft size={20} />
          </button>
          <h1 className="text-xl md:text-3xl font-semibold text-[#FCFFFF]">Claim Plan</h1>
        </div>
        <button
          onClick={onWithdraw}
          className="w-full md:w-auto px-6 py-3 border border-[#33C5E0] text-[#33C5E0] rounded-lg hover:bg-[#33C5E0]/10 transition-colors font-medium text-sm md:text-base"
        >
          WITHDRAW
        </button>
      </div>

      <p className="text-xs md:text-sm text-[#92A5A8] mb-6 md:mb-8">
        To transfer inheritance to your wallet, click on the 'Withdraw' button.
      </p>

      <div className="bg-[#1C252A] rounded-2xl p-4 md:p-8">
        <h2 className="text-lg md:text-xl font-semibold text-[#FCFFFF] mb-4 md:mb-6">
          Plan Summary
        </h2>

        <div className="space-y-3 md:space-y-4">
          <div className="flex flex-col gap-2 pb-3 md:pb-4 border-b border-[#2A3338]">
            <span className="text-xs md:text-sm text-[#92A5A8] uppercase">PLAN NAME</span>
            <span className="text-sm md:text-base text-[#FCFFFF] font-medium">{data.planName}</span>
          </div>

          <div className="flex flex-col gap-2 pb-3 md:pb-4 border-b border-[#2A3338]">
            <span className="text-xs md:text-sm text-[#92A5A8] uppercase">DESCRIPTION</span>
            <span className="text-sm md:text-base text-[#FCFFFF] font-medium">
              {data.description}
            </span>
          </div>

          <div className="flex flex-col gap-2 pb-3 md:pb-4 border-b border-[#2A3338]">
            <span className="text-xs md:text-sm text-[#92A5A8] uppercase">BENEFICIARY</span>
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-full bg-[#33C5E0] flex items-center justify-center text-xs text-[#161E22] font-semibold flex-shrink-0">
                {data.beneficiary.charAt(0).toUpperCase()}
              </div>
              <a
                href="#"
                className="text-sm md:text-base text-[#33C5E0] hover:underline font-medium break-all"
              >
                {data.beneficiary}
              </a>
            </div>
          </div>

          <div className="flex flex-col gap-2 pb-3 md:pb-4 border-b border-[#2A3338]">
            <span className="text-xs md:text-sm text-[#92A5A8] uppercase">ASSETS</span>
            <span className="text-sm md:text-base text-[#FCFFFF] font-medium">
              {data.assets.join(", ")}
            </span>
          </div>

          <div className="flex flex-col gap-2 pb-3 md:pb-4 border-b border-[#2A3338]">
            <span className="text-xs md:text-sm text-[#92A5A8] uppercase">EMAIL</span>
            <a
              href={`mailto:${data.beneficiaryEmail}`}
              className="text-sm md:text-base text-[#33C5E0] hover:underline font-medium break-all"
            >
              {data.beneficiaryEmail}
            </a>
          </div>

          <div className="flex flex-col gap-2 pb-3 md:pb-4 border-b border-[#2A3338]">
            <span className="text-xs md:text-sm text-[#92A5A8] uppercase">WALLET ADDRESS</span>
            <span className="text-sm md:text-base text-[#FCFFFF] font-medium font-mono break-all">
              {data.walletAddress}
            </span>
          </div>

          <div className="flex flex-col gap-2">
            <span className="text-xs md:text-sm text-[#92A5A8] uppercase">EXECUTE ON</span>
            <span className="text-sm md:text-base text-[#FCFFFF] font-medium">{data.executeOn}</span>
          </div>
        </div>

        {/* Expandable Sections */}
        <div className="mt-6 md:mt-8 space-y-3 md:space-y-4">
          {expandableSections.map((section) => (
            <button
              key={section}
              className="w-full flex items-center justify-between py-3 md:py-4 px-4 bg-[#161E22] rounded-lg hover:bg-[#2A3338] transition-colors"
            >
              <span className="text-sm md:text-base text-[#FCFFFF] font-medium">{section}</span>
              <span className="text-[#92A5A8] text-lg">&gt;</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
