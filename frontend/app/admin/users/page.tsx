"use client";
import { useState, useMemo } from "react";
import { Download, FileText } from "lucide-react";
import { mockUsers } from "@/lib/mockAdminUsers";
import { AdminUser, KYCStatus, UserStatus } from "@/lib/adminTypes";
import { SearchFilterBar } from "@/components/admin/SearchFilterBar";
import { UserManagementTable } from "@/components/admin/UserManagementTable";
import { Pagination } from "@/components/admin/Pagination";
import { exportUsersAsCsv, exportUsersAsPdf } from "@/lib/export";

const PAGE_SIZE = 10;

export default function AdminUsersPage() {
  const [users, setUsers] = useState<AdminUser[]>(mockUsers);
  const [search, setSearch] = useState("");
  const [kycFilter, setKycFilter] = useState<KYCStatus | "all">("all");
  const [page, setPage] = useState(1);

  const filtered = useMemo(() => {
    return users.filter((u) => {
      const matchesSearch = u.walletAddress
        .toLowerCase()
        .includes(search.toLowerCase());
      const matchesKYC = kycFilter === "all" || u.kycStatus === kycFilter;
      return matchesSearch && matchesKYC;
    });
  }, [users, search, kycFilter]);

  const totalPages = Math.ceil(filtered.length / PAGE_SIZE);
  const paginated = filtered.slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE);

  const handleSearchChange = (value: string) => {
    setSearch(value);
    setPage(1);
  };

  const handleFilterChange = (value: KYCStatus | "all") => {
    setKycFilter(value);
    setPage(1);
  };

  const handleApproveKYC = async (userId: string) => {
    setUsers((prev) =>
      prev.map((u) =>
        u.id === userId ? { ...u, kycStatus: "approved" as KYCStatus } : u
      )
    );
  };

  const handleTerminatePlan = async (userId: string) => {
    setUsers((prev) =>
      prev.map((u) =>
        u.id === userId ? { ...u, activePlansCount: 0 } : u
      )
    );
  };

  const handleSuspendUser = async (userId: string) => {
    setUsers((prev) =>
      prev.map((u) =>
        u.id === userId
          ? { ...u, status: u.status === "active" ? ("suspended" as UserStatus) : ("active" as UserStatus) }
          : u
      )
    );
  };

  return (
    <div className="animate-fade-in space-y-6 max-w-5xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-white">Users</h1>
          <p className="text-sm text-slate-500 mt-1">Platform user accounts and KYC oversight. Showing {filtered.length} of {users.length} users.</p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => exportUsersAsCsv(filtered, `users-${new Date().toISOString().split('T')[0]}.csv`)}
            className="px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-slate-300 hover:text-white hover:bg-white/10 transition-colors flex items-center gap-2 text-sm"
            title="Export as CSV"
          >
            <Download size={16} />
            CSV
          </button>
          <button
            onClick={() => exportUsersAsPdf(filtered)}
            className="px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-slate-300 hover:text-white hover:bg-white/10 transition-colors flex items-center gap-2 text-sm"
            title="Export as PDF"
          >
            <FileText size={16} />
            PDF
          </button>
        </div>
      </div>

      <SearchFilterBar
        onSearchChange={handleSearchChange}
        onFilterChange={handleFilterChange}
      />

      <div className="bg-white/[0.02] border border-white/5 rounded-2xl overflow-hidden">
        <UserManagementTable
          users={paginated}
          onApproveKYC={handleApproveKYC}
          onTerminatePlan={handleTerminatePlan}
          onSuspendUser={handleSuspendUser}
        />
      </div>

      {totalPages > 1 && (
        <Pagination
          currentPage={page}
          totalPages={totalPages}
          onPageChange={setPage}
        />
      )}
    </div>
  );
}
