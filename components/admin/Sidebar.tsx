"use client";

import React from "react";
import Image from "next/image";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
    LayoutDashboard,
    ShieldCheck,
    Users,
    FileText,
    Activity,
    ArrowLeft,
    LogOut
} from "lucide-react";
import { motion } from "framer-motion";

const navItems = [
    { name: "Dashboard", icon: LayoutDashboard, href: "/admin", section: "ADMINISTRATION" },
    { name: "KYC Management", icon: ShieldCheck, href: "/admin/kyc-management", section: "ADMINISTRATION" },
    { name: "Users", icon: Users, href: "/admin/users", section: "ADMINISTRATION" },
    { name: "All Plans", icon: FileText, href: "/admin/all-plans", section: "ADMINISTRATION" },
    { name: "Activity Log", icon: Activity, href: "/admin/activity-log", section: "ADMINISTRATION" },
];

const quickLinks = [
    { name: "Back to Dashboard", icon: ArrowLeft, href: "/dashboard", section: "QUICK LINKS" },
    { name: "Sign Out", icon: LogOut, href: "/logout", section: "QUICK LINKS", danger: true },
];

export function Sidebar() {
    const pathname = usePathname();

    return (
        <aside className="w-64 border-r border-[#161E22] bg-[#060B0D] flex flex-col h-screen sticky top-0 hidden lg:flex">
            <div className="p-6">
                <div className="flex items-center gap-2 mb-10">
                    <div className="relative w-8 h-8">
                        <Image
                            src="/logo.svg"
                            alt="InheritX Logo"
                            fill
                            className="object-contain"
                        />
                    </div>
                    <span className="text-xl font-bold tracking-tight">InheritX</span>
                    <span className="bg-[#161E22] text-[#8899A6] text-[10px] px-2 py-0.5 rounded font-medium uppercase ml-1">
                        Admin
                    </span>
                </div>

                <div className="space-y-8">
                    <div>
                        <h3 className="text-[#4A5568] text-[11px] font-bold uppercase tracking-wider mb-4 px-2">
                            Administration
                        </h3>
                        <nav className="space-y-1">
                            {navItems.map((item) => {
                                const isActive = pathname === item.href;
                                return (
                                    <Link
                                        key={item.href}
                                        href={item.href}
                                        className={`flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-200 group ${isActive
                                                ? "bg-[#161E22] text-[#33C5E0]"
                                                : "text-[#8899A6] hover:bg-[#161E22] hover:text-white"
                                            }`}
                                    >
                                        <item.icon size={18} className={isActive ? "text-[#33C5E0]" : "text-[#8899A6] group-hover:text-white"} />
                                        <span className="text-sm font-medium">{item.name}</span>
                                    </Link>
                                );
                            })}
                        </nav>
                    </div>

                    <div>
                        <h3 className="text-[#4A5568] text-[11px] font-bold uppercase tracking-wider mb-4 px-2">
                            Quick Links
                        </h3>
                        <nav className="space-y-1">
                            {quickLinks.map((item) => (
                                <Link
                                    key={item.href}
                                    href={item.href}
                                    className={`flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-200 group ${item.danger
                                            ? "text-[#FF4D4D] hover:bg-[#FF4D4D]/10"
                                            : "text-[#8899A6] hover:bg-[#161E22] hover:text-white"
                                        }`}
                                >
                                    <item.icon size={18} />
                                    <span className="text-sm font-medium">{item.name}</span>
                                </Link>
                            ))}
                        </nav>
                    </div>
                </div>
            </div>

            <div className="mt-auto p-4 border-t border-[#161E22]">
                <div className="bg-[#0A0F11] rounded-xl p-4 flex flex-col gap-1">
                    <p className="text-xs text-[#4A5568]">Logged in as</p>
                    <p className="text-sm font-semibold text-white">Super Admin</p>
                    <p className="text-[10px] text-[#33C5E0] font-mono">SUPER_ADMIN</p>
                </div>
            </div>
        </aside>
    );
}
