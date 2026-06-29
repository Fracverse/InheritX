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
    href: "/admin/users",
    icon: <ShieldCheck size={16} />,
  },
];


function NavLinks({
  pathname,
  closeMenu,
}: {
  pathname: string;
  closeMenu: () => void;
}) {
  return (
    <nav className="mt-6 flex flex-col gap-1">
      {navItems.map((item) => {
        const isActive = pathname === item.href;

        return (
          <Link
            key={item.href}
            href={item.href}
            onClick={closeMenu}
            className={`flex items-center gap-3 rounded-lg px-4 py-2.5 text-sm font-medium transition-colors ${
              isActive
                ? "border border-primary/20 bg-primary/10 text-primary"
                : "text-gray-400 hover:bg-white/5 hover:text-foreground"
            }`}
          >
            <span className={isActive ? "text-primary" : "text-gray-500"}>
              {item.icon}
            </span>

            {item.label}
          </Link>
        );
      })}
    </nav>
  );
}


export function SidebarNav() {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      {/* Desktop Sidebar */}
      <aside className="hidden min-h-screen w-56 shrink-0 flex-col border-r border-white/10 bg-[#0d1117] px-3 py-6 md:flex">
        <div className="mb-2 px-4">
          <span className="text-sm font-semibold uppercase tracking-wide text-primary">
            InheritX
          </span>
        </div>

        <NavLinks
          pathname={pathname}
          closeMenu={() => setIsOpen(false)}
        />
      </aside>


      {/* Mobile Top Bar */}
      <div className="flex items-center justify-between border-b border-white/10 bg-[#0d1117] px-4 py-3 md:hidden">
        <span className="text-sm font-semibold uppercase tracking-wide text-primary">
          InheritX
        </span>

        <button
          onClick={() => setIsOpen(true)}
          className="p-1 text-gray-400 hover:text-foreground"
          aria-label="Open menu"
        >
          <Menu size={20} />
        </button>
      </div>


      {/* Mobile Drawer */}
      <AnimatePresence>
        {isOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => setIsOpen(false)}
              className="fixed inset-0 z-40 bg-black/60 md:hidden"
            />

            <motion.div
              initial={{ x: "-100%" }}
              animate={{ x: 0 }}
              exit={{ x: "-100%" }}
              transition={{
                type: "spring",
                damping: 25,
                stiffness: 200,
              }}
              className="fixed left-0 top-0 z-50 h-full w-64 border-r border-white/10 bg-[#0d1117] px-3 py-6 md:hidden"
            >
              <div className="mb-2 flex items-center justify-between px-4">
                <span className="text-sm font-semibold uppercase tracking-wide text-primary">
                  InheritX
                </span>

                <button
                  onClick={() => setIsOpen(false)}
                  className="text-gray-400 hover:text-foreground"
                  aria-label="Close menu"
                >
                  <X size={18}/>
                </button>
              </div>

              <NavLinks
                pathname={pathname}
                closeMenu={() => setIsOpen(false)}
              />
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </>
  );
}