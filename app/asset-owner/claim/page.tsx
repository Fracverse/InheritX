"use client";

import React, { useState } from "react";
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

export default function ClaimPage() {
  const [activeTab, setActiveTab] = useState<"claims" | "activities">("claims");
  const [viewState, setViewState] = useState<ViewState>("list");
  const [selectedClaim, setSelectedClaim] = useState<string | null>(null);

  const handleSubmitClaim = (formData: ClaimFormData) => {
    // Simulate API call
    const isSuccess = Math.random() > 0.3; // 70% success rate for demo
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
  if (viewState === "success") {
    return (
      <SuccessModal onCancel={handleSuccessCancel} onContinue={handleSuccessContinue} />
    );
  }

  // Error Modal
  if (viewState === "error") {
    return (
      <ErrorModal onCancel={handleErrorContinue} onContinue={handleErrorContinue} />
    );
  }

  // Claim Form View
  if (viewState === "form") {
    return <ClaimForm onSubmit={handleSubmitClaim} />;
  }

  // Plan Summary View
  if (viewState === "summary") {
    return (
      <PlanSummary
        data={MOCK_PLAN_SUMMARY}
        onBack={() => setViewState("list")}
        onWithdraw={() => {
          // Handle withdraw action
          console.log("Withdraw clicked");
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
          console.log("Filter clicked");
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
