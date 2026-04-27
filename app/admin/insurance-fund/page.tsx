"use client";

import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { Shield, TrendingUp, DollarSign, Activity, Clock, ArrowLeft, Download, RefreshCw } from "lucide-react";
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
    fund_id: string;
    transaction_type: string;
    user_id: string | null;
    plan_id: string | null;
    loan_id: string | null;
    asset_code: string;
    amount: number;
    balance_after: number;
    description: string | null;
    created_at: string;
}

interface InsuranceClaim {
    id: string;
    fund_id: string;
    user_id: string;
    plan_id: string | null;
    loan_id: string | null;
    claim_type: string;
    claimed_amount: number;
    approved_amount: number | null;
    payout_amount: number | null;
    status: string;
    rejection_reason: string | null;
    reviewed_by: string | null;
    reviewed_at: string | null;
    paid_at: string | null;
    created_at: string;
    updated_at: string;
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
            return { color: "#48BB78", bg: "rgba(72, 187, 120, 0.1)", border: "#48BB78" };
        case "warning":
            return { color: "#ECC94B", bg: "rgba(236, 201, 75, 0.1)", border: "#ECC94B" };
        case "critical":
            return { color: "#F56565", bg: "rgba(245, 101, 101, 0.1)", border: "#F56565" };
        case "insolvent":
            return { color: "#9F7AEA", bg: "rgba(159, 119, 232, 0.1)", border: "#9F7AEA" };
        default:
            return { color: "#8899A6", bg: "rgba(136, 153, 166, 0.1)", border: "#8899A6" };
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
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
    });
};

