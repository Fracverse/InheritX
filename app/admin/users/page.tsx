"use client";

import React, { useState } from "react";
import { Search, ChevronDown, Filter, AlertTriangle } from "lucide-react";

// Mock Data
const MOCK_USERS = [
  {
    id: "1",
    name: "Ebube Ebuka Onuora",
    address: "0x8a4f...0fe5",
    fullAddress: "0x8a4fde3a12...0fe5",
    kycStatus: "NO KYC",
    plans: [],
    joined: "Jan 18, 2026, 12:01 PM",
    role: "User",
    avatarColor: "bg-orange-400",
  },
  {
    id: "2",
    name: "",
    address: "0x3d19...f9fa",
    fullAddress: "0x3d19bc2...f9fa",
    kycStatus: "NO KYC",
    plans: [],
    joined: "Jan 17, 2026, 08:30 PM",
    role: "User",
    avatarColor: "bg-blue-400",
  },
  {
    id: "3",
    name: "",
    address: "0x1a75...8c5c",
    fullAddress: "0x1a75de...8c5c",
    kycStatus: "NO KYC",
    plans: [],
    joined: "Jan 17, 2026, 07:53 PM",
    role: "User",
    avatarColor: "bg-green-400",
  },
  {
    id: "4",
    name: "Super Admin",
    address: "0x0000...0001",
    fullAddress: "0x0000...0001",
    kycStatus: "NO KYC",
    plans: [],
    joined: "Jan 17, 2026, 07:41 PM",
    role: "Super Admin",
    avatarColor: "bg-purple-500",
  },
  {
    id: "5",
    name: "Adamu Jethro",
    address: "0x4ff8...b807",
    fullAddress: "0x4ff8...b807",
    kycStatus: "NO KYC",
    plans: [],
    joined: "Jan 17, 2026, 07:36 PM",
    role: "User",
    avatarColor: "bg-pink-400",
  },
];

const TABS = [
  { id: "all", label: "All" },
  { id: "user", label: "USER" },
  { id: "admin", label: "ADMIN" },
  { id: "super_admin", label: "SUPER ADMIN" },
];

