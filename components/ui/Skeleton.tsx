"use client";

import { cn } from "@/util/cn";

interface SkeletonProps {
  className?: string;
}

/** Base shimmer skeleton block */
export function Skeleton({ className }: SkeletonProps) {
  return (
    <div
      className={cn(
        "animate-pulse rounded-md bg-[#1C252A]",
        className,
      )}
    />
  );
}

/** Skeleton for a stat card (number + label) */
export function StatCardSkeleton() {
  return (
    <div className="px-[22px] py-8 bg-[#182024] flex flex-col items-center rounded-3xl gap-2">
      <Skeleton className="h-10 w-16" />
      <Skeleton className="h-4 w-24" />
    </div>
  );
}

/** Skeleton for a plan row in the plans table */
export function PlanRowSkeleton() {
  return (
    <div className="flex items-center gap-4 p-4 rounded-2xl bg-[#182024]">
      <Skeleton className="h-10 w-10 rounded-full" />
      <div className="flex-1 space-y-2">
        <Skeleton className="h-4 w-32" />
        <Skeleton className="h-3 w-20" />
      </div>
      <Skeleton className="h-6 w-20 rounded-full" />
    </div>
  );
}

/** Skeleton for a lending stats card */
export function LendingStatSkeleton() {
  return (
    <div className="p-6 bg-[#182024] rounded-2xl space-y-3">
      <Skeleton className="h-4 w-28" />
      <Skeleton className="h-8 w-36" />
      <Skeleton className="h-3 w-20" />
    </div>
  );
}

/** Skeleton for a transaction row */
export function TransactionRowSkeleton() {
  return (
    <div className="flex items-center justify-between p-4 border-b border-[#1C252A]">
      <div className="flex items-center gap-3">
        <Skeleton className="h-8 w-8 rounded-full" />
        <div className="space-y-1">
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-3 w-16" />
        </div>
      </div>
      <Skeleton className="h-4 w-16" />
    </div>
  );
}

/** Skeleton for a message card */
export function MessageCardSkeleton() {
  return (
    <div className="p-4 bg-[#182024] rounded-2xl space-y-3">
      <div className="flex items-center justify-between">
        <Skeleton className="h-5 w-40" />
        <Skeleton className="h-5 w-16 rounded-full" />
      </div>
      <Skeleton className="h-4 w-full" />
      <Skeleton className="h-4 w-3/4" />
      <Skeleton className="h-3 w-24" />
    </div>
  );
}

/** Full-page loading skeleton for the dashboard */
export function DashboardSkeleton() {
  return (
    <div className="space-y-6 animate-pulse">
      <div className="space-y-2">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-64" />
      </div>
      <div className="grid grid-cols-2 gap-5">
        {Array.from({ length: 4 }).map((_, i) => (
          <StatCardSkeleton key={i} />
        ))}
      </div>
      <div className="space-y-3">
        <Skeleton className="h-5 w-32" />
        {Array.from({ length: 3 }).map((_, i) => (
          <TransactionRowSkeleton key={i} />
        ))}
      </div>
    </div>
  );
}

/** Skeleton for the plans page */
export function PlansPageSkeleton() {
  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="h-10 w-32 rounded-full" />
      </div>
      {Array.from({ length: 4 }).map((_, i) => (
        <PlanRowSkeleton key={i} />
      ))}
    </div>
  );
}

/** Skeleton for the lending page */
export function LendingPageSkeleton() {
  return (
    <div className="space-y-8">
      <div className="space-y-2">
        <Skeleton className="h-8 w-40" />
        <Skeleton className="h-4 w-56" />
      </div>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <LendingStatSkeleton key={i} />
        ))}
      </div>
      <div className="space-y-3">
        <Skeleton className="h-5 w-36" />
        {Array.from({ length: 5 }).map((_, i) => (
          <TransactionRowSkeleton key={i} />
        ))}
      </div>
    </div>
  );
}
