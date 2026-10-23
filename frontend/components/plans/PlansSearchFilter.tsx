"use client";

import React, { useState, useEffect, useCallback, useTransition } from "react";
import { useSearchParams, useRouter, usePathname } from "next/navigation";
import { Search, X, SlidersHorizontal, Loader2 } from "lucide-react";

const STATUS_TABS = [
  { value: "all", label: "All" },
  { value: "active", label: "Active" },
  { value: "inactive", label: "Inactive" },
  { value: "pending", label: "Pending Claim" },
  { value: "disputed", label: "Disputed" },
] as const;

type StatusValue = (typeof STATUS_TABS)[number]["value"];

interface PlansSearchFilterProps {
  onFilterChange?: (search: string, status: string) => void;
  resultCount?: number;
  totalCount?: number;
}

export function PlansSearchFilter({
  onFilterChange,
  resultCount,
  totalCount,
}: PlansSearchFilterProps) {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const [isPending, startTransition] = useTransition();

  const [search, setSearch] = useState(searchParams.get("search") || "");
  const status = (searchParams.get("status") || "all") as StatusValue;

  // Sync URL → state on external navigation
  useEffect(() => {
    setSearch(searchParams.get("search") || "");
  }, [searchParams]);

  const updateParam = useCallback(
    (key: string, value: string | null) => {
      const params = new URLSearchParams(searchParams.toString());
      if (value && value !== "all") {
        params.set(key, value);
      } else {
        params.delete(key);
      }
      const query = params.toString();
      startTransition(() => {
        router.replace(`${pathname}${query ? `?${query}` : ""}`, { scroll: false });
      });
    },
    [router, pathname, searchParams]
  );

  // Debounced search → URL
  useEffect(() => {
    const timer = setTimeout(() => {
      const current = searchParams.get("search") || "";
      if (search !== current) {
        updateParam("search", search || null);
      }
      onFilterChange?.(search, status);
    }, 300);
    return () => clearTimeout(timer);
  }, [search, status, searchParams, updateParam, onFilterChange]);

  const handleStatusChange = (newStatus: StatusValue) => {
    updateParam("status", newStatus === "all" ? null : newStatus);
    onFilterChange?.(search, newStatus);
  };

  const clearAll = () => {
    setSearch("");
    startTransition(() => {
      router.replace(pathname, { scroll: false });
    });
    onFilterChange?.("", "all");
  };

  const hasFilters = search || status !== "all";

  return (
    <div className="space-y-3" data-testid="plans-search-filter">
      <div className="flex flex-col sm:flex-row gap-3">
        {/* Search input */}
        <div className="relative flex-1">
          <Search
            size={14}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 pointer-events-none"
            aria-hidden="true"
          />
          <input
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by plan name, token symbol, or beneficiary address…"
            className="w-full pl-9 pr-9 py-2.5 rounded-xl bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-[#33C5E0]/40 transition-colors"
            aria-label="Search plans"
            data-testid="plans-search-input"
          />
          {isPending && (
            <Loader2
              size={14}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 animate-spin"
            />
          )}
          {!isPending && search && (
            <button
              onClick={() => setSearch("")}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 transition-colors"
              aria-label="Clear search"
            >
              <X size={14} />
            </button>
          )}
        </div>

        {/* Clear all */}
        {hasFilters && (
          <button
            onClick={clearAll}
            className="flex items-center gap-1.5 px-3 py-2.5 rounded-xl bg-white/5 border border-white/10 text-xs text-slate-400 hover:text-white hover:bg-white/10 transition-colors whitespace-nowrap"
          >
            <X size={12} />
            Clear filters
          </button>
        )}
      </div>

      {/* Status tabs */}
      <div className="flex items-center gap-2 overflow-x-auto pb-1" role="tablist" aria-label="Filter by status">
        <SlidersHorizontal size={13} className="text-slate-500 flex-shrink-0" aria-hidden="true" />
        {STATUS_TABS.map((tab) => (
          <button
            key={tab.value}
            role="tab"
            aria-selected={status === tab.value}
            onClick={() => handleStatusChange(tab.value)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors whitespace-nowrap ${
              status === tab.value
                ? "bg-[#33C5E0] text-black"
                : "bg-white/5 text-slate-400 border border-white/10 hover:text-white hover:bg-white/10"
            }`}
            data-testid={`status-tab-${tab.value}`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Result count */}
      {typeof resultCount === "number" && typeof totalCount === "number" && (
        <p className="text-[11px] text-slate-500" aria-live="polite">
          Showing {resultCount} of {totalCount} plan{totalCount !== 1 ? "s" : ""}
          {hasFilters && " (filtered)"}
        </p>
      )}

      {/* Active filter chips */}
      {hasFilters && (
        <div className="flex flex-wrap gap-1.5">
          {search && (
            <span className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-[#33C5E0]/10 border border-[#33C5E0]/20 text-[10px] text-[#33C5E0]">
              Search: "{search.length > 20 ? search.slice(0, 20) + "…" : search}"
              <button
                onClick={() => setSearch("")}
                className="hover:text-white"
                aria-label="Remove search filter"
              >
                <X size={10} />
              </button>
            </span>
          )}
          {status !== "all" && (
            <span className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-[#33C5E0]/10 border border-[#33C5E0]/20 text-[10px] text-[#33C5E0]">
              Status: {STATUS_TABS.find((t) => t.value === status)?.label}
              <button
                onClick={() => handleStatusChange("all")}
                className="hover:text-white"
                aria-label="Remove status filter"
              >
                <X size={10} />
              </button>
            </span>
          )}
        </div>
      )}
    </div>
  );
}
