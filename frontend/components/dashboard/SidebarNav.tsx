"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Menu, ShieldCheck } from "lucide-react";
import HomeIcon from "@/app/svg/HomeIcon";
import PlansIcon from "@/app/svg/PlansIcon";
import ClaimIcon from "@/app/svg/ClaimIcon";
import SecurityIcon from "@/app/svg/SecurityIcon";

const navItems = [
  {
    label: "Overview",
    href: "/asset-owner",
    icon: <HomeIcon />,
  },
  {
    label: "Create Plan",
    href: "/asset-owner/plans/create",
    icon: <PlansIcon />,
  },
  {
    label: "Edit Plan",
    href: "/asset-owner/plans/edit",
    icon: <PlansIcon />,
  },
  {
    label: "Claim Plan",
    href: "/asset-owner/plans/claim",
    icon: <ClaimIcon />,
  },
  {
    label: "KYC Verification",
    href: "/asset-owner/kyc",
    icon: <SecurityIcon />,
  },
  {
    label: "Admin Dashboard",
    href: "/admin/dashboard",
    icon: <ShieldCheck size={16} />,
  },
];

function NavLinks({ onClose }: { onClose?: () => void }) {
  const pathname = usePathname();
  return (
    <nav className="flex flex-col gap-1 mt-6">
      {navItems.map((item) => {
        const isActive = pathname === item.href;
        return (
          <Link
            key={item.href}
            href={item.href}
            onClick={onClose}
            className={`flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium transition-colors ${
              isActive ? "bg-primary/10 text-primary border border-primary/20" : "text-gray-400 hover:text-foreground hover:bg-white/5"
            }`}
          >
            <span className={isActive ? "text-primary" : "text-gray-500"}>{item.icon}</span>
            {item.label}
          </Link>
        );
      })}
    </nav>
  );
}

export function SidebarNav() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      {/* ── Desktop Sidebar ── */}
      <aside className="hidden md:flex flex-col w-56 min-h-screen bg-[#0d1117] border-r border-white/10 px-3 py-6 shrink-0">
        <div className="px-4 mb-2">
          <span className="text-primary font-semibold tracking-wide text-sm uppercase">InheritX</span>
        </div>
        <NavLinks />
      </aside>

      {/* ── Mobile Top Bar (visible only on mobile, sits at top of page) ── */}
      <div className="md:hidden fixed top-0 left-0 right-0 z-30 flex items-center justify-between px-4 h-14 bg-[#0d1117] border-b border-white/10">
        <button onClick={() => setIsOpen(true)} className="text-gray-400 hover:text-foreground p-1" aria-label="Open menu">
          <Menu size={20} />
        </button>
        <span className="text-primary font-semibold tracking-wide text-sm uppercase">InheritX</span>
      </div>

      {/* ── Spacer so page content clears the fixed mobile top bar ── */}
      <div className="md:hidden h-14 shrink-0" />

      {/* ── Mobile Drawer ── */}
      <AnimatePresence>
        {isOpen && (
          <>
            {/* Backdrop */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => setIsOpen(false)}
              className="fixed inset-0 bg-black/60 z-40 md:hidden"
            />

            {/* Drawer panel */}
            <motion.div
              initial={{ x: "-100%" }}
              animate={{ x: 0 }}
              exit={{ x: "-100%" }}
              transition={{ type: "spring", damping: 25, stiffness: 200 }}
              className="fixed top-0 left-0 h-full w-64 bg-[#0d1117] border-r border-white/10 px-3 py-6 z-50 md:hidden"
            >
              <div className="flex items-center justify-between px-4 mb-2">
                <span className="text-primary font-semibold tracking-wide text-sm uppercase">InheritX</span>
                <button onClick={() => setIsOpen(false)} className="text-gray-400 hover:text-foreground" aria-label="Close menu">
                  <X size={18} />
                </button>
              </div>
              <NavLinks onClose={() => setIsOpen(false)} />
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </>
  );
}
