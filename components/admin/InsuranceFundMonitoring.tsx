"use client";

import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { Shield, TrendingUp, AlertTriangle, DollarSign, Activity, Clock, CheckCircle, XCircle } from "lucide-react";
import Link from "next/link";

interface InsuranceFundMetrics {
    fund_id: string;
    fund_name: string;
    total_reserves: number;
    available_reserves: number;
    locked_reserves: number;
    total_covered_liabilities: number;
    coverage_ratio: number;
    reserve_health_score: number;
    status: string;
    coverage_ratio_percentage: number;
    health_status_description: string;
    recorded_at: string;
}

interface InsuranceFundTransaction {
    id: string;
    transaction_type: string;
    amount: number;
    asset_code: string;
    balance_after: number;
    created_at: string;
}

interface InsuranceClaim {
    id: string;
    claim_type: string;
    claimed_amount: number;
    status: string;
    created_at: string;
}

interface FundTrends {
    coverage_ratio_change_24h: number | null;
    reserves_change_24h: number | null;
    claims_last_7_days: number;
    payouts_last_7_days: number;
}

interface InsuranceFundDashboard {
    fund: InsuranceFundMetrics;
    recent_transactions: InsuranceFundTransaction[];
    pending_claims: InsuranceClaim[];
    total_claims_count: number;
    total_claims_amount: number;
    trends: FundTrends;
}

const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
        case "healthy":
            return { color: "#48BB78", bg: "rgba(72, 187, 120, 0.1)" };
        case "warning":
            return { color: "#ECC94B", bg: "rgba(236, 201, 75, 0.1)" };
        case "critical":
            return { color: "#F56565", bg: "rgba(245, 101, 101, 0.1)" };
        case "insolvent":
            return { color: "#9F7AEA", bg: "rgba(159, 119, 232, 0.1)" };
        default:
            return { color: "#8899A6", bg: "rgba(136, 153, 166, 0.1)" };
    }
};

const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
        case "healthy":
            return CheckCircle;
        case "warning":
        case "critical":
            return AlertTriangle;
        default:
            return Shield;
    }
};

const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat("en-US", {
        style: "currency",
        currency: "USD",
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
    }).format(amount);
};

const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
    });
};

