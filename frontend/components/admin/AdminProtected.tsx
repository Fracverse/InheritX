"use client";

import React, { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAdminAuth } from "@/context/AdminAuthContext";

export function AdminProtected({ children }: { children: React.ReactNode }) {
    const router = useRouter();
    const { isAuthenticated, isLoading } = useAdminAuth();

    useEffect(() => {
        if (!isLoading && !isAuthenticated) {
            router.push("/admin/login");
        }
    }, [isAuthenticated, isLoading, router]);

    if (isLoading) {
        return (
            <div className="min-h-screen bg-[#060B0D] flex items-center justify-center">
                <div className="text-white text-center">
                    <div className="w-12 h-12 border-4 border-[#1A262C] border-t-[#33C5E0] rounded-full animate-spin mx-auto mb-4"></div>
                    <p>Loading...</p>
                </div>
            </div>
        );
    }

    if (!isAuthenticated) {
        return null;
    }

    return <>{children}</>;
}
