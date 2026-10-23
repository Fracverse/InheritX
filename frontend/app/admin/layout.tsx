"use client";
import { useEffect } from "react";
import { useRouter, usePathname } from "next/navigation";
import { useAdminAuth } from "@/context/AdminAuthContext";
import { AdminSidebarNav } from "@/components/dashboard/AdminSidebarNav";
import { ThemeSwitcher } from "@/components/ThemeSwitcher";

export default function AdminLayout({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAdminAuth();
  const router = useRouter();
  const pathname = usePathname();
  const isLoginPage = pathname === "/admin/login";

  useEffect(() => {
    if (!isLoading && !isAuthenticated && !isLoginPage) {
      router.push("/admin/login");
    }
  }, [isAuthenticated, isLoading, isLoginPage, router]);

  if (isLoading) {
    return (
      <div className="min-h-screen bg-[#161E22] flex items-center justify-center">
        <div className="w-6 h-6 rounded-full border-2 border-[#33C5E0] border-t-transparent animate-spin" />
      </div>
    );
  }

  // Login page gets its own full-screen layout
  if (isLoginPage) return <>{children}</>;

  if (!isAuthenticated) return null;

  return (
    <div className="flex min-h-screen bg-[#161E22]">
      <AdminSidebarNav />
      <div className="flex flex-col flex-1 min-w-0">
        {/* Admin Top Bar */}
        <header className="h-14 flex items-center justify-between px-6 border-b border-white/5 bg-[#0d1117] shrink-0">
          <p className="text-xs text-slate-500">Admin Portal</p>
          <div className="flex items-center gap-4">
            <ThemeSwitcher />
            <div className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-violet-400 shadow-[0_0_4px_#a78bfa]" />
              <span className="text-xs text-violet-400 font-medium">Admin</span>
            </div>
          </div>
        </header>
        <main className="flex-1 overflow-y-auto sm:p-6 p-4">{children}</main>
      </div>
    </div>
  );
}