export default function InsuranceFundPage() {
    const [dashboard, setDashboard] = useState<InsuranceFundDashboard | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState<"overview" | "transactions" | "claims">("overview");
    const [timeRange, setTimeRange] = useState(30);

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

    useEffect(() => {
        fetchDashboard();
    }, []);

    const handleRefresh = () => {
        setLoading(true);
        fetchDashboard();
    };

    if (error) {
        return (
            <div className="space-y-6">
                <Link href="/admin" className="inline-flex items-center gap-2 text-[#33C5E0] hover:underline">
                    <ArrowLeft size={16} />
                    Back to Admin Dashboard
                </Link>
                <div className="bg-[#F56565]/10 border border-[#F56565] rounded-2xl p-6">
                    <p className="text-[#F56565]">{error}</p>
                </div>
            </div>
        );
    }

    if (!dashboard) {
        return (
            <div className="space-y-6">
                <Link href="/admin" className="inline-flex items-center gap-2 text-[#33C5E0] hover:underline">
                    <ArrowLeft size={16} />
                    Back to Admin Dashboard
                </Link>
                <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-12 text-center">
                    <p className="text-[#8899A6]">Loading insurance fund data...</p>
                </div>
            </div>
        );
    }

    const { fund, recent_transactions, pending_claims, total_claims_count, total_claims_amount, trends } = dashboard;
    const statusColors = getStatusColor(fund.status);

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <Link href="/admin" className="inline-flex items-center gap-2 text-[#33C5E0] hover:underline">
                    <ArrowLeft size={16} />
                    Back to Admin Dashboard
                </Link>
                <button
                    onClick={handleRefresh}
                    className="flex items-center gap-2 px-4 py-2 bg-[#33C5E0]/10 border border-[#33C5E0] rounded-lg text-[#33C5E0] hover:bg-[#33C5E0]/20 transition-colors"
                >
                    <RefreshCw size={16} />
                    Refresh
                </button>
            </div>

            {/* Page Title */}
            <motion.div
                initial={{ opacity: 0, y: -20 }}
                animate={{ opacity: 1, y: 0 }}
                className="flex items-center gap-3"
            >
                <Shield size={32} color="#33C5E0" />
                <div>
                    <h1 className="text-3xl font-bold text-white">{fund.fund_name}</h1>
                    <p className="text-[#8899A6]">Insurance Fund Monitoring & Management</p>
                </div>
            </motion.div>

            {/* Status Banner */}
            <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.1 }}
                className="p-6 rounded-2xl border-2"
                style={{ backgroundColor: statusColors.bg, borderColor: statusColors.border }}
            >
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <Shield size={48} color={statusColors.color} />
                        <div>
                            <p className="text-sm text-[#8899A6] font-medium uppercase tracking-wider">Current Status</p>
                            <p className="text-3xl font-bold capitalize" style={{ color: statusColors.color }}>
                                {fund.status}
                            </p>
                            <p className="text-sm text-[#8899A6] mt-1">{fund.health_status_description}</p>
                        </div>
                    </div>
                    <div className="text-right">
                        <p className="text-sm text-[#8899A6] font-medium">Reserve Health Score</p>
                        <p className="text-5xl font-bold text-white">{fund.reserve_health_score.toFixed(0)}</p>
                        <p className="text-sm text-[#8899A6]">out of 100</p>
                    </div>
                </div>
            </motion.div>

            {/* Key Metrics Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {/* Total Reserves */}
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.2 }}
                    className="p-6 rounded-2xl border border-[#161E22] bg-[#0D1215]"
                >
                    <div className="flex items-center gap-3 mb-4">
                        <DollarSign size={24} color="#48BB78" />
                        <span className="text-sm text-[#8899A6] font-medium uppercase">Total Reserves</span>
                    </div>
                    <p className="text-3xl font-bold text-white mb-2">{formatCurrency(fund.total_reserves)}</p>
                    <div className="space-y-1">
                        <p className="text-xs text-[#8899A6]">Available: <span className="text-white font-semibold">{formatCurrency(fund.available_reserves)}</span></p>
                        <p className="text-xs text-[#8899A6]">Locked: <span className="text-white font-semibold">{formatCurrency(fund.locked_reserves)}</span></p>
                    </div>
                    {trends.reserves_change_24h !== null && (
                        <p className={`text-xs mt-3 ${trends.reserves_change_24h! >= 0 ? "text-[#48BB78]" : "text-[#F56565]"}`}>
                            {trends.reserves_change_24h! >= 0 ? "↑" : "↓"} {formatCurrency(Math.abs(trends.reserves_change_24h!))} in last 24h
                        </p>
                    )}
                </motion.div>

                {/* Coverage Ratio */}
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.3 }}
                    className="p-6 rounded-2xl border border-[#161E22] bg-[#0D1215]"
                >
                    <div className="flex items-center gap-3 mb-4">
                        <TrendingUp size={24} color="#33C5E0" />
                        <span className="text-sm text-[#8899A6] font-medium uppercase">Coverage Ratio</span>
                    </div>
                    <p className="text-3xl font-bold text-white mb-2">{fund.coverage_ratio.toFixed(2)}x</p>
                    <div className="space-y-1">
                        <p className="text-xs text-[#8899A6]">Target: <span className="text-[#48BB78] font-semibold">1.50x</span></p>
                        <p className="text-xs text-[#8899A6]">Minimum: <span className="text-[#ECC94B] font-semibold">1.00x</span></p>
                    </div>
                    {trends.coverage_ratio_change_24h !== null && (
                        <p className={`text-xs mt-3 ${trends.coverage_ratio_change_24h! >= 0 ? "text-[#48BB78]" : "text-[#F56565]"}`}>
                            {trends.coverage_ratio_change_24h! >= 0 ? "↑" : "↓"} {Math.abs(trends.coverage_ratio_change_24h!).toFixed(2)} in last 24h
                        </p>
                    )}
                </motion.div>

                {/* Covered Liabilities */}
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.4 }}
                    className="p-6 rounded-2xl border border-[#161E22] bg-[#0D1215]"
                >
                    <div className="flex items-center gap-3 mb-4">
                        <Activity size={24} color="#ECC94B" />
                        <span className="text-sm text-[#8899A6] font-medium uppercase">Covered Liabilities</span>
                    </div>
                    <p className="text-3xl font-bold text-white mb-2">{formatCurrency(fund.total_covered_liabilities)}</p>
                    <div className="space-y-1">
                        <p className="text-xs text-[#8899A6]">Total Claims: <span className="text-white font-semibold">{total_claims_count}</span></p>
                        <p className="text-xs text-[#8899A6]">Total Amount: <span className="text-white font-semibold">{formatCurrency(total_claims_amount)}</span></p>
                    </div>
                </motion.div>

                {/* 7-Day Stats */}
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.5 }}
                    className="p-6 rounded-2xl border border-[#161E22] bg-[#0D1215]"
                >
                    <div className="flex items-center gap-3 mb-4">
                        <Clock size={24} color="#9F7AEA" />
                        <span className="text-sm text-[#8899A6] font-medium uppercase">Last 7 Days</span>
                    </div>
                    <div className="space-y-3">
                        <div>
                            <p className="text-2xl font-bold text-white">{trends.claims_last_7_days}</p>
                            <p className="text-xs text-[#8899A6]">Claims Submitted</p>
                        </div>
                        <div className="pt-2 border-t border-[#161E22]">
                            <p className="text-2xl font-bold text-white">{formatCurrency(trends.payouts_last_7_days)}</p>
                            <p className="text-xs text-[#8899A6]">Total Payouts</p>
                        </div>
                    </div>
                </motion.div>
            </div>

            {/* Tabs */}
            <div className="flex gap-4 border-b border-[#161E22]">
                <button
                    onClick={() => setActiveTab("overview")}
                    className={`px-4 py-2 font-semibold transition-colors ${
                        activeTab === "overview"
                            ? "text-[#33C5E0] border-b-2 border-[#33C5E0]"
                            : "text-[#8899A6] hover:text-white"
                    }`}
                >
                    Overview
                </button>
                <button
                    onClick={() => setActiveTab("transactions")}
                    className={`px-4 py-2 font-semibold transition-colors ${
                        activeTab === "transactions"
                            ? "text-[#33C5E0] border-b-2 border-[#33C5E0]"
                            : "text-[#8899A6] hover:text-white"
                    }`}
                >
                    Transactions
                </button>
                <button
                    onClick={() => setActiveTab("claims")}
                    className={`px-4 py-2 font-semibold transition-colors ${
                        activeTab === "claims"
                            ? "text-[#33C5E0] border-b-2 border-[#33C5E0]"
                            : "text-[#8899A6] hover:text-white"
                    }`}
                >
                    Claims ({pending_claims.length} pending)
                </button>
            </div>

            {/* Tab Content */}
            {activeTab === "overview" && (
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="grid grid-cols-1 lg:grid-cols-2 gap-6"
                >
                    {/* Recent Transactions */}
                    <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6">
                        <h3 className="font-bold text-white text-lg mb-4">Recent Transactions</h3>
                        {recent_transactions.length === 0 ? (
                            <p className="text-[#8899A6]">No recent transactions</p>
                        ) : (
                            <div className="space-y-3">
                                {recent_transactions.slice(0, 10).map((tx) => (
                                    <div key={tx.id} className="flex items-center justify-between p-3 rounded-lg bg-[#0D1215]">
                                        <div>
                                            <span className={`px-2 py-1 rounded text-xs font-medium uppercase ${
                                                tx.transaction_type === "contribution" || tx.transaction_type === "yield"
                                                    ? "bg-[#48BB78]/20 text-[#48BB78]"
                                                    : "bg-[#F56565]/20 text-[#F56565]"
                                            }`}>
                                                {tx.transaction_type}
                                            </span>
                                            <p className="text-xs text-[#8899A6] mt-1">{formatDate(tx.created_at)}</p>
                                        </div>
                                        <div className="text-right">
                                            <p className={`font-bold ${
                                                tx.transaction_type === "contribution" || tx.transaction_type === "yield"
                                                    ? "text-[#48BB78]"
                                                    : "text-[#F56565]"
                                            }`}>
                                                {tx.transaction_type === "contribution" || tx.transaction_type === "yield" ? "+" : "-"}
                                                {formatCurrency(tx.amount)} {tx.asset_code}
                                            </p>
                                            <p className="text-xs text-[#8899A6]">Balance: {formatCurrency(tx.balance_after)}</p>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>

                    {/* Pending Claims */}
                    <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6">
                        <h3 className="font-bold text-white text-lg mb-4">Pending Claims</h3>
                        {pending_claims.length === 0 ? (
                            <p className="text-[#8899A6]">No pending claims</p>
                        ) : (
                            <div className="space-y-3">
                                {pending_claims.map((claim) => (
                                    <div key={claim.id} className="p-3 rounded-lg bg-[#0D1215] border border-[#ECC94B]/20">
                                        <div className="flex items-center justify-between mb-2">
                                            <span className="px-2 py-1 rounded text-xs font-medium uppercase bg-[#ECC94B]/20 text-[#ECC94B]">
                                                {claim.claim_type}
                                            </span>
                                            <span className="text-xs text-[#8899A6]">{formatDate(claim.created_at)}</span>
                                        </div>
                                        <p className="text-lg font-bold text-[#F56565]">{formatCurrency(claim.claimed_amount)}</p>
                                        {claim.plan_id && (
                                            <p className="text-xs text-[#8899A6] mt-1">Plan: {claim.plan_id}</p>
                                        )}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </motion.div>
            )}

            {activeTab === "transactions" && (
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6"
                >
                    <div className="flex items-center justify-between mb-6">
                        <h3 className="font-bold text-white text-lg">All Transactions</h3>
                        <div className="flex gap-2">
                            {[7, 30, 90].map((days) => (
                                <button
                                    key={days}
                                    onClick={() => setTimeRange(days)}
                                    className={`px-3 py-1 rounded text-xs font-medium ${
                                        timeRange === days
                                            ? "bg-[#33C5E0] text-white"
                                            : "bg-[#161E22] text-[#8899A6] hover:text-white"
                                    }`}
                                >
                                    {days}d
                                </button>
                            ))}
                        </div>
                    </div>
                    <div className="overflow-x-auto">
                        <table className="w-full text-sm">
                            <thead>
                                <tr className="border-b border-[#161E22]">
                                    <th className="text-left py-3 px-4 text-[#8899A6] font-medium">Type</th>
                                    <th className="text-left py-3 px-4 text-[#8899A6] font-medium">Amount</th>
                                    <th className="text-left py-3 px-4 text-[#8899A6] font-medium">Balance After</th>
                                    <th className="text-left py-3 px-4 text-[#8899A6] font-medium">Date</th>
                                </tr>
                            </thead>
                            <tbody>
                                {recent_transactions.map((tx) => (
                                    <tr key={tx.id} className="border-b border-[#161E22]">
                                        <td className="py-3 px-4">
                                            <span className={`px-2 py-1 rounded text-xs font-medium uppercase ${
                                                tx.transaction_type === "contribution" || tx.transaction_type === "yield"
                                                    ? "bg-[#48BB78]/20 text-[#48BB78]"
                                                    : "bg-[#F56565]/20 text-[#F56565]"
                                            }`}>
                                                {tx.transaction_type}
                                            </span>
                                        </td>
                                        <td className={`py-3 px-4 font-bold ${
                                            tx.transaction_type === "contribution" || tx.transaction_type === "yield"
                                                ? "text-[#48BB78]"
                                                : "text-[#F56565]"
                                        }`}>
                                            {tx.transaction_type === "contribution" || tx.transaction_type === "yield" ? "+" : "-"}
                                            {formatCurrency(tx.amount)} {tx.asset_code}
                                        </td>
                                        <td className="py-3 px-4 text-white">{formatCurrency(tx.balance_after)}</td>
                                        <td className="py-3 px-4 text-[#8899A6]">{formatDate(tx.created_at)}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </motion.div>
            )}

            {activeTab === "claims" && (
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6"
                >
                    <h3 className="font-bold text-white text-lg mb-6">All Insurance Claims</h3>
                    <div className="text-center py-12 text-[#8899A6]">
                        <p>Claims management interface coming soon</p>
                        <p className="text-sm mt-2">Total claims: {total_claims_count} | Total amount: {formatCurrency(total_claims_amount)}</p>
                    </div>
                </motion.div>
            )}
        </div>
    );
}
