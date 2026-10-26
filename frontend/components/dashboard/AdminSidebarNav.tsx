"use client";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  X,
  Menu,
  LayoutDashboard,
  Users,
  FileStack,
  ArrowLeftRight,
  LogOut,
  ChevronRight,
  ShieldCheck,
} from "lucide-react";
import { useAdminAuth } from "@/context/AdminAuthContext";

const adminNavItems = [
  { label: "Dashboard", href: "/admin/dashboard", icon: LayoutDashboard },
  { label: "Users", href: "/admin/users", icon: Users },
  { label: "Plans", href: "/admin/plans", icon: FileStack },
  { label: "Payouts", href: "/admin/payouts", icon: ArrowLeftRight },
];

function AdminNavLinks({ onClose }: { onClose?: () => void }) {
  const pathname = usePathname();
  return (
    <nav className="flex flex-col gap-0.5 mt-2" aria-label="Admin navigation">
      {adminNavItems.map((item) => {
        const isActive = pathname.startsWith(item.href);
        const Icon = item.icon;
        return (
          <Link
            key={item.href}
            href={item.href}
            onClick={onClose}
            className={`flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 group ${
              isActive
                ? "bg-violet-500/10 text-violet-300 border border-violet-500/20"
                : "text-slate-400 hover:text-slate-100 hover:bg-white/5"
            }`}
            aria-current={isActive ? "page" : undefined}
          >
            <Icon
              size={16}
              className={isActive ? "text-violet-400" : "text-slate-500 group-hover:text-slate-300"}
            />
            <span>{item.label}</span>
            {isActive && <ChevronRight size={12} className="ml-auto text-violet-400/60" />}
          </Link>
        );
      })}
    </nav>
  );
}

export function AdminSidebarNav() {
  const [isOpen, setIsOpen] = useState(false);
  const { logout } = useAdminAuth();
  const router = useRouter();

  const handleLogout = () => {
    logout();
    router.push("/admin/login");
  };

  const SidebarContent = ({ onClose }: { onClose?: () => void }) => (
    <div className="flex flex-col h-full">
      {/* Logo */}
      <div className="flex items-center gap-2.5 px-3 py-5 border-b border-white/5">
        <div className="w-7 h-7 rounded-lg bg-violet-500/20 border border-violet-500/30 flex items-center justify-center">
          <ShieldCheck size={14} className="text-violet-400" />
        </div>
        <div>
          <p className="text-white font-semibold text-sm">InheritX</p>
          <p className="text-[10px] text-violet-400">Admin Portal</p>
        </div>
        {onClose && (
          <button onClick={onClose} className="ml-auto text-slate-400 hover:text-white p-0.5" aria-label="Close menu">
            <X size={16} />
          </button>
        )}
      </div>

      {/* Nav */}
      <div className="flex-1 px-2 py-4 overflow-y-auto">
        <p className="text-[10px] text-slate-600 uppercase tracking-widest px-3 mb-2 font-medium">Administration</p>
        <AdminNavLinks onClose={onClose} />
      </div>

      {/* Footer */}
      <div className="px-2 pb-4 border-t border-white/5 pt-3">
        <button
          onClick={handleLogout}
          className="flex items-center gap-2 w-full px-3 py-2.5 rounded-xl text-sm text-slate-500 hover:text-red-400 hover:bg-red-500/5 transition-colors"
        >
          <LogOut size={14} />
          <span>Sign Out</span>
        </button>
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
        <span className="text-violet-400 font-semibold text-sm">Admin Portal</span>
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
