"use client";

import React from "react";
import { Users, Clock, FileText, DollarSign } from "lucide-react";
import { motion } from "framer-motion";
import { useApi } from "@/app/hooks/useApi";
import { adminAPI } from "@/app/lib/api/admin";
import { SkeletonStats } from "@/components/ui/Skeleton";

export function StatsCards() {
    const { data: metrics, loading, error } = useApi(
        () => adminAPI.getMetrics(),
        {
            immediate: true,
            onError: (err) => console.error("Failed to fetch metrics:", err),
        }
    );

    if (loading) {
        return <SkeletonStats />;
    }

    if (error) {
        return (
            <div className="bg-red-500/10 border border-red-500/30 rounded-2xl p-6 text-center">
                <p className="text-red-400">Failed to load metrics: {error}</p>
            </div>
        );
    }

    const stats = [
        {
            label: "Total Users",
            value: metrics?.total_users?.toString() || "0",
            icon: Users,
            color: "#33C5E0",
            bg: "rgba(51, 197, 224, 0.1)",
        },
        {
            label: "Active Plans",
            value: metrics?.active_plans?.toString() || "0",
            icon: Clock,
            color: "#9F7AEA",
            bg: "rgba(159, 122, 234, 0.1)",
        },
        {
            label: "Total Plans",
            value: metrics?.total_plans?.toString() || "0",
            icon: FileText,
            color: "#48BB78",
            bg: "rgba(72, 187, 120, 0.1)",
        },
        {
            label: "Total Claims",
            value: metrics?.total_claims?.toString() || "0",
            icon: DollarSign,
            color: "#ECC94B",
            bg: "rgba(236, 201, 75, 0.1)",
        },
    ];

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {stats.map((stat, index) => (
                <motion.div
                    key={stat.label}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.4, delay: index * 0.1 }}
                    className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6 flex flex-col gap-4 hover:border-[#33C5E0]/30 transition-all duration-300"
                >
                    <div
                        className="w-12 h-12 rounded-xl flex items-center justify-center"
                        style={{ backgroundColor: stat.bg }}
                    >
                        <stat.icon size={24} style={{ color: stat.color }} />
                    </div>
                    <div>
                        <div className="text-3xl font-bold text-white mb-1">{stat.value}</div>
                        <div className="text-sm font-medium text-[#8899A6]">{stat.label}</div>
                    </div>
                </motion.div>
            ))}
        </div>
    );
}
