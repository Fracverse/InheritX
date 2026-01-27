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
                <button className="bg-[#33C5E0] hover:bg-[#2BAAC2] text-black font-bold py-2.5 px-6 rounded-lg transition-all duration-200">
                    <span className="text-sm">Connect Wallet</span>
                </button>
            </div>
        </header>
    );
}