export function InsuranceFundMonitoring() {
    const [dashboard, setDashboard] = useState<InsuranceFundDashboard | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchDashboard = async () => {
            try {
                const token = localStorage.getItem("adminToken");
                const response = await fetch("/api/admin/insurance-fund", {
                    headers: {
                        Authorization: `Bearer ${token}`,
                        "Content-Type": "application/json",
                    },
                });

                if (!response.ok) {
                    if (response.status === 401) {
                        setError("Please log in to view insurance fund data");
                        return;
                    }
                    throw new Error("Failed to fetch insurance fund data");
                }

                const data = await response.json();
                setDashboard(data.data);
            } catch (err) {
                setError(err instanceof Error ? err.message : "An error occurred");
            } finally {
                setLoading(false);
            }
        };

        fetchDashboard();
    }, []);

    if (loading) {
        return (
            <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6 flex items-center justify-center min-h-[400px]">
                <div className="text-[#8899A6]">Loading insurance fund data...</div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6">
                <div className="text-[#F56565]">{error}</div>
            </div>
        );
    }

    if (!dashboard) {
        return (
            <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6">
                <div className="text-[#8899A6]">No insurance fund data available</div>
            </div>
        );
    }

    const { fund, recent_transactions, pending_claims, trends } = dashboard;
    const statusColors = getStatusColor(fund.status);
    const StatusIcon = getStatusIcon(fund.status);

    return (
        <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6 flex flex-col gap-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <Shield size={24} color="#33C5E0" />
                    <div>
                        <h3 className="font-bold text-white text-lg">{fund.fund_name}</h3>
                        <p className="text-[#8899A6] text-xs">Insurance Fund Monitoring</p>
                    </div>
                </div>
                <Link href="/admin/insurance-fund" className="text-[#33C5E0] text-xs font-semibold hover:underline">
                    View Details
                </Link>
            </div>

            {/* Status Badge */}
            <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                className="flex items-center gap-3 p-4 rounded-xl border border-[#161E22]"
                style={{ backgroundColor: statusColors.bg }}
            >
                <StatusIcon size={24} color={statusColors.color} />
                <div className="flex-1">
                    <p className="text-xs text-[#8899A6] font-medium uppercase tracking-wider">Fund Status</p>
                    <p className="text-lg font-bold capitalize" style={{ color: statusColors.color }}>
                        {fund.status}
                    </p>
                </div>
                <div className="text-right">
                    <p className="text-xs text-[#8899A6] font-medium">Health Score</p>
                    <p className="text-2xl font-bold text-white">{fund.reserve_health_score.toFixed(0)}/100</p>
                </div>
            </motion.div>

            {/* Key Metrics Grid */}
            <div className="grid grid-cols-2 gap-4">
                {/* Total Reserves */}
                <div className="p-4 rounded-xl border border-[#161E22] bg-[#0D1215]">
                    <div className="flex items-center gap-2 mb-2">
                        <DollarSign size={16} color="#48BB78" />
                        <span className="text-[10px] text-[#8899A6] font-medium uppercase">Total Reserves</span>
                    </div>
                    <p className="text-xl font-bold text-white">{formatCurrency(fund.total_reserves)}</p>
                    {trends.reserves_change_24h !== null && (
                        <p className={`text-xs mt-1 ${trends.reserves_change_24h! >= 0 ? "text-[#48BB78]" : "text-[#F56565]"}`}>
                            {trends.reserves_change_24h! >= 0 ? "↑" : "↓"} {formatCurrency(Math.abs(trends.reserves_change_24h!))} (24h)
                        </p>
                    )}
                </div>

                {/* Coverage Ratio */}
                <div className="p-4 rounded-xl border border-[#161E22] bg-[#0D1215]">
                    <div className="flex items-center gap-2 mb-2">
                        <TrendingUp size={16} color="#33C5E0" />
                        <span className="text-[10px] text-[#8899A6] font-medium uppercase">Coverage Ratio</span>
                    </div>
                    <p className="text-xl font-bold text-white">{fund.coverage_ratio.toFixed(2)}x</p>
                    {trends.coverage_ratio_change_24h !== null && (
                        <p className={`text-xs mt-1 ${trends.coverage_ratio_change_24h! >= 0 ? "text-[#48BB78]" : "text-[#F56565]"}`}>
                            {trends.coverage_ratio_change_24h! >= 0 ? "↑" : "↓"} {Math.abs(trends.coverage_ratio_change_24h!).toFixed(2)} (24h)
                        </p>
                    )}
                </div>

                {/* Covered Liabilities */}
                <div className="p-4 rounded-xl border border-[#161E22] bg-[#0D1215]">
                    <div className="flex items-center gap-2 mb-2">
                        <Activity size={16} color="#ECC94B" />
                        <span className="text-[10px] text-[#8899A6] font-medium uppercase">Covered Liabilities</span>
                    </div>
                    <p className="text-xl font-bold text-white">{formatCurrency(fund.total_covered_liabilities)}</p>
                </div>

                {/* Available Reserves */}
                <div className="p-4 rounded-xl border border-[#161E22] bg-[#0D1215]">
                    <div className="flex items-center gap-2 mb-2">
                        <Shield size={16} color="#9F7AEA" />
                        <span className="text-[10px] text-[#8899A6] font-medium uppercase">Available Reserves</span>
                    </div>
                    <p className="text-xl font-bold text-white">{formatCurrency(fund.available_reserves)}</p>
                </div>
            </div>

            {/* Recent Transactions */}
            <div className="border-t border-[#161E22] pt-4">
                <div className="flex items-center justify-between mb-3">
                    <h4 className="font-bold text-white text-sm">Recent Transactions</h4>
                    <Clock size={14} className="text-[#8899A6]" />
                </div>
                {recent_transactions.length === 0 ? (
                    <p className="text-[#8899A6] text-sm">No recent transactions</p>
                ) : (
                    <div className="space-y-2 max-h-40 overflow-y-auto">
                        {recent_transactions.slice(0, 5).map((tx) => (
                            <div key={tx.id} className="flex items-center justify-between text-xs p-2 rounded-lg bg-[#0D1215]">
                                <div className="flex items-center gap-2">
                                    <span className={`px-2 py-0.5 rounded text-[10px] font-medium uppercase ${
                                        tx.transaction_type === "contribution" || tx.transaction_type === "yield"
                                            ? "bg-[#48BB78]/20 text-[#48BB78]"
                                            : "bg-[#F56565]/20 text-[#F56565]"
                                    }`}>
                                        {tx.transaction_type}
                                    </span>
                                    <span className="text-[#8899A6]">{formatDate(tx.created_at)}</span>
                                </div>
                                <span className={`font-bold ${
                                    tx.transaction_type === "contribution" || tx.transaction_type === "yield"
                                        ? "text-[#48BB78]"
                                        : "text-[#F56565]"
                                }`}>
                                    {tx.transaction_type === "contribution" || tx.transaction_type === "yield" ? "+" : "-"}
                                    {formatCurrency(tx.amount)} {tx.asset_code}
                                </span>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            {/* Pending Claims */}
            <div className="border-t border-[#161E22] pt-4">
                <div className="flex items-center justify-between mb-3">
                    <h4 className="font-bold text-white text-sm">Pending Claims</h4>
                    {pending_claims.length > 0 && (
                        <span className="text-xs bg-[#ECC94B]/20 text-[#ECC94B] px-2 py-0.5 rounded-full font-bold">
                            {pending_claims.length}
                        </span>
                    )}
                </div>
                {pending_claims.length === 0 ? (
                    <p className="text-[#8899A6] text-sm">No pending claims</p>
                ) : (
                    <div className="space-y-2 max-h-40 overflow-y-auto">
                        {pending_claims.slice(0, 3).map((claim) => (
                            <div key={claim.id} className="flex items-center justify-between text-xs p-2 rounded-lg bg-[#0D1215]">
                                <div className="flex items-center gap-2">
                                    <span className="px-2 py-0.5 rounded text-[10px] font-medium uppercase bg-[#ECC94B]/20 text-[#ECC94B]">
                                        {claim.claim_type}
                                    </span>
                                    <span className="text-[#8899A6]">{formatDate(claim.created_at)}</span>
                                </div>
                                <span className="font-bold text-[#F56565]">
                                    {formatCurrency(claim.claimed_amount)}
                                </span>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            {/* Summary Stats */}
            <div className="pt-4 border-t border-[#161E22]">
                <div className="grid grid-cols-3 gap-4">
                    <div className="text-center">
                        <p className="text-2xl font-bold text-white">{trends.claims_last_7_days}</p>
                        <p className="text-[10px] text-[#8899A6] font-medium uppercase">Claims (7d)</p>
                    </div>
                    <div className="text-center">
                        <p className="text-2xl font-bold text-white">{formatCurrency(trends.payouts_last_7_days)}</p>
                        <p className="text-[10px] text-[#8899A6] font-medium uppercase">Payouts (7d)</p>
                    </div>
                    <div className="text-center">
                        <p className="text-2xl font-bold text-white">{fund.coverage_ratio.toFixed(2)}x</p>
                        <p className="text-[10px] text-[#8899A6] font-medium uppercase">Current Ratio</p>
                    </div>
                </div>
            </div>
        </div>
    );
}
