"use client";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Image from "next/image";
import {
  X,
  Menu,
  LayoutDashboard,
  FileText,
  PlusCircle,
  HandCoins,
  ShieldCheck,
  Zap,
  LogOut,
  ChevronRight,
} from "lucide-react";
import { useWallet } from "@/context/WalletContext";
import { formatAddress } from "@/util/address";

const ownerNavItems = [
  { label: "Overview", href: "/asset-owner", icon: LayoutDashboard },
  { label: "My Plans", href: "/asset-owner/plans", icon: FileText },
  { label: "Create Plan", href: "/asset-owner/plans/create", icon: PlusCircle },
  { label: "Claim Plan", href: "/asset-owner/plans/claim", icon: HandCoins },
  { label: "Proof of Life", href: "/asset-owner/ping", icon: Zap },
  { label: "KYC Verification", href: "/asset-owner/kyc", icon: ShieldCheck },
];

function OwnerNavLinks({ onClose }: { onClose?: () => void }) {
  const pathname = usePathname();
  return (
    <nav className="flex flex-col gap-0.5 mt-2" aria-label="Asset owner navigation">
      {ownerNavItems.map((item) => {
        const isActive =
          item.href === "/asset-owner"
            ? pathname === "/asset-owner"
            : pathname.startsWith(item.href);
        const Icon = item.icon;
        return (
          <Link
            key={item.href}
            href={item.href}
            onClick={onClose}
            className={`flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 group ${
              isActive
                ? "bg-[#33C5E0]/10 text-[#33C5E0] border border-[#33C5E0]/20"
                : "text-slate-400 hover:text-slate-100 hover:bg-white/5"
            }`}
            aria-current={isActive ? "page" : undefined}
          >
            <Icon
              size={16}
              className={isActive ? "text-[#33C5E0]" : "text-slate-500 group-hover:text-slate-300"}
            />
            <span>{item.label}</span>
            {isActive && <ChevronRight size={12} className="ml-auto text-[#33C5E0]/60" />}
          </Link>
        );
      })}
    </nav>
  );
}

export function OwnerSidebarNav() {
  const [isOpen, setIsOpen] = useState(false);
  const { isConnected, address, openModal } = useWallet();

  const SidebarContent = ({ onClose }: { onClose?: () => void }) => (
    <div className="flex flex-col h-full">
      {/* Logo */}
      <div className="flex items-center gap-2.5 px-3 py-5 border-b border-white/5">
        <div className="w-7 h-7 rounded-lg bg-[#33C5E0]/20 border border-[#33C5E0]/30 flex items-center justify-center">
          <span className="text-[#33C5E0] text-xs font-bold">IX</span>
        </div>
        <span className="text-white font-semibold text-sm tracking-wide">InheritX</span>
        {onClose && (
          <button onClick={onClose} className="ml-auto text-slate-400 hover:text-white p-0.5" aria-label="Close menu">
            <X size={16} />
          </button>
        )}
      </div>

      {/* Nav */}
      <div className="flex-1 px-2 py-4 overflow-y-auto">
        <p className="text-[10px] text-slate-600 uppercase tracking-widest px-3 mb-2 font-medium">Asset Owner</p>
        <OwnerNavLinks onClose={onClose} />
      </div>

      {/* Wallet / Footer */}
      <div className="px-2 pb-4 border-t border-white/5 pt-3">
        {isConnected && address ? (
          <div className="px-3 py-2.5 rounded-xl bg-emerald-500/5 border border-emerald-500/15">
            <div className="flex items-center gap-2 mb-1">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_4px_#34d399]" />
              <span className="text-[10px] text-emerald-400 font-medium uppercase tracking-wide">Connected</span>
            </div>
            <p className="text-xs font-mono text-[#33C5E0]">{formatAddress(address)}</p>
          </div>
        ) : (
          <button
            onClick={openModal}
            className="w-full px-3 py-2.5 rounded-xl bg-[#33C5E0]/10 border border-[#33C5E0]/20 text-[#33C5E0] text-xs font-medium hover:bg-[#33C5E0]/20 transition-colors"
          >
            Connect Wallet
          </button>
        )}
      </div>
    </div>
  );

  return (
    <>
      {/* Desktop Sidebar */}
      <aside className="hidden md:flex flex-col w-56 min-h-screen bg-[#0d1117] border-r border-white/5 shrink-0">
        <SidebarContent />
      </aside>

      {/* Mobile Top Bar */}
      <div className="md:hidden fixed top-0 left-0 right-0 z-30 flex items-center justify-between px-4 h-14 bg-[#0d1117] border-b border-white/5">
        <button onClick={() => setIsOpen(true)} className="text-slate-400 hover:text-white p-1" aria-label="Open menu">
          <Menu size={20} />
        </button>
        <span className="text-[#33C5E0] font-semibold text-sm tracking-wide">InheritX</span>
        <div className="w-8" />
      </div>
      <div className="md:hidden h-14 shrink-0" />

      {/* Mobile Drawer */}
      <AnimatePresence>
        {isOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => setIsOpen(false)}
              className="fixed inset-0 bg-black/70 z-40 md:hidden"
            />
            <motion.div
              initial={{ x: "-100%" }}
              animate={{ x: 0 }}
              exit={{ x: "-100%" }}
              transition={{ type: "spring", damping: 28, stiffness: 220 }}
              className="fixed top-0 left-0 h-full w-60 bg-[#0d1117] border-r border-white/5 z-50 md:hidden"
            >
              <SidebarContent onClose={() => setIsOpen(false)} />
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </>
  );
}
