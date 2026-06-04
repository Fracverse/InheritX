"use client";

import React from "react";
import Link from "next/link";
import { ArrowLeft, ExternalLink, Copy, Clock, CheckCircle } from "lucide-react";

// Static mock plan data
const MOCK_PLAN = {
    id: "001",
    planName: "Family Inheritance Plan",
    planDescription:
        "This plan is designed to transfer assets to my children upon the specified conditions being met. The assets will be distributed according to the allocation percentages.",
    assetType: "ETH",
    assetAmount: "2.0",
    createdAt: "2025-08-12T10:00:00Z",
    transferDate: "2026-01-01T00:00:00Z",
    distributionMethod: "LUMP_SUM",
    status: "ACTIVE",
    globalPlanId: "GP-001",
    userPlanId: "UP-001",
    txHash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    beneficiaries: [
        {
            id: "1",
            name: "John Doe",
            email: "john@example.com",
            relationship: "Son",
            allocatedPercentage: 6000,
            hasClaimed: false,
        },
        {
            id: "2",
            name: "Jane Doe",
            email: "jane@example.com",
            relationship: "Daughter",
            allocatedPercentage: 4000,
            hasClaimed: true,
        },
    ],
    distributions: [
        {
            periodNumber: 1,
            scheduledDate: "2026-01-01T00:00:00Z",
            amount: "2.0",
            status: "PENDING",
        },
    ],
};

const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString("en-US", {
        year: "numeric",
        month: "long",
        day: "numeric",
    });
};

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

