"use client";

import React from "react";
import Image from "next/image";
import error from "@/public/error.svg";
interface ErrorModalProps {
  onCancel: () => void;
  onContinue: () => void;
}

export default function ErrorModal({ onCancel, onContinue }: ErrorModalProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-fade-in p-4">
      <div className="border border-[#2A3338] bg-[#161E22] rounded-3xl p-6 md:p-12 max-w-203 w-full animate-scale-in">
        <div className="text-center">
          <h2 className="text-[16px]  text-[#FCFFFF] mb-4 md:mb-6 px-2">
            Oops! Looks like something went wrong. Try again.
          </h2>
          <div className="flex justify-center mb-6 md:mb-8">
             <div className="flex justify-center py-4">
                                    <Image src={error} alt="error Icon" />
                                  </div>
          </div>
          <div className="flex flex-col sm:flex-row gap-3 md:gap-4 justify-center">
            <button
              onClick={onCancel}
              className="w-full py-4 px-6 bg-[#1C252A] border border-[#2A3338] text-[#FCFFFF] rounded-full hover:bg-[#2A3338] transition-colors text-sm md:text-base"
            >
              Cancel
            </button>
            <button
              onClick={onContinue}
              className="w-full py-4 px-6 bg-[#33C5E0] text-[#161E22] rounded-full hover:bg-[#2AB8D3] transition-colors font-medium text-sm md:text-base"
            >
              Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
