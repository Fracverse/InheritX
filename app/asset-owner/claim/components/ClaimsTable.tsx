"use client";

import React from "react";
import StatusBadge from "./StatusBadge";

interface Claim {
  id: string;
  planName: string;
  uniqueId: string;
  assets: string;
  beneficiaryCount: number;
  trigger: string;
  status: string;
}

interface ClaimsTableProps {
  claims: Claim[];
  onClaimClick: (claimId: string) => void;
}

export default function ClaimsTable({ claims, onClaimClick }: ClaimsTableProps) {
  return (
    <div className="bg-[#1C252A] rounded-2xl overflow-hidden">
      {/* Desktop Table View */}
      <div className="hidden md:block overflow-x-auto">
        {/* Table Header */}
        <div className="grid grid-cols-[1.5fr_1fr_1fr_1.5fr_1fr_1.5fr] gap-4 text-sm text-[#92A5A8] pb-4 pt-6 px-6 border-b border-[#2A3338] min-w-200">
          <div>Plan Name/ ID</div>
          <div>Assets</div>
          <div>Beneficiary</div>
          <div>Trigger</div>
          <div>Status</div>
          <div>Action</div>
        </div>

        {/* Table Rows */}
        <div className="min-w-200">
          {claims.map((claim, index) => (
            <div
              key={claim.id}
              className="grid grid-cols-[1.5fr_1fr_1fr_1.5fr_1fr_1.5fr] gap-4 py-6 px-6 border-b border-[#2A3338] items-center hover:bg-[#161E22]/50 transition-colors"
            >
              {/* Plan Name/ID */}
              <div className="flex items-center gap-3">
                <span className="text-[#92A5A8] text-sm">{index + 1}.</span>
                <div>
                  <div className="text-[#FCFFFF] font-medium">{claim.planName}</div>
                  <div className="text-xs text-[#92A5A8]">{claim.uniqueId}</div>
                </div>
              </div>

              {/* Assets */}
              <div className="text-[#FCFFFF]">{claim.assets}</div>

              {/* Beneficiary Count */}
              <div className="text-[#FCFFFF]">{claim.beneficiaryCount}</div>

              {/* Trigger */}
              <div>
                <StatusBadge status={claim.trigger} type="trigger" />
              </div>

              {/* Status */}
              <div>
                <StatusBadge status={claim.status} type="status" />
              </div>

              {/* Actions */}
              <div>
                <button
                  onClick={() => onClaimClick(claim.id)}
                  className="bg-[#33C5E0] text-[#161E22] px-8 py-2 rounded-3xl text-sm font-medium hover:bg-[#2AB8D3] transition-colors"
                >
                  CLAIM PLAN
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Mobile Card View */}
      <div className="md:hidden space-y-4 p-4">
        {claims.map((claim, index) => (
          <div key={claim.id} className="bg-[#161E22] rounded-xl p-4 space-y-3">
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-3">
                <span className="text-[#92A5A8] text-sm">{index + 1}.</span>
                <div>
                  <div className="text-[#FCFFFF] font-medium">{claim.planName}</div>
                  <div className="text-xs text-[#92A5A8]">{claim.uniqueId}</div>
                </div>
              </div>
              <StatusBadge status={claim.status} type="status" />
            </div>

            <div className="grid grid-cols-2 gap-3 text-sm">
              <div>
                <span className="text-[#92A5A8]">Assets:</span>
                <span className="text-[#FCFFFF] ml-2">{claim.assets}</span>
              </div>
              <div>
                <span className="text-[#92A5A8]">Beneficiary:</span>
                <span className="text-[#FCFFFF] ml-2">{claim.beneficiaryCount}</span>
              </div>
            </div>

            <div>
              <StatusBadge status={claim.trigger} type="trigger" />
            </div>

            <button
              onClick={() => onClaimClick(claim.id)}
              className="w-full bg-[#33C5E0] text-[#161E22] py-3 rounded-lg text-sm font-medium hover:bg-[#2AB8D3] transition-colors"
            >
              CLAIM PLAN
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
