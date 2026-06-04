"use client";

import React from "react";
import { Activity, ArrowRight } from "lucide-react";
import Link from "next/link";
import { motion } from "framer-motion";

const activities = [
    {
        id: 1,
        type: "Admin logged in via email/password",
        user: "0x0000...0001",
        time: "Jan 24, 2026, 11:41 AM",
        tag: "USER LOGIN",
    },
    {
        id: 2,
        type: "User logged in",
        user: "0x8a4f...0fe5",
        time: "Jan 22, 2026, 07:07 PM",
        tag: "USER LOGIN",
    },
    {
        id: 3,
        type: "User logged in",
        user: "0x1a75...8c5c",
        time: "Jan 22, 2026, 05:41 PM",
        tag: "USER LOGIN",
    },
    {
        id: 4,
        type: "User logged in",
        user: "0x8a4f...0fe5",
        time: "Jan 21, 2026, 11:47 AM",
        tag: "USER LOGIN",
    },
];

export function RecentActivity() {
    return (
        <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl overflow-hidden flex flex-col">
            <div className="p-6 border-b border-[#161E22] flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Activity size={20} className="text-[#33C5E0]" />
                    <h3 className="text-lg font-bold text-white">Recent Activity</h3>
                </div>
                <Link
                    href="/admin/activity"
                    className="text-[#33C5E0] text-sm font-semibold flex items-center gap-1 hover:underline"
                >
                    View All <ArrowRight size={14} />
                </Link>
            </div>

            <div className="divide-y divide-[#161E22]">
                {activities.map((activity, index) => (
                    <motion.div
                        key={activity.id}
                        initial={{ opacity: 0, x: -10 }}
                        animate={{ opacity: 1, x: 0 }}
                        transition={{ duration: 0.3, delay: 0.2 + index * 0.05 }}
                        className="p-6 flex items-center justify-between hover:bg-[#161E22]/20 transition-colors"
                    >
                        <div className="flex items-center gap-4">
                            <div className="w-10 h-10 rounded-xl bg-[#161E22] flex items-center justify-center text-[#33C5E0]">
                                <Activity size={18} />
                            </div>
                            <div>
                                <p className="text-sm font-semibold text-white">{activity.type}</p>
                                <div className="flex items-center gap-2 mt-1">
                                    <span className="text-xs font-mono text-[#8899A6]">{activity.user}</span>
                                    <span className="text-[#4A5568]">•</span>
                                    <span className="text-xs text-[#8899A6]">{activity.time}</span>
                                </div>
                            </div>
                        </div>
                        <span className="bg-[#161E22] text-[#8899A6] text-[10px] px-2 py-0.5 rounded font-bold uppercase tracking-wider">
                            {activity.tag}
                        </span>
                    </motion.div>
                ))}
            </div>
        </div>
    );
}
