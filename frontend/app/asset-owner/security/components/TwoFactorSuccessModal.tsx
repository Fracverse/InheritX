"use client";

import Image from "next/image";
import success from "@/public/stuff.svg";

interface TwoFactorSuccessModalProps {
  onContinue: () => void;
}

export default function TwoFactorSuccessModal({
  onContinue,
}: TwoFactorSuccessModalProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
      <div
        className="overflow-y-scroll w-full md:w-[812px] relative   overflow-hidden bg-[#161E22] border border-[#1C252A] flex justify-center  rounded-3xl max-h-[90vh]  no-scrollbar"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="bg-[#161E22] pb-5 overflow-y-scroll w-full  rounded-3xl no-scrollbar overflow-hidden animate-scale-up">
          {/* Content */}
          <div className="p-6 md:p-6 space-y-3 ">
            <h3 className="text-[16px]  text-[#FCFFFF] text-center">
              Two-Factor Authentication successful
            </h3>

            {/* Fingerprint Icon */}
            <div className="flex justify-center py-4">
              <Image src={success} alt="success Icon" />
            </div>
          </div>
          {/* Footer */}
          <div className="p-3 border-t max-w-92.5  mx-auto border-[#1C252A]">
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
