"use client";
import { SearchFilterBarProps, KYCStatus } from "@/lib/adminTypes";

export function SearchFilterBar({ onSearchChange, onFilterChange }: SearchFilterBarProps) {
  return (
    <div className="flex flex-col sm:flex-row gap-3 mb-6">
      {/* Search by wallet address */}
      <div className="relative flex-1">
        <svg
          className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
        <input
          type="text"
          placeholder="Search by wallet address..."
          onChange={(e) => onSearchChange(e.target.value)}
          className="w-full pl-9 pr-4 py-2.5 text-sm bg-white/5 border border-white/10 rounded-lg text-foreground placeholder-gray-500 focus:outline-none focus:border-primary/50 focus:bg-white/8 transition-colors"
        />
      </div>

      {/* Filter by KYC status */}
      <select
        onChange={(e) => onFilterChange(e.target.value as KYCStatus | "all")}
        className="px-4 py-2.5 text-sm bg-white/5 border border-white/10 rounded-lg text-foreground focus:outline-none focus:border-primary/50 transition-colors cursor-pointer sm:w-48"
      >
        <option value="all" className="bg-[#0a0a0a]">All KYC Status</option>
        <option value="pending" className="bg-[#0a0a0a]">Pending</option>
        <option value="approved" className="bg-[#0a0a0a]">Approved</option>
        <option value="rejected" className="bg-[#0a0a0a]">Rejected</option>
      </select>
    </div>
  );
}