export default function UsersPage() {
  const [activeTab, setActiveTab] = useState("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [users, setUsers] = useState(MOCK_USERS);
  const [openDropdownId, setOpenDropdownId] = useState<string | null>(null);

  // Filter Logic
  const filteredUsers = users.filter((user) => {
    // Tab Filter
    const matchesTab =
      activeTab === "all"
        ? true
        : activeTab === "super_admin"
          ? user.role === "Super Admin"
          : user.role.toUpperCase() === activeTab.toUpperCase();

    // Search Filter
    const query = searchQuery.toLowerCase();
    const matchesSearch =
      user.name.toLowerCase().includes(query) ||
      user.address.toLowerCase().includes(query) ||
      user.fullAddress.toLowerCase().includes(query);

    return matchesTab && matchesSearch;
  });

  const toggleDropdown = (userId: string) => {
    if (openDropdownId === userId) {
      setOpenDropdownId(null);
    } else {
      setOpenDropdownId(userId);
    }
  };

  const handleRoleChange = (userId: string, newRole: string) => {
    setUsers(users.map((u) => (u.id === userId ? { ...u, role: newRole } : u)));
    setOpenDropdownId(null);
  };

  const ROLE_OPTIONS = ["User", "Admin", "Super Admin"];

  return (
    <div className="space-y-6 animate-fade-in max-w-[1400px]">
      {/* Wallet Warning Banner */}
      <div className="bg-[#1A160A] border border-[#3D2E08] rounded-xl p-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-[#3D2E08] rounded-full">
            <AlertTriangle className="text-[#F59E0B]" size={20} />
          </div>
          <div>
            <h3 className="text-[#F59E0B] font-medium text-sm">
              Wallet not connected
            </h3>
            <p className="text-[#8899A6] text-xs">
              Connect your wallet to perform blockchain operations like KYC
              approval.
            </p>
          </div>
        </div>
        <button className="bg-[#33C5E0] hover:bg-[#2baa c2] text-black font-semibold px-4 py-2 rounded-lg text-sm transition-colors">
          Connect Wallet
        </button>
      </div>

      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white mb-1">User Management</h1>
        <p className="text-[#8899A6] text-sm">
          View and manage platform users.
        </p>
      </div>

      {/* Controls */}
      <div className="flex flex-col md:flex-row gap-4 items-start md:items-center justify-between">
        {/* Search */}
        <div className="relative w-full md:w-96">
          <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <Search className="text-[#4A5568]" size={18} />
          </div>
          <input
            type="text"
            placeholder="Search by address..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="bg-[#0A0F11] border border-[#161E22] text-white text-sm rounded-lg focus:ring-[#33C5E0] focus:border-[#33C5E0] block w-full pl-10 p-2.5 placeholder-[#4A5568] transition-all duration-200"
          />
        </div>

        {/* Filter Tabs */}
        <div className="flex items-center gap-2 overflow-x-auto max-w-full pb-1 md:pb-0">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-2 rounded-lg text-xs font-bold uppercase tracking-wider transition-all duration-200 whitespace-nowrap border ${
                activeTab === tab.id
                  ? "bg-[#33C5E0] text-black border-[#33C5E0]"
                  : "bg-[#0A0F11] text-[#8899A6] border-[#161E22] hover:text-white hover:border-[#4A5568]"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Users Table */}
      <div className="bg-[#0A0F11] border border-[#161E22] rounded-xl overflow-hidden min-h-[400px]">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="bg-[#111719] border-b border-[#161E22]">
                <th className="text-left py-4 px-6 text-[11px] font-bold text-[#4A5568] uppercase tracking-wider">
                  User
                </th>
                <th className="text-left py-4 px-6 text-[11px] font-bold text-[#4A5568] uppercase tracking-wider">
                  KYC Status
                </th>
                <th className="text-left py-4 px-6 text-[11px] font-bold text-[#4A5568] uppercase tracking-wider">
                  Plans
                </th>
                <th className="text-left py-4 px-6 text-[11px] font-bold text-[#4A5568] uppercase tracking-wider">
                  Joined
                </th>
                <th className="text-left py-4 px-6 text-[11px] font-bold text-[#4A5568] uppercase tracking-wider w-48">
                  Role
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#161E22]">
              {filteredUsers.length > 0 ? (
                filteredUsers.map((user) => (
                  <tr
                    key={user.id}
                    className="group hover:bg-[#161E22]/30 transition-colors"
                  >
                    <td className="py-4 px-6">
                      <div className="flex flex-col gap-0.5">
                        <span className="font-mono text-sm font-medium text-gray-200 group-hover:text-white transition-colors">
                          {user.address}
                        </span>
                        {user.name && (
                          <span className="text-xs text-[#8899A6]">
                            {user.name}
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      <span className="inline-flex items-center px-2.5 py-1 rounded border border-[#392E1E] bg-[#271E10] text-[#F59E0B] text-[10px] font-bold uppercase tracking-wide">
                        {user.kycStatus}
                      </span>
                    </td>
                    <td className="py-4 px-6">
                      {/* Empty for mock data as per design */}
                    </td>
                    <td className="py-4 px-6">
                      <span className="text-[#8899A6] text-sm">
                        {user.joined}
                      </span>
                    </td>
                    <td className="py-4 px-6">
                      {user.role === "Super Admin" ? (
                        <span className="inline-flex items-center px-2.5 py-1 rounded bg-[#0F2922] border border-[#0F2922] text-[#10B981] text-[10px] font-bold uppercase tracking-wide">
                          Super Admin
                        </span>
                      ) : (
                        <div className="relative">
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              toggleDropdown(user.id);
                            }}
                            className={`w-full flex items-center justify-between text-left text-sm bg-[#060B0D] border rounded px-3 py-2 transition-all ${
                              openDropdownId === user.id
                                ? "border-[#33C5E0] text-white ring-1 ring-[#33C5E0]"
                                : "border-[#161E22] text-gray-200 hover:border-[#4A5568]"
                            }`}
                          >
                            <span>{user.role}</span>
                            <ChevronDown
                              size={14}
                              className={
                                openDropdownId === user.id
                                  ? "text-[#33C5E0]"
                                  : "text-[#8899A6]"
                              }
                            />
                          </button>

                          {openDropdownId === user.id && (
                            <div className="absolute z-50 mt-1 w-full bg-[#060B0D] border border-[#161E22] rounded-lg shadow-xl overflow-hidden animate-fade-in">
                              {ROLE_OPTIONS.map((role) => (
                                <button
                                  key={role}
                                  onClick={() =>
                                    handleRoleChange(user.id, role)
                                  }
                                  className={`w-full text-left px-3 py-2 text-sm hover:bg-[#161E22] transition-colors ${
                                    user.role === role
                                      ? "text-[#33C5E0] font-medium"
                                      : "text-gray-300"
                                  }`}
                                >
                                  {role}
                                </button>
                              ))}
                            </div>
                          )}
                        </div>
                      )}
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td
                    colSpan={5}
                    className="py-12 text-center text-[#8899A6] text-sm"
                  >
                    No users found matching your filters.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Pagination / Count - Optional but good for UX */}
      <div className="text-right text-xs text-[#4A5568]">
        Showing {filteredUsers.length} users
      </div>
    </div>
  );
}
