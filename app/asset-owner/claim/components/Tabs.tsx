"use client";

import React from "react";
import { SlidersHorizontal } from "lucide-react";

interface TabsProps {
  activeTab: "claims" | "activities";
  onTabChange: (tab: "claims" | "activities") => void;
  onFilterClick?: () => void;
}

export default function Tabs({ activeTab, onTabChange, onFilterClick }: TabsProps) {
  return (
    <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-3 md:gap-4 mb-4 md:mb-6">
      <div className="flex border-b border-[#1C252A] w-full md:w-auto">
        <button
          onClick={() => onTabChange("claims")}
          className={`py-2 px-3 md:px-4 text-xs md:text-sm font-medium transition-colors flex-1 md:flex-none ${
            activeTab === "claims"
              ? "text-[#33C5E0] border-b-2 border-[#33C5E0]"
              : "text-[#92A5A8] hover:text-[#FCFFFF]"
          }`}
        >
          Claims
        </button>
        <button
          onClick={() => onTabChange("activities")}
          className={`py-2 px-3 md:px-4 text-xs md:text-sm font-medium transition-colors flex-1 md:flex-none ${
            activeTab === "activities"
              ? "text-[#33C5E0] border-b-2 border-[#33C5E0]"
              : "text-[#92A5A8] hover:text-[#FCFFFF]"
          }`}
        >
          Activities
        </button>
      </div>
      <button
        onClick={onFilterClick}
        className="flex items-center gap-x-2 text-[#92A5A8] hover:text-[#FCFFFF] transition-colors text-xs md:text-sm w-full md:w-auto justify-end md:justify-start"
      >
        <SlidersHorizontal size={14} className="md:w-4 md:h-4" />
        <span>Filter</span>
      </button>
    </div>
  );
}
