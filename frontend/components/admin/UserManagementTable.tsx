"use client";
import { UserManagementTableProps } from "@/lib/adminTypes";
import { KYCBadge } from "./KYCBadge";
import { QuickActions } from "./QuickActions";

function truncateWallet(address: string) {
  if (address.length <= 12) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

function formatTVL(value: number) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 0,
  }).format(value);
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function UserManagementTable({
  users,
  onApproveKYC,
  onTerminatePlan,
  onSuspendUser,
}: UserManagementTableProps) {
  if (users.length === 0) {
    return (
      <div className="text-center py-16 text-gray-500 border border-white/10 rounded-xl">
        No users found matching your search.
      </div>
    );
  }

  return (
    <div className="w-full overflow-x-auto rounded-xl border border-white/10">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-white/10 bg-white/3">
            <th className="text-left px-4 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
              Wallet Address
            </th>
            <th className="text-left px-4 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
              Registered
            </th>
            <th className="text-left px-4 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
              KYC Status
            </th>
            <th className="text-left px-4 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
              Active Plans
            </th>
            <th className="text-left px-4 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
              TVL
            </th>
            <th className="text-left px-4 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th className="text-left px-4 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-white/5">
          {users.map((user) => (
            <tr
              key={user.id}
              className="hover:bg-white/3 transition-colors"
            >
              {/* Wallet Address */}
              <td className="px-4 py-3.5 font-mono text-xs text-primary">
                {truncateWallet(user.walletAddress)}
              </td>

              {/* Registration Date */}
              <td className="px-4 py-3.5 text-gray-400">
                {formatDate(user.registrationDate)}
              </td>

              {/* KYC Status */}
              <td className="px-4 py-3.5">
                <KYCBadge status={user.kycStatus} />
              </td>

              {/* Active Plans */}
              <td className="px-4 py-3.5 text-foreground font-medium">
                {user.activePlansCount}
              </td>

              {/* TVL */}
              <td className="px-4 py-3.5 text-foreground font-medium">
                {formatTVL(user.tvl)}
              </td>

              {/* User Status */}
              <td className="px-4 py-3.5">
                <span
                  className={`inline-flex items-center gap-1.5 text-xs font-medium ${
                    user.status === "active" ? "text-emerald-400" : "text-gray-500"
                  }`}
                >
                  <span
                    className={`w-1.5 h-1.5 rounded-full ${
                      user.status === "active" ? "bg-emerald-400" : "bg-gray-500"
                    }`}
                  />
                  {user.status === "active" ? "Active" : "Suspended"}
                </span>
              </td>

              {/* Quick Actions */}
              <td className="px-4 py-3.5">
                <QuickActions
                  user={user}
                  onApproveKYC={onApproveKYC}
                  onTerminatePlan={onTerminatePlan}
                  onSuspendUser={onSuspendUser}
                />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
