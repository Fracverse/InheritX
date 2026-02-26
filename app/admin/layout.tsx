"use client";

import { Sidebar } from "@/components/admin/Sidebar";
import { Topbar } from "@/components/admin/Topbar";
import { AdminProtected } from "@/components/admin/AdminProtected";
import { usePathname } from "next/navigation";

export default function AdminLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    const pathname = usePathname();
    const isLoginPage = pathname === "/admin/login";

    if (isLoginPage) {
        return <>{children}</>;
    }

    return (
        <AdminProtected>
            <div className="flex min-h-screen bg-[#060B0D] text-white font-sans">
                {/* Sidebar */}
                <Sidebar />

                {/* Main Content */}
                <div className="flex-1 flex flex-col min-h-screen max-w-full">
                    <Topbar />
                    <main className="flex-1 p-6 md:p-10">
                        {children}
                    </main>
                </div>
            </div>
        </AdminProtected>
    );
}
