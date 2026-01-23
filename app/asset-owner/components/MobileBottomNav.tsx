"use client";

import React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import clsx from "clsx";

import HomeIcon from "@/app/svg/HomeIcon";
import PlansIcon from "@/app/svg/PlansIcon";
import SecurityIcon from "@/app/svg/SecurityIcon";
import SwapIcon from "@/app/svg/SwapIcon";

const normalizePath = (path: string) => {
  if (path !== "/" && path.endsWith("/")) {
    return path.slice(0, -1);
  }
  return path;
};

const MOBILE_NAV_ITEMS = [
  { label: "HOME", href: "/asset-owner/", icon: HomeIcon, exact: true },
  { label: "PLANS", href: "/asset-owner/plans", icon: PlansIcon },
  { label: "SWAP", href: "/asset-owner/swap", icon: SwapIcon },
  { label: "SECURITY", href: "/asset-owner/security", icon: SecurityIcon },
];

export default function MobileBottomNav() {
  const pathname = normalizePath(usePathname());

  return (
    <div className="fixed bottom-0 left-0 right-0 z-50 border-t border-[#1C252A] px-6 py-4 md:hidden">
      <nav className="flex justify-between items-center max-w-sm mx-auto w-full">
        {MOBILE_NAV_ITEMS.map(({ label, href, icon: Icon, exact }) => {
          const normalizedHref = normalizePath(href);

          const isActive = exact
            ? pathname === normalizedHref
            : pathname.startsWith(normalizedHref);

          return (
            <Link
              key={label}
              href={href}
              className="flex flex-col items-center gap-y-1"
            >
              <div
                className={clsx(
                  "p-2 rounded-xl transition-all duration-200",
                  isActive ? "bg-[#1C252A]" : "bg-transparent"
                )}
              >
                <div
                  className={clsx(
                    "transition-colors duration-200",
                    isActive ? "text-[#33C5E0]" : "text-white"
                  )}
                >
                  <Icon />
                </div>
              </div>
              <span
                className={clsx(
                  "text-[10px] font-bold tracking-wider transition-colors duration-200",
                  isActive ? "text-[#33C5E0]" : "text-white"
                )}
              >
                {label}
              </span>
            </Link>
          );
        })}
      </nav>
    </div>
  );
}
