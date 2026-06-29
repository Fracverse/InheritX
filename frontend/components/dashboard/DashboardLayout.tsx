"use client";
import { SidebarNav } from "./SidebarNav";
import { DashboardHeader } from "./DashboardHeader";

export function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen bg-background">
      {/* Sidebar — desktop only, renders inline */}
      <SidebarNav />

      {/* Main area */}
      <div className="flex flex-col flex-1 min-w-0">
        {/* Mobile top bar is inside SidebarNav, header is separate */}
        <DashboardHeader />

        {/* Page content */}
        <main className="flex-1 overflow-y-auto sm:p-6 p-4">{children}</main>
      </div>
    </div>
  );
}
