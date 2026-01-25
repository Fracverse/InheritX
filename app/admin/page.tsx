"use client";

import React from "react";
import { StatsCards } from "@/components/admin/StatsCards";
import { RecentActivity } from "@/components/admin/RecentActivity";
import { KYCOverview } from "@/components/admin/KYCOverview";
import { QuickActions } from "@/components/admin/QuickActions";
import { motion } from "framer-motion";

export default function AdminDashboardPage() {
    return (
        <div className="space-y-10">
            {/* Header Section */}
            <motion.div
                initial={{ opacity: 0, y: -20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5 }}
                className="flex flex-col gap-2"
            >
                <h1 className="text-3xl font-bold text-white flex items-center gap-2">
                    Admin Dashboard 🏛️
                </h1>
                <p className="text-[#8899A6]">
                    Welcome back, Super Admin. Here&apos;s the platform overview.
                </p>
            </motion.div>

            {/* Stats Section */}
            <StatsCards />

            {/* Main Grid Section */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                <div className="lg:col-span-2">
                    <RecentActivity />
                </div>
                <div className="space-y-8">
                    <KYCOverview />
                    <QuickActions />
                </div>
            </div>
        </div>
    );
}
