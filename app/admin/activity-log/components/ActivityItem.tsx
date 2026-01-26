"use client";

import React from "react";
import { Activity } from "lucide-react";
import { motion } from "framer-motion";

export interface ActivityData {
    id: number;
    type: string;
    user: string;
    time: string;
    tag: string;
}

interface ActivityItemProps {
    activity: ActivityData;
    index: number;
}

export function ActivityItem({ activity, index }: ActivityItemProps) {
    return (
        <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.3, delay: index * 0.05 }}
            className="p-4 md:p-6 flex flex-col md:flex-row md:items-center justify-between gap-4 hover:bg-[#161E22]/20 transition-colors group cursor-default"
        >
            <div className="flex items-start gap-4">
                <div className="w-10 h-10 rounded-xl bg-[#161E22] flex items-center justify-center text-[#33C5E0] shrink-0 group-hover:bg-[#33C5E0]/10 transition-colors mt-1 md:mt-0">
                    <Activity size={18} />
                </div>
                <div className="min-w-0 flex-1">
                    <p className="text-sm font-semibold text-white group-hover:text-[#33C5E0] transition-colors wrap-break-word">
                        {activity.type}
                    </p>
                    <div className="flex flex-wrap items-center gap-x-2 gap-y-1 mt-1">
                        <span className="text-xs font-mono text-[#8899A6] truncate max-w-37.5 sm:max-w-none">
                            {activity.user}
                        </span>
                        <span className="hidden sm:inline text-[#4A5568]">•</span>
                        <span className="text-xs text-[#8899A6]">{activity.time}</span>
                    </div>
                </div>
            </div>
            <span className="bg-[#161E22] text-[#33C5E0] text-[10px] px-3 py-1 rounded font-bold uppercase tracking-wider self-start md:self-center ml-14 md:ml-0 whitespace-nowrap">
                {activity.tag}
            </span>
        </motion.div>
    );
}
