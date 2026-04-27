"use client";

import { useState } from "react";
import Image from "next/image";
import { motion } from "framer-motion";
import { useAdminAuth } from "@/context/AdminAuthContext";
import { Mail, Lock } from "lucide-react";

export default function AdminLoginPage() {
    const { login, isLoading } = useAdminAuth();
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");

    const handleLogin = async (e: React.FormEvent) => {
        e.preventDefault();
        await login(email, password);
    };

    return (
        <div className="min-h-screen bg-[#060B0D] flex items-center justify-center px-4">
            <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5 }}
                className="w-full max-w-sm bg-[#0B1215] rounded-2xl shadow-[0_20px_60px_rgba(0,0,0,0.6)] p-8"
            >
                <div className="flex justify-center mb-6">
                    <Image
                        src="/logo.svg"
                        alt="Logo"
                        width={36}
                        height={36}
                    />
                </div>

                <h1 className="text-center text-xl font-semibold text-white">
                    Admin Login
                </h1>
                <p className="text-center text-sm text-[#8899A6] mt-1">
                    Sign in to access the admin dashboard
                </p>

                {/* Form */}
                <form onSubmit={handleLogin} className="mt-8 space-y-5">
                    <div>
                        <p className="text-[#8899A6] text-sm mb-1">Email Address</p>
                        <div className="relative">
                            <Mail
                                className="absolute left-4 top-3.5 text-[#4A5568]"
                                size={18}
                            />
                            <input
                                type="email"
                                placeholder="admin@inheritx.com"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                disabled={isLoading}
                                required
                                className="w-full h-12 pl-11 pr-4 rounded-xl bg-[#0E161A] border border-[#1A262C] text-white placeholder-[#4A5568]"
                            />
                        </div>
                    </div>

                    <div>
                        <p className="text-[#8899A6] text-sm mb-1">Password</p>
                        <div className="relative">
                            <Lock
                                className="absolute left-4 top-3.5 text-[#4A5568]"
                                size={18}
                            />
                            <input
                                type="password"
                                placeholder="Enter your password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                disabled={isLoading}
                                required
                                className="w-full h-12 pl-11 pr-4 rounded-xl bg-[#0E161A] border border-[#1A262C] text-white placeholder-[#4A5568]"
                            />
                        </div>
                    </div>

                    <button
                        type="submit"
                        disabled={isLoading}
                        className="w-full h-12 rounded-xl bg-[#33C5E0] text-[#041014] font-semibold hover:opacity-90 transition disabled:opacity-60"
                    >
                        {isLoading ? "Signing in..." : "Sign In"}
                    </button>
                </form>

                <div className="mt-6 text-center">
                    <button
                        type="button"
                        className="text-sm text-[#8899A6] hover:text-[#33C5E0] transition"
                    >
                        ← Back to Home
                    </button>
                </div>
            </motion.div>
        </div>
    );
}
