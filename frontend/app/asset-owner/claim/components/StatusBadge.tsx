"use client";

import React from "react";

interface StatusBadgeProps {
  status: string;
  type?: "status" | "trigger";
}

export default function StatusBadge({ status, type = "status" }: StatusBadgeProps) {
  const getStyle = () => {
    if (type === "trigger") {
      return "bg-[#2A3338] text-[#92A5A8] border border-[#39494F]";
    }

    switch (status) {
      case "ACTIVE":
        return "bg-[#33C5E014] border border-[#33C5E03D] text-[#33C5E0]";
      case "COMPLETED":
        return "bg-transparent border border-[#D4A017] text-[#D4A017]";
      case "PENDING":
        return "bg-[#D4A017] text-[#161E22]";
      case "EXPIRED":
        return "bg-transparent border border-[#4A5558] text-[#4A5558]";
      default:
        return "bg-[#1C252A] text-[#92A5A8]";
    }
  };

  return (
    <span className={`px-3 py-1.5 rounded-3xl text-xs ${getStyle()}`}>
      {status}
    </span>
  );
}
