"use client";

import React from "react";
import { VerificationResult } from "@/app/lib/api/verification";

interface Props {
  result: VerificationResult;
  onClose?: () => void;
}

export default function VerificationCertificate({ result, onClose }: Props) {
  return (
    <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6">
      <div className="text-center space-y-4">
        <div className="w-16 h-16 mx-auto bg-green-500/10 rounded-full flex items-center justify-center">
          <svg
            className="w-8 h-8 text-green-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>

        <div>
          <h3 className="text-xl font-semibold text-white">
            Verification Certificate
          </h3>
          <p className="text-gray-400 text-sm mt-1">
            Document authenticity confirmed
          </p>
        </div>

        <div className="bg-[#1C252A] rounded-xl p-4 space-y-3">
          <div className="flex justify-between items-center">
            <span className="text-gray-400 text-sm">Document Hash:</span>
            <span className="text-white font-mono text-xs">
              {result.stored_hash?.substring(0, 16)}...
            </span>
          </div>
          
          <div className="flex justify-between items-center">
            <span className="text-gray-400 text-sm">Status:</span>
            <span className="text-green-400 font-semibold">
              {result.is_valid ? "Valid" : "Invalid"}
            </span>
          </div>

          <div className="flex justify-between items-center">
            <span className="text-gray-400 text-sm">Verified At:</span>
            <span className="text-white text-sm">
              {new Date(result.verified_at).toLocaleDateString()}
            </span>
          </div>
        </div>

        <div className="flex gap-3">
          <button className="flex-1 bg-[#1C252A] hover:bg-[#252E34] text-white px-6 py-3 rounded-xl transition-colors">
            Download Certificate
          </button>
          {onClose && (
            <button 
              onClick={onClose}
              className="flex-1 bg-gray-600 hover:bg-gray-700 text-white px-6 py-3 rounded-xl transition-colors"
            >
              Close
            </button>
          )}
        </div>
      </div>
    </div>
  );
}