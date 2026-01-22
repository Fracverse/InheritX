"use client";

import React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import clsx from "clsx";

import ClaimIcon from "@/app/svg/ClaimIcon";
import HomeIcon from "@/app/svg/HomeIcon";
import InactivityIcon from "@/app/svg/InactivityIcon";
import PlansIcon from "@/app/svg/PlansIcon";
import PortfolioIcon from "@/app/svg/PortfolioIcon";
import SecurityIcon from "@/app/svg/SecurityIcon";
import SwapIcon from "@/app/svg/SwapIcon";

const normalizePath = (path: string) => {
  if (path !== "/" && path.endsWith("/")) {
    return path.slice(0, -1);
  }
  return path;
};

const SIDEBAR_ITEMS = [
  { label: "Home", href: "/asset-owner/", icon: HomeIcon, exact: true },
  { label: "Plans", href: "/asset-owner/plans", icon: PlansIcon },
  { label: "Claim", href: "/asset-owner/claim", icon: ClaimIcon },
  { label: "Swap", href: "/asset-owner/swap", icon: SwapIcon },
  { label: "Portfolio", href: "/asset-owner/portfolio", icon: PortfolioIcon },
  {
    label: "Inactivity",
    href: "/asset-owner/inactivity",
    icon: InactivityIcon,
  },
  { label: "Security", href: "/asset-owner/security", icon: SecurityIcon },
];

export default function Sidebar() {
  const pathname = normalizePath(usePathname());

  return (
    <div className="pl-12">
      <nav
        className="py-10 px-5 w-[250px] flex flex-col gap-y-4 rounded-t-[8px] rounded-b-[48px]
        shadow-[inset_4px_4px_10px_0px_#11171AE5,inset_-4px_-4px_8px_0px_#1B252AE5,inset_4px_-4px_8px_0px_#11171A33,inset_-4px_4px_8px_0px_#11171A33]"
      >
        {SIDEBAR_ITEMS.map(({ label, href, icon: Icon, exact }) => {
          const normalizedHref = normalizePath(href);

          const isActive = exact
            ? pathname === normalizedHref
            : pathname.startsWith(normalizedHref);

          return (
            <Link
              key={label}
              href={href}
              className={clsx(
                "group flex items-center gap-x-1 font-semibold transition-all duration-200",
                isActive
                  ? "text-[#33C5E0]"
                  : "text-[#92A5A8] hover:text-[#CDEFF5]",
              )}
            >
              <div
                className={clsx(
                  "h-8 w-[6px] rounded-full transition-all duration-200",
                  isActive
                    ? "bg-[#1C252A]"
                    : "bg-transparent group-hover:bg-[#1C252A]/60",
                )}
              />

              <div
                className={clsx(
                  "flex-1 py-5 pl-[50px] rounded-l-[4px] rounded-r-[16px] flex items-center gap-x-3 transition-all duration-200",
                  isActive ? "bg-[#1C252A]" : "group-hover:bg-[#1C252A]/60",
                )}
              >
                <Icon />
                {label}
              </div>
            </Link>
          );
        })}
      </nav>
    </div>
  );
}
