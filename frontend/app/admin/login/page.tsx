"use client";
import { useState } from "react";
import { useAdminAuth } from "@/context/AdminAuthContext";

export default function AdminLoginPage() {
  const { login } = useAdminAuth();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleLogin = async () => {
    setError("");
    setIsLoading(true);
    try {
      await login(email, password);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Login failed");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center px-4">
      <div className="w-full max-w-sm">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-semibold text-foreground">InheritX</h1>
          <p className="text-sm text-gray-500 mt-1">Admin Portal</p>
        </div>
        <div className="bg-white/3 border border-white/10 rounded-2xl p-6">
          <h2 className="text-base font-medium text-foreground mb-6">
            Sign in to continue
          </h2>
          <div className="mb-4">
            <label className="block text-xs text-gray-500 mb-1.5">Email address</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="admin@inheritx.com"
              className="w-full px-3 py-2.5 text-sm bg-white/5 border border-white/10 rounded-lg text-foreground placeholder-gray-600 focus:outline-none focus:border-primary/50 transition-colors"
            />
          </div>
          <div className="mb-6">
            <label className="block text-xs text-gray-500 mb-1.5">Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••"
              onKeyDown={(e) => e.key === "Enter" && handleLogin()}
              className="w-full px-3 py-2.5 text-sm bg-white/5 border border-white/10 rounded-lg text-foreground placeholder-gray-600 focus:outline-none focus:border-primary/50 transition-colors"
            />
          </div>
          {error && (
            <p className="text-xs text-red-400 mb-4 bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2">
              {error}
            </p>
          )}
          <button
            onClick={handleLogin}
            disabled={isLoading || !email || !password}
            className="w-full py-2.5 text-sm font-medium rounded-lg bg-primary text-black hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isLoading ? "Signing in..." : "Sign in"}
          </button>
        </div>
      </div>
    </div>
  );
}
