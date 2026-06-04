"use client";

import React from "react";

interface Activity {
  id: number;
  description: string;
  timestamp: string;
}

interface ActivitiesListProps {
  activities: Activity[];
}

export default function ActivitiesList({ activities }: ActivitiesListProps) {
  return (
    <div className="bg-[#1C252A] rounded-2xl p-4 md:p-6">
      <div className="space-y-3 md:space-y-4">
        {activities.map((activity, index) => (
          <div
            key={activity.id}
            className="flex flex-col md:flex-row md:justify-between md:items-center gap-2 py-3 md:py-4 border-b border-[#2A3338] last:border-0"
          >
            <div className="flex items-start md:items-center gap-3 md:gap-4">
              <span className="text-[#92A5A8] text-xs md:text-sm w-6 md:w-8 flex-shrink-0">
                {index + 1}.
              </span>
              <span className="text-sm md:text-base text-[#FCFFFF] font-medium break-words">
                {activity.description}
              </span>
            </div>
            <span className="text-[#92A5A8] text-xs md:text-sm md:ml-4 flex-shrink-0">
              {activity.timestamp}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
