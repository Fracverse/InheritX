"use client";

import React from "react";
import { Activity } from "lucide-react";
import { ActivityItem, ActivityData } from "./ActivityItem";

interface ActivityListProps {
    activities: ActivityData[];
}

export function ActivityList({ activities }: ActivityListProps) {
    return (
        <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl overflow-hidden flex flex-col">
            <div className="p-4 md:p-6 border-b border-[#161E22] flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                <div className="flex items-center gap-2">
                    <Activity size={18} className="text-[#33C5E0]" />
                    <h3 className="text-sm font-bold text-white">Recent Activity</h3>
                </div>
                <span className="text-[#4A5568] text-xs font-medium self-end sm:self-auto">
                    {activities.length} total events
                </span>
            </div>

            <div className="divide-y divide-[#161E22]">
                {activities.length === 0 ? (
                    <div className="p-8 text-center text-[#4A5568] text-sm">
                        No activities found for this filter.
                    </div>
                ) : (
                    activities.map((activity, index) => (
                        <ActivityItem key={activity.id} activity={activity} index={index} />
                    ))
                )}
            </div>
        </div>
    );
}
