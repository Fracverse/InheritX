"use client";

import React, { useState } from "react";
import { ClipboardList } from "lucide-react";
import { WalletAlert } from "@/app/admin/activity-log/components/WalletAlert";
import { ActivityFilters } from "@/app/admin/activity-log/components/ActivityFilters";
import { ActivityList } from "@/app/admin/activity-log/components/ActivityList";
import { ALL_ACTIVITIES, TABS } from "@/app/admin/activity-log/components/mock-data";

export default function ActivityLogPage() {
    const [activeTab, setActiveTab] = useState("All Activity");

    const filteredActivities = activeTab === "All Activity" 
        ? ALL_ACTIVITIES 
        : ALL_ACTIVITIES.filter(item => item.tag === activeTab);

    return (
        <div className="space-y-8 pb-10">
            <WalletAlert />

            <div className="space-y-2">
                <div className="flex items-center gap-2">
                    <h1 className="text-3xl font-bold text-white">Activity Log</h1>
                    <ClipboardList size={28} className="text-[#C5D0D6]" />
                </div>
                <p className="text-[#8899A6] text-sm">View all platform activity and events.</p>
            </div>

            <ActivityFilters 
                tabs={TABS} 
                activeTab={activeTab} 
                onTabChange={setActiveTab} 
            />

            <ActivityList activities={filteredActivities} />
        </div>
    );
}