export default function PlanDetailsPage() {
    const plan = MOCK_PLAN;

    const copyClaimCode = () => {
        navigator.clipboard.writeText("ABC123");
        alert("Claim code copied to clipboard!");
    };

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <Link
                        href="/asset-owner/plans"
                        className="w-10 h-10 rounded-full bg-[#33C5E0] flex items-center justify-center text-[#161E22] hover:bg-[#2AB8D3] transition-colors"
                    >
                        <ArrowLeft size={20} />
                    </Link>
                    <div>
                        <h1 className="text-2xl font-semibold text-[#FCFFFF]">
                            {plan.planName}
                        </h1>
                        <p className="text-sm text-[#92A5A8] mt-1">
                            Created {formatDate(plan.createdAt)}
                        </p>
                    </div>
                </div>
                <div className="flex items-center gap-3">
                    <span className={`px-3 py-1.5 rounded-full text-xs ${getStatusStyle(plan.status)}`}>
                        {plan.status}
                    </span>
                    {plan.txHash && (
                        <a
                            href={`https://sepolia-blockscout.lisk.com/tx/${plan.txHash}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-[#92A5A8] hover:text-[#FCFFFF] transition-colors"
                        >
                            <ExternalLink size={18} />
                        </a>
                    )}
                </div>
            </div>

            <div className="grid gap-6 md:grid-cols-2">
                {/* Plan Overview */}
                <div className="bg-[#182024] rounded-2xl p-6">
                    <h2 className="text-lg font-semibold text-[#FCFFFF] mb-4 flex items-center gap-2">
                        <span className="text-[#33C5E0]">$</span>
                        Plan Overview
                    </h2>
                    <div className="space-y-3">
                        <div>
                            <label className="text-xs font-medium text-[#92A5A8]">
                                Description
                            </label>
                            <p className="mt-1 text-sm text-[#FCFFFF]">
                                {plan.planDescription}
                            </p>
                        </div>
                        <div className="grid grid-cols-2 gap-4 pt-3 border-t border-[#1C252A]">
                            <div>
                                <label className="text-xs font-medium text-[#92A5A8]">
                                    Asset Type
                                </label>
                                <p className="mt-1 font-medium text-[#FCFFFF]">
                                    {plan.assetType}
                                </p>
                            </div>
                            <div>
                                <label className="text-xs font-medium text-[#92A5A8]">
                                    Amount
                                </label>
                                <p className="mt-1 font-medium text-[#FCFFFF]">
                                    {plan.assetAmount} {plan.assetType}
                                </p>
                            </div>
                        </div>
                        <div className="pt-3 border-t border-[#1C252A]">
                            <div className="flex justify-between text-sm">
                                <span className="text-[#92A5A8]">Creation Fee (5%)</span>
                                <span className="text-[#FCFFFF]">0.1 {plan.assetType}</span>
                            </div>
                            <div className="flex justify-between text-sm font-medium mt-2 pt-2 border-t border-[#1C252A]">
                                <span className="text-[#FCFFFF]">Total Locked</span>
                                <span className="text-[#FCFFFF]">2.1 {plan.assetType}</span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Distribution Details */}
                <div className="bg-[#182024] rounded-2xl p-6">
                    <h2 className="text-lg font-semibold text-[#FCFFFF] mb-4 flex items-center gap-2">
                        <span className="text-[#33C5E0]">📅</span>
                        Distribution Details
                    </h2>
                    <div className="space-y-3">
                        <div>
                            <label className="text-xs font-medium text-[#92A5A8]">
                                Method
                            </label>
                            <p className="mt-1 font-medium text-[#FCFFFF]">
                                {plan.distributionMethod.replace("_", " ")}
                            </p>
                        </div>
                        <div>
                            <label className="text-xs font-medium text-[#92A5A8]">
                                Transfer Date
                            </label>
                            <p className="mt-1 font-medium text-[#FCFFFF] flex items-center gap-2">
                                <Clock size={14} className="text-[#33C5E0]" />
                                {formatDate(plan.transferDate)}
                            </p>
                        </div>
                        <div className="pt-3 border-t border-[#1C252A]">
                            <div className="grid grid-cols-2 gap-4 text-sm">
                                <div>
                                    <label className="text-xs font-medium text-[#92A5A8]">
                                        Global Plan ID
                                    </label>
                                    <p className="mt-1 font-mono text-[#FCFFFF]">
                                        {plan.globalPlanId}
                                    </p>
                                </div>
                                <div>
                                    <label className="text-xs font-medium text-[#92A5A8]">
                                        User Plan ID
                                    </label>
                                    <p className="mt-1 font-mono text-[#FCFFFF]">
                                        {plan.userPlanId}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Beneficiaries */}
                <div className="bg-[#182024] rounded-2xl p-6">
                    <h2 className="text-lg font-semibold text-[#FCFFFF] mb-4 flex items-center gap-2">
                        <span className="text-[#33C5E0]">👥</span>
                        Beneficiaries ({plan.beneficiaries.length})
                    </h2>
                    <div className="space-y-3">
                        {plan.beneficiaries.map((ben) => (
                            <div
                                key={ben.id}
                                className="p-3 bg-[#161E22] rounded-lg border border-[#1C252A]"
                            >
                                <div className="flex items-start justify-between">
                                    <div className="flex-1">
                                        <div className="flex items-center gap-2 mb-1">
                                            <div className="w-8 h-8 rounded-full bg-gradient-to-br from-[#33C5E0] to-[#8B5CF6] flex items-center justify-center text-xs font-semibold text-white">
                                                {ben.name.charAt(0).toUpperCase()}
                                            </div>
                                            <div>
                                                <p className="font-medium text-[#FCFFFF]">{ben.name}</p>
                                                <p className="text-xs text-[#92A5A8]">{ben.email}</p>
                                            </div>
                                        </div>
                                        <p className="text-xs text-[#92A5A8] mt-2">
                                            Relationship: {ben.relationship}
                                        </p>
                                    </div>
                                    <div className="text-right">
                                        <p className="font-semibold text-[#33C5E0]">
                                            {(ben.allocatedPercentage / 100).toFixed(1)}%
                                        </p>
                                        <p className="text-xs text-[#92A5A8]">
                                            {(
                                                (parseFloat(plan.assetAmount) * ben.allocatedPercentage) /
                                                10000
                                            ).toFixed(2)}{" "}
                                            {plan.assetType}
                                        </p>
                                    </div>
                                </div>
                                <div className="mt-3 pt-3 border-t border-[#1C252A] flex items-center justify-between text-xs">
                                    <span className="text-[#92A5A8]">Claim Status</span>
                                    {ben.hasClaimed ? (
                                        <span className="flex items-center gap-1 text-green-400">
                                            <CheckCircle size={14} />
                                            Claimed
                                        </span>
                                    ) : (
                                        <span className="flex items-center gap-1 text-[#92A5A8]">
                                            <Clock size={14} />
                                            Pending
                                        </span>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* Actions */}
                <div className="bg-[#182024] rounded-2xl p-6">
                    <h2 className="text-lg font-semibold text-[#FCFFFF] mb-4">Actions</h2>
                    <div className="space-y-3">
                        {/* Claim Code */}
                        <div className="p-4 bg-[#161E22] rounded-lg border border-[#1C252A]">
                            <div className="flex items-center justify-between mb-2">
                                <label className="text-sm font-medium text-[#FCFFFF]">
                                    Claim Code
                                </label>
                                <button
                                    onClick={copyClaimCode}
                                    className="text-[#92A5A8] hover:text-[#FCFFFF] transition-colors"
                                >
                                    <Copy size={16} />
                                </button>
                            </div>
                            <p className="font-mono text-lg font-bold text-[#33C5E0]">
                                ABC123
                            </p>
                        </div>

                        {/* Distribution Schedule */}
                        <div className="pt-4 border-t border-[#1C252A]">
                            <h3 className="text-sm font-semibold text-[#FCFFFF] mb-3">
                                Distribution Schedule
                            </h3>
                            <div className="space-y-2">
                                {plan.distributions.map((dist) => (
                                    <div
                                        key={dist.periodNumber}
                                        className="flex items-center justify-between p-2 bg-[#161E22] rounded text-xs"
                                    >
                                        <div>
                                            <span className="font-medium text-[#FCFFFF]">
                                                Period {dist.periodNumber}
                                            </span>
                                            <span className="text-[#92A5A8] ml-2">
                                                {formatDate(dist.scheduledDate)}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-2">
                                            <span className="font-medium text-[#FCFFFF]">
                                                {dist.amount} {plan.assetType}
                                            </span>
                                            {dist.status === "EXECUTED" ? (
                                                <CheckCircle className="text-green-400" size={14} />
                                            ) : (
                                                <Clock className="text-[#92A5A8]" size={14} />
                                            )}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
