"use client";

import { useState, useMemo } from "react";
import Link from "next/link";
import { Search, Landmark, ArrowUpRight, HelpCircle, Download, FileText } from "lucide-react";
import { exportPayoutsAsCsv, exportPayoutsAsPdf } from "@/lib/export";

// Mock payouts matching anchor flow
const mockPayouts = [
  {
    id: "pay_01jm8v45px",
    planId: "plan_04nm8w90pa",
    beneficiary: "GBZV2I...1H4J",
    amount: 30000,
    currency: "NGN",
    anchor: "Stellar NGN Anchor (Link)",
    status: "SETTLED",
    date: "2026-07-11T12:00:00Z",
  },
  {
    id: "pay_02km9v56py",
    planId: "plan_04nm8w90pa",
    beneficiary: "GDYU7H...8G5I",
    amount: 30000,
    currency: "KES",
    anchor: "Stellar KES Anchor (Pesalink)",
    status: "PROCESSING",
    date: "2026-07-15T09:30:00Z",
  },
  {
    id: "pay_03lm1v23pz",
    planId: "plan_03lm1v23pz",
    beneficiary: "GBXP2Z...5N7Q",
    amount: 37500,
    currency: "BRL",
    anchor: "Stellar BRL Anchor (Pix)",
    status: "FAILED",
    date: "2026-07-02T14:00:00Z",
  },
  {
    id: "pay_04jm8v56qx",
    planId: "plan_05nm8w90pb",
    beneficiary: "GCXY1Z...2J5K",
    amount: 25000,
    currency: "ZAR",
    anchor: "Stellar ZAR Anchor",
    status: "SETTLED",
    date: "2026-07-10T08:15:00Z",
  },
  {
    id: "pay_05km9v67ry",
    planId: "plan_06nm8w90pc",
    beneficiary: "GDRS3M...4L6N",
    amount: 45000,
    currency: "GHS",
    anchor: "Stellar GHS Anchor",
    status: "PROCESSING",
    date: "2026-07-14T16:45:00Z",
  },
  {
    id: "pay_06lm1v78sz",
    planId: "plan_07nm8w90pd",
    beneficiary: "GBUK5P...7O9R",
    amount: 15000,
    currency: "USD",
    anchor: "Stellar USD Anchor",
    status: "SETTLED",
    date: "2026-07-09T11:30:00Z",
  },
];

const ITEMS_PER_PAGE = 10;

