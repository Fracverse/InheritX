"use client";

import React from "react";
import { ArrowLeft } from "lucide-react";
import thumbPrint from "@/public/Rectangle 1046.svg";
import Image from "next/image";

interface EnableFingerprintModalProps {
  onClose: () => void;
  onContinue: () => void;
}

export default function EnableFingerprintModal({
  onClose,
  onContinue,
}: EnableFingerprintModalProps) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in"
      onClick={onClose}
    >
      <div
        className="overflow-y-scroll w-full md:w-[812px] relative   overflow-hidden bg-[#161E22] border border-[#1C252A] flex justify-center  rounded-3xl max-h-[90vh]  no-scrollbar"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="bg-[#161E22] overflow-y-scroll  rounded-3xl w-full no-scrollbar overflow-hidden animate-scale-up">
          {/* Header */}
          <div className="flex rounded-3xl  bg-[#161E22] absolute top-0 left-0 right-0 z-10 bg-gray-700s items-center gap-4 p-6 border-b border-[#1C252A]">
            <button
              onClick={onClose}
              className="text-[#92A5A8] hover:text-[#FCFFFF] transition-colors"
            >
              <ArrowLeft size={20} />
            </button>
            <h2 className="text-[14px]  font-extralight text-[#92A5A8] uppercase tracking-wide">
              Biometric Authentication
            </h2>
          </div>
          {/* Content */}
          <div className="p-6 md:p-6 space-y-3 mt-14">
            <h3 className="text-2xl font-semibold text-[#FCFFFF] text-center">
              Enable Fingerprint Unlock
            </h3>

            <p className="text-[12px]  text-[#92A5A8] text-center">
              Rest your finger on the print scanner to record your fingerprint
            </p>

            {/* Fingerprint Icon */}
            <div className="flex justify-center py-4">
              <Image src={thumbPrint} alt="Fingerprint Icon" />
            </div>
          </div>
          {/* Footer */}
          <div className="p-3 border-t max-w-[370px]  mx-auto border-[#1C252A]">
            <button
              onClick={onContinue}
              className="w-full py-4 px-6 bg-[#33C5E0] text-[#161E22] font-semibold rounded-full  hover:bg-[#33C5E0]/90 transition-colors"
            >
              Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
