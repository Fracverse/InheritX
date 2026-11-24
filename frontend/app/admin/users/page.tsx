"use client";
import { useState, useMemo, useEffect } from "react";
import { Download, FileText } from "lucide-react";
import { adminAPI } from "@/app/lib/api/admin";
import { describeAdminError } from "@/lib/adminErrors";
import { AdminErrorBanner } from "@/components/admin/AdminErrorBanner";
import { AdminUser, KYCStatus } from "@/lib/adminTypes";
import { SearchFilterBar } from "@/components/admin/SearchFilterBar";
import { UserManagementTable } from "@/components/admin/UserManagementTable";
import { Pagination } from "@/components/admin/Pagination";
import { exportUsersAsCsv, exportUsersAsPdf } from "@/lib/export";

const PAGE_SIZE = 10;

export default function AdminUsersPage() {
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [reloadKey, setReloadKey] = useState(0);
  const [search, setSearch] = useState("");
  const [kycFilter, setKycFilter] = useState<KYCStatus | "all">("all");
  const [page, setPage] = useState(1);

  useEffect(() => {
    let cancelled = false;
    adminAPI
      .listUsers()
      .then((data) => {
        if (!cancelled) setUsers(data);
      })
      .catch((err) => {
        if (!cancelled) setLoadError(describeAdminError(err, "Loading users"));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [reloadKey]);

  const retryLoad = () => {
    setLoadError(null);
    setLoading(true);
    setReloadKey((k) => k + 1);
  };

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

  /**
   * Sends a moderation request, then applies the server's response to the row.
   * On failure the row is left untouched and the error is shown.
   */
  const runAction = async (
    userId: string,
    verb: string,
    request: () => Promise<Partial<AdminUser>>
  ) => {
    const address = users.find((u) => u.id === userId)?.walletAddress ?? userId;
    const wallet =
      address.length > 12 ? `${address.slice(0, 6)}...${address.slice(-4)}` : address;
    setActionError(null);
    try {
      const fromServer = await request();
      setUsers((prev) =>
        prev.map((u) => (u.id === userId ? { ...u, ...fromServer } : u))
      );
    } catch (err) {
      setActionError(describeAdminError(err, `${verb} ${wallet}`));
    }
  };

  const handleApproveKYC = (userId: string) =>
    runAction(userId, "Approving KYC for", async () => ({
      kycStatus: await adminAPI.approveUserKyc(userId),
    }));

  const handleTerminatePlan = (userId: string) =>
    runAction(userId, "Terminating plans for", () =>
      adminAPI.terminateUserPlans(userId)
    );

  const handleSuspendUser = (userId: string) => {
    const isActive = users.find((u) => u.id === userId)?.status === "active";
    return runAction(userId, isActive ? "Suspending" : "Unsuspending", () =>
      isActive ? adminAPI.suspendUser(userId) : adminAPI.unsuspendUser(userId)
    );
  };

  return (
    <div className="animate-fade-in space-y-6 max-w-5xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-white">Users</h1>
          <p className="text-sm text-slate-500 mt-1">Platform user accounts and KYC oversight. {loading ? "Loading users…" : `Showing ${filtered.length} of ${users.length} users.`}</p>
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

      {loadError && <AdminErrorBanner message={loadError} onRetry={retryLoad} />}
      {actionError && (
        <AdminErrorBanner message={actionError} onDismiss={() => setActionError(null)} />
      )}

      <SearchFilterBar
        onSearchChange={handleSearchChange}
        onFilterChange={handleFilterChange}
      />

      {loading ? (
        <div role="status" className="py-16 text-center text-sm text-slate-500">
          Loading users…
        </div>
      ) : (
        !loadError && (
          <div className="bg-white/[0.02] border border-white/5 rounded-2xl overflow-hidden">
            <UserManagementTable
              users={paginated}
              onApproveKYC={handleApproveKYC}
              onTerminatePlan={handleTerminatePlan}
              onSuspendUser={handleSuspendUser}
            />
          </div>
        )
      )}

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
