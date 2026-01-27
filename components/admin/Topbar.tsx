"use client";

import React from "react";
import { Shield, ChevronDown } from "lucide-react";
import Image from "next/image";

export function Topbar() {
    return (
        <header className="h-20 border-b border-[#161E22] flex items-center justify-between px-6 md:px-10 bg-[#060B0D]">
            <div>
                {/* Mobile menu trigger could go here */}
            </div>

            <div className="flex items-center gap-6">
                <div className="flex items-center gap-4">
                    <div className="bg-[#0A0F11] border border-[#161E22] rounded-full py-1.5 px-3 flex items-center gap-2">
                        <span className="text-sm font-semibold text-white">0.079 ETH</span>
                        <div className="flex items-center gap-2 bg-[#161E22] rounded-full py-1 px-2.5">
                            <div className="w-5 h-5 relative rounded-full overflow-hidden bg-orange-400">
                                {/* Avatar/Identicon placeholder */}
                            </div>
                            <span className="text-xs font-mono text-[#8899A6]">0x8a...0fe5</span>
                            <ChevronDown size={14} className="text-[#8899A6]" />
                        </div>
                    </div>
                </div>

                <button className="bg-[#33C5E0] hover:bg-[#2BAAC2] text-black font-bold py-2.5 px-6 rounded-lg flex items-center gap-2 transition-all duration-200">
                    <Shield size={18} />
                    <span className="text-sm">Review KYC</span>
                </button>
            </div>
        </header>
    );
}
