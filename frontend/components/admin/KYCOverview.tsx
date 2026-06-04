"use client";

import React from "react";
import { Clock, CheckCircle2, XCircle } from "lucide-react";
import Link from "next/link";
import { motion } from "framer-motion";

const kycStats = [
    { label: "Pending", value: "0", icon: Clock, color: "#9F7AEA", bg: "rgba(155, 108, 247, 0.1)" },
    { label: "Approved", value: "3", icon: CheckCircle2, color: "#48BB78", bg: "rgba(72, 187, 120, 0.1)" },
    { label: "Rejected", value: "0", icon: XCircle, color: "#F56565", bg: "rgba(245, 101, 101, 0.1)" },
];

export function KYCOverview() {
    return (
        <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6 flex flex-col gap-6">
            <div className="flex items-center justify-between">
                <h3 className="font-bold text-white">KYC Overview</h3>
                <Link href="/admin/kyc-management" className="text-[#33C5E0] text-xs font-semibold hover:underline">
                    View All
                </Link>
            </div>

            <div className="space-y-3">
                {kycStats.map((stat, index) => (
                    <motion.div
                        key={stat.label}
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ duration: 0.3, delay: 0.4 + index * 0.1 }}
                         style={{ backgroundColor: stat.bg }}
                        className="border border-[#161E22] rounded-xl p-4 flex items-center justify-between"
                    >
                        <div className="flex items-center gap-3" >
                            <div
                                className="w-8 h-8 rounded-lg flex items-center justify-center"
                               
                            >
                                <stat.icon size={16} style={{ color: stat.color }} />
                            </div>
                            <div>
                                <p className="text-[10px] text-[#8899A6] font-medium uppercase tracking-wider">{stat.label}</p>
                                <p className="text-xl font-bold text-white leading-tight">{stat.value}</p>
                            </div>
                        </div>
                    </motion.div>
                ))}
            </div>

            <div className="pt-4 border-t border-[#161E22] mt-auto">
                <div className="flex items-end gap-2">
                    <span className="text-3xl font-bold text-white">0</span>
                    <span className="text-[#8899A6] text-sm mb-1.5 font-medium">overall KYC processed</span>
                </div>
            </div>
        </div>
    );
}
