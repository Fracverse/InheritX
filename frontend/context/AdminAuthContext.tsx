"use client";

import React, { createContext, useContext, useState, useEffect } from "react";
import { useRouter } from "next/navigation";

interface AdminAuthContextType {
    isAuthenticated: boolean;
    admin: { email: string } | null;
    login: (email: string, password: string) => Promise<void>;
    logout: () => void;
    isLoading: boolean;
}

const AdminAuthContext = createContext<AdminAuthContextType | undefined>(undefined);

export function AdminAuthProvider({ children }: { children: React.ReactNode }) {
    const router = useRouter();
    const [isAuthenticated, setIsAuthenticated] = useState(false);
    const [admin, setAdmin] = useState<{ email: string } | null>(null);
    const [isLoading, setIsLoading] = useState(true);

    // Check if admin is already logged in on mount
    useEffect(() => {
        const checkAuth = () => {
            try {
                const storedAdmin = localStorage.getItem("adminAuth");
                if (storedAdmin) {
                    const adminData = JSON.parse(storedAdmin);
                    setAdmin(adminData);
                    setIsAuthenticated(true);
                }
            } catch (error) {
                console.error("Failed to restore auth:", error);
            } finally {
                setIsLoading(false);
            }
        };

        checkAuth();
    }, []);

    const login = async (email: string, password: string) => {
        // In a real app, this would call your backend API
        // For now, we'll do basic validation
        if (!email || !password) {
            throw new Error("Email and password are required");
        }

        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 500));

        // Basic validation (in production, validate against backend)
        const adminData = { email };
        localStorage.setItem("adminAuth", JSON.stringify(adminData));
        setAdmin(adminData);
        setIsAuthenticated(true);

        // Navigate to admin dashboard
        router.push("/admin");
    };

    const logout = () => {
        localStorage.removeItem("adminAuth");
        setAdmin(null);
        setIsAuthenticated(false);
        router.push("/");
    };

    return (
        <AdminAuthContext.Provider value={{ isAuthenticated, admin, login, logout, isLoading }}>
            {children}
        </AdminAuthContext.Provider>
    );
}

export function useAdminAuth() {
    const context = useContext(AdminAuthContext);
    if (context === undefined) {
        throw new Error("useAdminAuth must be used within AdminAuthProvider");
    }
    return context;
}
