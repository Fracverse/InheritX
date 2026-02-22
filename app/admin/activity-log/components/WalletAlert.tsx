"use client";

import React from "react";
import { AlertTriangle } from "lucide-react";
import { motion } from "framer-motion";

export function WalletAlert() {
    return (
        <motion.div 
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="bg-[#1A160A] border border-[#5C4D1F] rounded-xl p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4"
        >
            <div className="flex gap-3">
                <div className="text-[#FFB800] mt-0.5">
                    <AlertTriangle size={20} />
                </div>
                <div>
                    <h3 className="text-[#FFB800] font-semibold text-sm">Wallet not connected</h3>
                    <p className="text-[#8899A6] text-xs mt-0.5">Connect your wallet to perform blockchain operations like KYC approval.</p>
                </div>
            </div>
            <button className="bg-[#33C5E0] hover:bg-[#2CB0C9] text-[#060B0D] text-sm font-bold px-4 py-2 rounded-lg transition-colors whitespace-nowrap w-full sm:w-auto">
                Connect Wallet
            </button>
        </motion.div>
    );
}
