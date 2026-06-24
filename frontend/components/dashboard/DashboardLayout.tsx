"use client";

import { useState, useMemo } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useWallet } from "@/hooks/useWallet";
import { IDashboardRoute } from "@/types/navigation";

const INHERITX_ROUTES: IDashboardRoute[] = [
  { name: "Create Plan", href: "/dashboard/create", icon: "✨" },
  { name: "Edit Plan", href: "/dashboard/edit", icon: "✍️" },
  { name: "Claim Plan", href: "/dashboard/claim", icon: "📥" },
  { name: "KYC Verification", href: "/dashboard/kyc", icon: "🛡️", badge: "Required" },
  { name: "Admin Dashboard", href: "/admin", icon: "👑", adminOnly: true },
];

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const { connected, publicKey, balance, assetSymbol, connect, disconnect } = useWallet();
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);

  // Simple privilege escalation flag mock for demonstration (tie into actual auth layers later)
  const isUserAdmin = true; 

  const visibleRoutes = useMemo(() => {
    return INHERITX_ROUTES.filter(route => !route.adminOnly || isUserAdmin);
  }, [isUserAdmin]);

  return (
    <div className="min-h-screen bg-[#0B0F17] text-[#F3F4F6] font-sans flex flex-col antialiased selection:bg-blue-600/30 selection:text-white">
      
      {/* GLOBAL TOP NAV BAR BARRIER */}
      <header className="h-16 w-full border-b border-gray-800/60 bg-[#0F1420]/90 backdrop-blur-md sticky top-0 z-40 flex items-center justify-between px-4 sm:px-6">
        <div className="flex items-center gap-3">
          <button
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="p-2 -ml-2 rounded-lg hover:bg-gray-800/60 transition-colors md:hidden cursor-pointer"
            aria-label="Toggle structural workspace drawer"
          >
            <span className="text-xl">☰</span>
          </button>
          <Link href="/dashboard" className="flex items-center gap-2 font-extrabold text-lg tracking-wider text-white uppercase font-mono">
            <span className="bg-gradient-to-r from-blue-500 to-indigo-600 text-transparent bg-clip-text">Inherit</span>
            <span className="text-gray-400">X</span>
          </Link>
        </div>

        {/* ACCOUNT STATUS METRICS EXPANSION DESK */}
        <div className="flex items-center gap-4">
          {connected && publicKey ? (
            <div className="hidden sm:flex items-center gap-2 bg-[#131B2E] border border-blue-900/40 rounded-xl px-3 h-9 text-xs font-mono">
              <span className="text-gray-400 font-sans font-medium">Balance:</span>
              <span className="text-blue-400 font-bold">{balance || "0.00"} {assetSymbol || "XLM"}</span>
            </div>
          ) : null}

          {connected && publicKey ? (
            <button
              onClick={() => disconnect?.()}
              className="h-9 px-4 rounded-xl border border-gray-800 hover:border-rose-900/50 hover:bg-rose-950/20 text-gray-300 hover:text-rose-400 text-xs font-semibold font-mono transition-all cursor-pointer"
            >
              {publicKey.slice(0, 4)}...{publicKey.slice(-4)}
            </button>
          ) : (
            <button
              onClick={() => connect?.()}
              className="h-9 px-4 rounded-xl bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 text-white text-xs font-bold shadow-md shadow-blue-950/40 transition-all cursor-pointer"
            >
              Connect Wallet
            </button>
          )}
        </div>
      </header>

      {/* RECONCILED VIEWPORT EXPANSION BLOCK */}
      <div className="flex flex-1 relative">
        
        {/* SIDEBAR NAVIGATION BLOCK */}
        <aside
          className={`fixed inset-y-16 left-0 transform ${
            isSidebarOpen ? "translate-x-0" : "-translate-x-full"
          } md:translate-x-0 md:static w-64 bg-[#0F1420]/60 border-r border-gray-800/40 p-4 transition-transform duration-300 ease-in-out z-30 flex flex-col justify-between backdrop-blur-sm shrink-0`}
        >
          <div className="space-y-6">
            <div className="space-y-1">
              <span className="text-[10px] font-bold tracking-widest text-gray-500 uppercase px-3 block">Navigation Tracks</span>
              <nav className="space-y-1 pt-2">
                {visibleRoutes.map((route) => {
                  const isActive = pathname === route.href;
                  return (
                    <Link
                      key={route.name}
                      href={route.href}
                      onClick={() => setIsSidebarOpen(false)}
                      className={`flex items-center justify-between px-3 h-11 rounded-xl text-sm font-medium transition-all group ${
                        isActive
                          ? "bg-gradient-to-r from-blue-600/15 to-indigo-600/5 border border-blue-500/20 text-blue-400 font-bold"
                          : "text-gray-400 hover:bg-gray-800/40 hover:text-gray-200"
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <span className={`text-base transition-transform group-hover:scale-110 ${isActive ? "opacity-100" : "opacity-60 group-hover:opacity-100"}`}>
                          {route.icon}
                        </span>
                        <span>{route.name}</span>
                      </div>
                      {route.badge && (
                        <span className="text-[9px] bg-amber-500/10 text-amber-500 border border-amber-500/20 px-1.5 py-0.5 rounded font-bold uppercase tracking-wider">
                          {route.badge}
                        </span>
                      )}
                    </Link>
                  );
                })}
              </nav>
            </div>
          </div>

          {/* ASYNC BALANCE HUD FOOTER DRAWER FOR MOBILE RENDER PATHS */}
          {connected && (
            <div className="sm:hidden border-t border-gray-800/60 pt-4 mt-auto">
              <div className="bg-[#131B2E]/60 border border-blue-950 rounded-xl p-3 text-center">
                <span className="text-[10px] text-gray-500 font-medium block uppercase tracking-wide">Wallet Balance</span>
                <span className="text-sm font-mono font-bold text-blue-400">{balance || "0.00"} {assetSymbol || "XLM"}</span>
              </div>
            </div>
          )}
        </aside>

        {/* MOBILE OVERLAY BACKGROUND PANEL BLOCKS */}
        {isSidebarOpen && (
          <div
            onClick={() => setIsSidebarOpen(false)}
            className="fixed inset-0 top-16 bg-black/60 backdrop-blur-xs z-20 md:hidden"
            aria-hidden="true"
          />
        )}

        {/* PRIMARY MAIN LAYOUT CANVAS FRAMEWORK */}
        <main className="flex-1 bg-gradient-to-b from-[#0F1420]/20 to-[#0B0F17] p-4 sm:p-6 lg:p-8 overflow-y-auto max-w-[100vw]">
          <div className="max-w-6xl mx-auto space-y-6">
            {children}
          </div>
        </main>

      </div>
    </div>
  );
}