export default function AdminPayoutsPage() {
  const [payouts, setPayouts] = useState(mockPayouts);
  const [search, setSearch] = useState("");
  const [beneficiaryFilter, setBeneficiaryFilter] = useState("");
  const [payoutTypeFilter, setPayoutTypeFilter] = useState("ALL");
  const [statusFilter, setStatusFilter] = useState("ALL");
  const [currentPage, setCurrentPage] = useState(1);

  const filtered = useMemo(() => {
    return payouts.filter((p) => {
      const matchesSearch =
        p.id.toLowerCase().includes(search.toLowerCase()) ||
        p.planId.toLowerCase().includes(search.toLowerCase());
      const matchesBeneficiary =
        !beneficiaryFilter ||
        p.beneficiary.toLowerCase().includes(beneficiaryFilter.toLowerCase());
      const matchesType =
        payoutTypeFilter === "ALL" ||
        (payoutTypeFilter === "DIRECT" && p.anchor.includes("Direct")) ||
        (payoutTypeFilter === "ANCHOR" && !p.anchor.includes("Direct"));
      const matchesStatus = statusFilter === "ALL" || p.status === statusFilter;
      return matchesSearch && matchesBeneficiary && matchesType && matchesStatus;
    });
  }, [payouts, search, beneficiaryFilter, payoutTypeFilter, statusFilter]);

  const totalPages = Math.ceil(filtered.length / ITEMS_PER_PAGE);
  const paginatedPayouts = filtered.slice(
    (currentPage - 1) * ITEMS_PER_PAGE,
    currentPage * ITEMS_PER_PAGE
  );

  // Reset to page 1 when filters change
  const handleFilterChange = (callback: () => void) => {
    callback();
    setCurrentPage(1);
  };

  const handleRetryPayout = (payoutId: string) => {
    alert(`Retrying settlement for payout transaction: ${payoutId}`);
    setPayouts((prev) => prev.map((p) => (p.id === payoutId ? { ...p, status: "PROCESSING" } : p)));
  };

  return (
    <div className="animate-fade-in space-y-6 max-w-6xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-white">Payout Settlements</h1>
          <p className="text-sm text-slate-500 mt-1">
            Monitor and retry fiat settlements initiated via Stellar Anchors. Showing {filtered.length} of {payouts.length} payouts.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => exportPayoutsAsCsv(filtered, `payouts-${new Date().toISOString().split('T')[0]}.csv`)}
            className="px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-slate-300 hover:text-white hover:bg-white/10 transition-colors flex items-center gap-2 text-sm"
            title="Export as CSV"
          >
            <Download size={16} />
            CSV
          </button>
          <button
            onClick={() => exportPayoutsAsPdf(filtered)}
            className="px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-slate-300 hover:text-white hover:bg-white/10 transition-colors flex items-center gap-2 text-sm"
            title="Export as PDF"
          >
            <FileText size={16} />
            PDF
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="space-y-3">
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="relative flex-1 min-w-64">
            <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
            <input
              value={search}
              onChange={(e) => handleFilterChange(() => setSearch(e.target.value))}
              placeholder="Search by payout ID or plan ID…"
              className="w-full pl-9 pr-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-violet-500/50 transition-colors"
            />
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          {/* Beneficiary Filter */}
          <div>
            <label className="text-xs font-semibold text-slate-400 mb-1.5 block">Beneficiary Address</label>
            <input
              type="text"
              value={beneficiaryFilter}
              onChange={(e) => handleFilterChange(() => setBeneficiaryFilter(e.target.value))}
              placeholder="Filter by beneficiary…"
              className="w-full px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-violet-500/50 transition-colors"
            />
          </div>

          {/* Payout Type Filter */}
          <div>
            <label className="text-xs font-semibold text-slate-400 mb-1.5 block">Payout Type</label>
            <select
              value={payoutTypeFilter}
              onChange={(e) => handleFilterChange(() => setPayoutTypeFilter(e.target.value))}
              className="w-full px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-slate-200 focus:outline-none focus:border-violet-500/50 transition-colors cursor-pointer"
            >
              <option value="ALL">All Types</option>
              <option value="ANCHOR">Anchor Fiat</option>
              <option value="DIRECT">Direct Transfer</option>
            </select>
          </div>

          {/* Status Filter */}
          <div>
            <label className="text-xs font-semibold text-slate-400 mb-1.5 block">Status</label>
            <select
              value={statusFilter}
              onChange={(e) => handleFilterChange(() => setStatusFilter(e.target.value))}
              className="w-full px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-slate-200 focus:outline-none focus:border-violet-500/50 transition-colors cursor-pointer"
            >
              <option value="ALL">All Statuses</option>
              <option value="SETTLED">Settled</option>
              <option value="PROCESSING">Processing</option>
              <option value="FAILED">Failed</option>
            </select>
          </div>
        </div>
      </div>

      {/* Table */}
      <div className="bg-white/[0.02] border border-white/5 rounded-2xl overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full border-collapse text-left text-xs">
            <thead>
              <tr className="border-b border-white/5 bg-white/[0.02] text-slate-400 font-medium uppercase tracking-wider">
                <th className="p-4">Payout ID</th>
                <th className="p-4">Plan ID</th>
                <th className="p-4">Beneficiary</th>
                <th className="p-4">Amount</th>
                <th className="p-4">Anchor Partner</th>
                <th className="p-4">Status</th>
                <th className="p-4">Settlement Date</th>
                <th className="p-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              {paginatedPayouts.map((pay) => (
                <tr key={pay.id} className="hover:bg-white/[0.01] transition-colors text-slate-300">
                  <td className="p-4 font-mono font-medium text-violet-400">{pay.id}</td>
                  <td className="p-4 font-mono text-slate-400">
                    <Link href={`/admin/plans/${pay.planId}`} className="hover:underline flex items-center gap-1">
                      {pay.planId}
                      <ArrowUpRight size={10} />
                    </Link>
                  </td>
                  <td className="p-4 font-mono text-slate-400">{pay.beneficiary}</td>
                  <td className="p-4 font-semibold text-white">
                    ${pay.amount.toLocaleString()} <span className="text-[10px] text-slate-500">{pay.currency}</span>
                  </td>
                  <td className="p-4 flex items-center gap-2">
                    <Landmark size={12} className="text-slate-500" />
                    <span>{pay.anchor}</span>
                  </td>
                  <td className="p-4">
                    <span
                      className={`inline-flex items-center px-2 py-0.5 rounded-md text-[10px] font-semibold border ${
                        pay.status === "SETTLED"
                          ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
                          : pay.status === "PROCESSING"
                          ? "bg-yellow-500/10 text-yellow-400 border-yellow-500/20"
                          : "bg-red-500/10 text-red-400 border-red-500/20"
                      }`}
                    >
                      {pay.status}
                    </span>
                  </td>
                  <td className="p-4 text-slate-400">
                    {new Date(pay.date).toLocaleDateString()}
                  </td>
                  <td className="p-4 text-right">
                    {pay.status === "FAILED" && (
                      <button
                        onClick={() => handleRetryPayout(pay.id)}
                        className="px-2.5 py-1 rounded-lg bg-violet-500 text-white text-[11px] font-semibold hover:bg-violet-400 transition-colors"
                      >
                        Retry
                      </button>
                    )}
                  </td>
                </tr>
              ))}
              {paginatedPayouts.length === 0 && (
                <tr>
                  <td colSpan={8} className="p-8 text-center text-slate-500">
                    {filtered.length === 0 ? "No settlements found." : ""}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between">
          <div className="text-sm text-slate-400">
            Page {currentPage} of {totalPages} ({filtered.length} total)
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
              disabled={currentPage === 1}
              className="px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-slate-300 hover:text-white hover:bg-white/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Previous
            </button>
            <div className="flex items-center gap-1">
              {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                let pageNum = i + 1;
                if (totalPages > 5 && currentPage > 3) {
                  pageNum = currentPage - 2 + i;
                }
                if (pageNum > totalPages) return null;
                
                return (
                  <button
                    key={pageNum}
                    onClick={() => setCurrentPage(pageNum)}
                    className={`px-3 py-2 rounded-lg text-sm transition-colors ${
                      currentPage === pageNum
                        ? "bg-violet-500 text-white"
                        : "bg-white/5 border border-white/10 text-slate-300 hover:text-white hover:bg-white/10"
                    }`}
                  >
                    {pageNum}
                  </button>
                );
              })}
            </div>
            <button
              onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
              disabled={currentPage === totalPages}
              className="px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-slate-300 hover:text-white hover:bg-white/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
