"use client";

import React, { useState } from "react";
import Link from "next/link";
import { Plus, SlidersHorizontal, Trash2 } from "lucide-react";
import CreatePlanModal from "./components/CreatePlanModal";

// Static mock data for plans
const MOCK_PLANS = [
  {
    id: "001",
    name: "Plan Name",
    uniqueId: "Unique ID",
    assets: "2 ETH",
    assetIcon: null,
    beneficiaryCount: 3,
    trigger: "INACTIVITY (6 MONTHS)",
    status: "ACTIVE",
  },
  {
    id: "002",
    name: "Plan Name",
    uniqueId: "Unique ID",
    assets: "7 NFTs",
    assetIcon: "nft",
    beneficiaryCount: 1,
    trigger: "TIME-LOCKED",
    status: "COMPLETED",
  },
  {
    id: "003",
    name: "Plan Name",
    uniqueId: "Unique ID",
    assets: "1 NFT",
    assetIcon: "eth",
    beneficiaryCount: 2,
    trigger: "INACTIVITY (6 MONTHS)",
    status: "PENDING",
  },
  {
    id: "004",
    name: "Plan Name",
    uniqueId: "Unique ID",
    assets: "1 BTC",
    assetIcon: null,
    beneficiaryCount: 1,
    trigger: "INACTIVITY (6 MONTHS)",
    status: "EXPIRED",
  },
];

// Static mock data for activities
const MOCK_ACTIVITIES = [
  {
    id: 1,
    description: "Plan #001 Created (3 Beneficiaries, Inactivity Trigger Set)",
    timestamp: "12th August, 2025",
  },
  {
    id: 2,
    description: "Guardian Added To Plan #002",
    timestamp: "12th August, 2025",
  },
  {
    id: 3,
    description: "Plan #001 Status Changed To Active",
    timestamp: "12th August, 2025",
  },
  {
    id: 4,
    description: "1 NFC Converted",
    timestamp: "12th August, 2025",
  },
];

