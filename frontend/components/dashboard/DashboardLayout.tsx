"use client";
import { OwnerSidebarNav } from "./OwnerSidebarNav";
import { DashboardHeader } from "./DashboardHeader";

export function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen bg-[#161E22]">
      <OwnerSidebarNav />
      <div className="flex flex-col flex-1 min-w-0">
        <DashboardHeader />
        <main className="flex-1 overflow-y-auto sm:p-6 p-4">{children}</main>
      </div>
    </div>
  );
}

export function AdminDashboardLayout({ children }: { children: React.ReactNode }) {
  // Dynamically imported to avoid circular deps
  import AdminSidebarNav from "./AdminSidebarNav";
  return (
    <div className="flex min-h-screen bg-[#161E22]">
      <AdminSidebarNav />
      <div className="flex flex-col flex-1 min-w-0">
        <header className="h-14 flex items-center justify-end px-6 border-b border-white/5 bg-[#0d1117] shrink-0">
          <span className="text-xs text-slate-500">InheritX Admin</span>
        </header>
        <main className="flex-1 overflow-y-auto sm:p-6 p-4">{children}</main>
      </div>
    </div>
  );
}
