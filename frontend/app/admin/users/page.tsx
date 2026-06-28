"use client";
import { useState, useMemo } from "react";
import { mockUsers } from "@/lib/mockAdminUsers";
import { AdminUser, KYCStatus } from "@/lib/adminTypes";
import { SearchFilterBar } from "@/components/admin/SearchFilterBar";
import { UserManagementTable } from "@/components/admin/UserManagementTable";
import { Pagination } from "@/components/admin/Pagination";

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
    // TODO: await api.approveKYC(userId);
  };

  const handleTerminatePlan = async (userId: string) => {
    setUsers((prev) =>
      prev.map((u) =>
        u.id === userId ? { ...u, activePlansCount: 0 } : u
      )
    );
    // TODO: await api.terminatePlan(userId);
  };

  const handleSuspendUser = async (userId: string) => {
    setUsers((prev) =>
      prev.map((u) =>
        u.id === userId
          ? { ...u, status: u.status === "active" ? "suspended" : "active" }
          : u
      )
    );
    // TODO: await api.suspendUser(userId);
  };

  return (
    <div className="min-h-screen bg-background p-6 md:p-10">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-semibold text-foreground">
          User Management
        </h1>
        <p className="text-sm text-gray-500 mt-1">
          Manage users, KYC approvals, and active plans.
        </p>
      </div>

      {/* Stats row */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-8">
        {[
          { label: "Total Users", value: users.length },
          {
            label: "KYC Pending",
            value: users.filter((u) => u.kycStatus === "pending").length,
          },
          {
            label: "Active Users",
            value: users.filter((u) => u.status === "active").length,
          },
          {
            label: "Suspended",
            value: users.filter((u) => u.status === "suspended").length,
          },
        ].map((stat) => (
          <div
            key={stat.label}
            className="bg-white/3 border border-white/10 rounded-xl px-4 py-4"
          >
            <p className="text-xs text-gray-500 mb-1">{stat.label}</p>
            <p className="text-2xl font-semibold text-foreground">
              {stat.value}
            </p>
          </div>
        ))}
      </div>

      {/* Search & Filter */}
      <SearchFilterBar
        onSearchChange={handleSearchChange}
        onFilterChange={handleFilterChange}
      />

      {/* Results count */}
      <p className="text-xs text-gray-500 mb-3">
        Showing {paginated.length} of {filtered.length} users
      </p>

      {/* Table */}
      <UserManagementTable
        users={paginated}
        onApproveKYC={handleApproveKYC}
        onTerminatePlan={handleTerminatePlan}
        onSuspendUser={handleSuspendUser}
      />

      {/* Pagination */}
      <Pagination
        currentPage={page}
        totalPages={totalPages}
        onPageChange={setPage}
      />
    </div>
  );
}
