"use client";

import React from "react";
import SummaryCard from "./SummaryCard";

interface SummaryCardsProps {
  onViewClaims: () => void;
}

export default function SummaryCards({ onViewClaims }: SummaryCardsProps) {
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-3 md:gap-6 mb-6 md:mb-8">
      <SummaryCard
        count={2}
        label="Tokens"
        buttonText="Withdraw Token"
        buttonTextMobile="Withdraw"
      />
      <SummaryCard
        count={0}
        label="NFT"
        buttonText="Withdraw NFT"
        buttonTextMobile="Withdraw"
        disabled={true}
      />
      <SummaryCard
        count={0}
        label="Real World Asset"
        buttonText="Withdraw Asset"
        buttonTextMobile="Withdraw"
        disabled={true}
      />
      <SummaryCard
        count={1}
        label="Pending Inheritance"
        buttonText="View Claims"
        buttonTextMobile="View"
        onClick={onViewClaims}
      />
    </div>
  );
}