const getStatusStyle = (status: string) => {
  switch (status) {
    case "ACTIVE":
      return "bg-transparent border border-[#33C5E0] text-[#33C5E0]";
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

const getTriggerStyle = (trigger: string) => {
  if (trigger.includes("INACTIVITY")) {
    return "bg-[#33C5E0]/20 text-[#33C5E0]";
  }
  return "bg-[#1C252A] text-[#92A5A8]";
};

export default function PlansPage() {
  const [activeTab, setActiveTab] = useState<"plans" | "activities">("plans");
  const [showCreateModal, setShowCreateModal] = useState(false);

  return (
    <div>
      {/* Header */}
      <div className="flex justify-between items-start mb-8">
        <div>
          <h1 className="text-2xl font-semibold text-[#FCFFFF] mb-1">
            Your Plans
          </h1>
          <p className="text-sm text-[#92A5A8]">
            Create, edit, and publish your inheritance plans
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-x-2 bg-transparent border border-[#33C5E0] text-[#33C5E0] py-3 px-5 rounded-full hover:bg-[#33C5E0]/10 transition-colors"
        >
          <Plus size={18} />
          Create New Plan
        </button>
      </div>

      {/* Tabs and Filter */}
      <div className="flex justify-between items-center mb-8">
        <div className="flex">
          <button
            onClick={() => setActiveTab("plans")}
            className={`py-2 px-4 text-sm font-medium transition-colors ${activeTab === "plans"
                ? "text-[#33C5E0] border-b-2 border-[#33C5E0]"
                : "text-[#92A5A8] hover:text-[#FCFFFF]"
              }`}
          >
            Plans
          </button>
          <button
            onClick={() => setActiveTab("activities")}
            className={`py-2 px-4 text-sm font-medium transition-colors ${activeTab === "activities"
                ? "text-[#33C5E0] border-b-2 border-[#33C5E0]"
                : "text-[#92A5A8] hover:text-[#FCFFFF]"
              }`}
          >
            Activities
          </button>
        </div>
        <button className="flex items-center gap-x-2 text-[#92A5A8] hover:text-[#FCFFFF] transition-colors">
          <SlidersHorizontal size={16} />
          Filter
        </button>
      </div>

      {/* Plans Tab Content */}
      {activeTab === "plans" && (
        <div className="overflow-x-auto">
          {/* Table Header */}
          <div className="grid grid-cols-[1.5fr_1fr_1fr_1.5fr_1fr_1.5fr] gap-4 text-sm text-[#92A5A8] pb-4 border-b border-[#1C252A] min-w-[800px]">
            <div>Plan Name/ ID</div>
            <div>Assets</div>
            <div>Beneficiary</div>
            <div>Trigger</div>
            <div>Status</div>
            <div>Action</div>
          </div>

          {/* Table Rows */}
          <div className="min-w-[800px]">
            {MOCK_PLANS.map((plan, index) => (
              <div
                key={plan.id}
                className="grid grid-cols-[1.5fr_1fr_1fr_1.5fr_1fr_1.5fr] gap-4 py-6 border-b border-[#1C252A] items-center"
              >
                {/* Plan Name/ID */}
                <div className="flex items-center gap-3">
                  <span className="text-[#92A5A8] text-sm">{index + 1}.</span>
                  <div>
                    <div className="text-[#FCFFFF] font-medium">{plan.name}</div>
                    <div className="text-xs text-[#92A5A8]">{plan.uniqueId}</div>
                  </div>
                </div>

                {/* Assets */}
                <div className="flex items-center gap-2 text-[#FCFFFF]">
                  {plan.assets}
                  {plan.assetIcon === "nft" && (
                    <span className="flex items-center gap-1 bg-[#33C5E0] text-[#161E22] text-xs px-2 py-0.5 rounded-full">
                      <span className="flex -space-x-1">
                        <span className="w-3 h-3 rounded-full bg-[#161E22]" />
                        <span className="w-3 h-3 rounded-full bg-[#1C252A]" />
                        <span className="w-3 h-3 rounded-full bg-[#2A3338]" />
                      </span>
                      3+
                    </span>
                  )}
                  {plan.assetIcon === "eth" && (
                    <span className="w-5 h-5 rounded-full bg-[#627EEA] flex items-center justify-center text-xs text-white">
                      Ξ
                    </span>
                  )}
                </div>

                {/* Beneficiary Count */}
                <div className="text-[#FCFFFF]">{plan.beneficiaryCount}</div>

                {/* Trigger */}
                <div>
                  <span
                    className={`px-3 py-1.5 rounded-full text-xs ${getTriggerStyle(plan.trigger)}`}
                  >
                    {plan.trigger}
                  </span>
                </div>

                {/* Status */}
                <div>
                  <span
                    className={`px-3 py-1.5 rounded-full text-xs ${getStatusStyle(plan.status)}`}
                  >
                    {plan.status}
                  </span>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2">
                  <button className="bg-[#1C252A] text-[#92A5A8] px-4 py-2 rounded-lg text-sm hover:bg-[#2A3338] transition-colors">
                    EDIT
                  </button>
                  <Link
                    href={`/asset-owner/plans/${plan.id}`}
                    className="bg-[#33C5E0] text-[#161E22] px-4 py-2 rounded-lg text-sm hover:bg-[#2AB8D3] transition-colors"
                  >
                    VIEW
                  </Link>
                  <button className="text-[#92A5A8] p-2 hover:text-red-400 transition-colors">
                    <Trash2 size={18} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Activities Tab Content */}
      {activeTab === "activities" && (
        <div>
          {/* This Month Section */}
          <div className="mb-6">
            <h3 className="text-sm text-[#92A5A8] mb-4">This Month</h3>

            {/* Activities Header */}
            <div className="grid grid-cols-[auto_1fr] gap-4 pb-4">
              <div className="w-8" />
              <div className="flex justify-between text-sm text-[#92A5A8]">
                <span></span>
                <span>Timestamp</span>
              </div>
            </div>

            {/* Activity Items */}
            <div className="space-y-4">
              {MOCK_ACTIVITIES.map((activity, index) => (
                <div
                  key={activity.id}
                  className="grid grid-cols-[auto_1fr] gap-4 py-4 border-b border-[#1C252A] items-center"
                >
                  <span className="text-[#92A5A8] text-sm w-8">{index + 1}.</span>
                  <div className="flex justify-between items-center">
                    <span className="text-[#FCFFFF] font-medium">
                      {activity.description}
                    </span>
                    <span className="text-[#92A5A8] text-sm">
                      {activity.timestamp}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Create Plan Modal */}
      {showCreateModal && (
        <CreatePlanModal onClose={() => setShowCreateModal(false)} />
      )}
    </div>
  );
}
