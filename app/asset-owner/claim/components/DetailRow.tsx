import React, { ReactNode } from "react";

interface DetailRowProps {
  label: string;
  value: ReactNode;
  className?: string;
  last?: boolean;
}

export default function DetailRow({ label, value, className = "", last = false }: DetailRowProps) {
  return (
    <div className={`grid grid-cols-1 md:grid-cols-[180px_1fr] gap-1 md:gap-4 ${!last ? "border-b border-[#1C252A] pb-3 md:pb-4" : ""} ${className}`}>
      <span className="text-xs md:text-sm text-[#92A5A8] uppercase">{label}</span>
      <div className="text-sm md:text-base text-[#FCFFFF] font-medium break-all">
        {value}
      </div>
    </div>
  );
}
