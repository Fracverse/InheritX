"use client";

import React, { Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import { ViewState, ClaimFormData } from "./types";
import { MOCK_CLAIMS, MOCK_ACTIVITIES, MOCK_PLAN_SUMMARY } from "./data/mockData";
import SuccessModal from "./components/SuccessModal";
import ErrorModal from "./components/ErrorModal";
import ClaimForm from "./components/ClaimForm";
import PlanSummary from "./components/PlanSummary";
import SummaryCards from "./components/SummaryCards";
import Tabs from "./components/Tabs";
import ClaimsTable from "./components/ClaimsTable";
import ActivitiesList from "./components/ActivitiesList";

function ClaimPageContent() {
  const searchParams = useSearchParams();
  const [activeTab, setActiveTab] = useState<"claims" | "activities">("claims");
  const [viewState, setViewState] = useState<ViewState>("list");
  const [selectedClaim, setSelectedClaim] = useState<string | null>(null);
  const claimResult = searchParams.get("claimResult");
  const claimId = searchParams.get("claimId");
  const effectiveViewState: ViewState =
    claimResult === "success"
      ? "success"
      : claimResult === "summary"
        ? "summary"
        : selectedClaim || claimId
          ? "form"
          : viewState;

  const handleSubmitClaim = (formData: ClaimFormData) => {
    // Simulate API call
    const isSuccess =
      process.env.NEXT_PUBLIC_E2E_MOCK_WALLET === "true" || Math.random() > 0.3; // 70% success rate for demo
    setTimeout(() => {
      setViewState(isSuccess ? "success" : "error");
    }, 1000);
  };

  const handleSuccessContinue = () => {
    setViewState("summary");
  };

  const handleSuccessCancel = () => {
    setViewState("list");
  };

  const handleErrorContinue = () => {
    setViewState("form");
  };

  const handleErrorCancel = () => {
    setViewState("form");
  };

  const handleClaimClick = (claimId: string) => {
    setSelectedClaim(claimId);
    setViewState("form");
  };

  const handleViewClaims = () => {
    setViewState("summary");
  };

  // Success Modal
  if (effectiveViewState === "success") {
    return (
      <SuccessModal onCancel={handleSuccessCancel} onContinue={handleSuccessContinue} />
    );
  }

  // Error Modal
  if (effectiveViewState === "error") {
    return (
      <ErrorModal onCancel={handleErrorContinue} onContinue={handleErrorContinue} />
    );
  }

  // Claim Form View
  if (effectiveViewState === "form") {
    return <ClaimForm onSubmit={handleSubmitClaim} />;
  }

  // Plan Summary View
  if (effectiveViewState === "summary") {
    return (
      <PlanSummary
        data={MOCK_PLAN_SUMMARY}
        onBack={() => setViewState("list")}
        onWithdraw={() => {
          // Handle withdraw action
        }}
      />
    );
  }

  // Default List View
  return (
    <div className="animate-fade-in">
      {/* Header */}
      <div className="mb-6 md:mb-8">
        <h1 className="text-xl md:text-3xl font-semibold text-[#FCFFFF] mb-1 md:mb-2">
          Claim Plan
        </h1>
        <p className="text-xs md:text-base text-[#92A5A8]">
          Claim your inheritance here
        </p>
      </div>

      {/* Summary Cards */}
      <SummaryCards onViewClaims={handleViewClaims} />

      {/* Tabs and Filter */}
      <Tabs
        activeTab={activeTab}
        onTabChange={setActiveTab}
        onFilterClick={() => {
          // Handle filter click
        }}
      />

      {/* Claims Tab Content */}
      {activeTab === "claims" && (
        <ClaimsTable claims={MOCK_CLAIMS} onClaimClick={handleClaimClick} />
      )}

      {/* Activities Tab Content */}
      {activeTab === "activities" && <ActivitiesList activities={MOCK_ACTIVITIES} />}
    </div>
  );
}

export default function ClaimPage() {
  return (
    <Suspense fallback={null}>
      <ClaimPageContent />
    </Suspense>
  );
}
