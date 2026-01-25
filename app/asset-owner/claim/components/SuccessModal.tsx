"use client";

import React from "react";
import { Check } from "lucide-react";

interface SuccessModalProps {
  onCancel: () => void;
  onContinue: () => void;
}

export default function SuccessModal({ onCancel, onContinue }: SuccessModalProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-fade-in p-4">
      <div className="bg-[#1C252A] rounded-2xl p-6 md:p-12 max-w-md w-full animate-scale-in">
        <div className="text-center">
          <h2 className="text-xl md:text-2xl font-semibold text-[#FCFFFF] mb-4 md:mb-6">
            Inheritance claimed is Successful
          </h2>
          <div className="flex justify-center mb-6 md:mb-8">
            <div className="relative">
              <div className="w-20 h-20 md:w-32 md:h-32 rounded-full bg-[#4ADE80] flex items-center justify-center animate-scale-in">
                <Check size={40} className="md:w-12 md:h-12 text-[#161E22]" strokeWidth={3} />
              </div>
              <div className="absolute inset-0 rounded-full bg-[#4ADE80]/30 animate-pulse-glow" />
              <div
                className="absolute inset-0 rounded-full bg-[#4ADE80]/20 animate-pulse-glow"
                style={{ animationDelay: "0.5s" }}
              />
            </div>
          </div>
          <div className="flex flex-col sm:flex-row gap-3 md:gap-4 justify-center">
            <button
              onClick={onCancel}
              className="px-6 py-3 bg-[#1C252A] border border-[#2A3338] text-[#FCFFFF] rounded-lg hover:bg-[#2A3338] transition-colors text-sm md:text-base"
            >
              Cancel
            </button>
            <button
              onClick={onContinue}
              className="px-6 py-3 bg-[#33C5E0] text-[#161E22] rounded-lg hover:bg-[#2AB8D3] transition-colors font-medium text-sm md:text-base"
            >
              Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
