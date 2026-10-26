"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import Link from "next/link";
import { ArrowUpRight, Menu, X, ArrowLeft } from "lucide-react";
import { ConnectButton } from "@/components/ConnectButton";
import Footer from "../components/Footer";

export default function AboutPage() {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [isScrolled, setIsScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 50);
    };
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <div className="relative min-h-screen bg-[#161E22] text-slate-300 selection:text-black overflow-x-hidden">
      {/* Navigation */}
      <header className={`sticky top-0 z-50 backdrop-blur-xs transition-all duration-300 ${isScrolled ? "bg-[#161E22]/60 shadow-lg" : "bg-[#161E22]/40"}`} role="banner">
        <nav className="flex justify-between items-center px-6 md:px-40 py-6 mx-auto" role="navigation" aria-label="Main navigation">
          <div className="flex items-center gap-12 relative z-10">
            <Link href="/" className="focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm">
              <Image src="/logo.svg" alt="InheritX" width={50} height={50} quality={75} />
            </Link>
            <div className="hidden md:flex gap-8 text-xs font-medium uppercase tracking-widest text-slate-400">
              <Link href="/" className="hover:text-cyan-400 transition-colors px-1">Home</Link>
              <Link href="/about" className="text-cyan-400 transition-colors px-1">About</Link>
              <Link href="/faqs" className="hover:text-cyan-400 transition-colors px-1">FAQs</Link>
              <Link href="/Guidelines" className="hover:text-cyan-400 transition-colors px-1">Guidelines</Link>
            </div>
          </div>

          <button
            className="md:hidden text-slate-300 hover:text-cyan-400 p-2 relative z-10"
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
            aria-label={isMobileMenuOpen ? "Close menu" : "Open menu"}
            aria-expanded={isMobileMenuOpen}
          >
            {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
          </button>

          {isMobileMenuOpen && (
            <div className="absolute top-full left-0 w-full bg-[#161E22] border-b border-[#2A3338] p-4 flex flex-col gap-4 md:hidden shadow-2xl z-10">
              <Link href="/" onClick={() => setIsMobileMenuOpen(false)} className="text-slate-300 hover:text-cyan-400 py-2 uppercase">Home</Link>
              <Link href="/about" onClick={() => setIsMobileMenuOpen(false)} className="text-cyan-400 py-2 uppercase">About</Link>
              <Link href="/faqs" onClick={() => setIsMobileMenuOpen(false)} className="text-slate-300 hover:text-cyan-400 py-2 uppercase">FAQs</Link>
              <Link href="/Guidelines" onClick={() => setIsMobileMenuOpen(false)} className="text-slate-300 hover:text-cyan-400 py-2 uppercase">Guidelines</Link>
              <ConnectButton />
            </div>
          )}
          <div className="md:block hidden">
            <ConnectButton />
          </div>
        </nav>
      </header>

      {/* Main Content */}
      <main className="max-w-4xl mx-auto px-6 py-20 relative z-10">
        <Link href="/" className="inline-flex items-center gap-2 text-xs text-[#33C5E0] hover:underline mb-8">
          <ArrowLeft size={12} /> Back to Home
        </Link>

        <h1 className="text-4xl font-bold text-white mb-6">About InheritX</h1>
        <div className="space-y-6 text-sm leading-relaxed text-slate-400">
          <p>
            InheritX is a programmable, fiat-native digital estate planning and wealth inheritance protocol built on the **Stellar Network** using **Soroban Smart Contracts**. We bridge the gap between secure blockchain asset custody and real-world accessibility.
          </p>
          <p>
            Traditional estate planning is slow, expensive, and complex. By leveraging smart contract execution, InheritX removes intermediaries, allows users to secure yield-generating vaults, and automatically initiates payouts to beneficiaries when liveness verification fails.
          </p>

          <h2 className="text-xl font-semibold text-white mt-10 mb-4">Our Mission</h2>
          <p>
            Our mission is to empower individuals to secure their digital legacies with total autonomy. By incorporating **Stellar Anchor integrations**, we ensure that heirs do not need blockchain literacy or wallets—receiving assets off-ramped into local fiat currencies directly into their local bank accounts or mobile wallets.
          </p>

          <h2 className="text-xl font-semibold text-white mt-10 mb-4">Core Principles</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-6">
            <div className="p-5 rounded-2xl bg-white/[0.02] border border-white/5">
              <h3 className="text-sm font-semibold text-white mb-2">Autonomy</h3>
              <p className="text-xs text-slate-500">Total non-custodial control over your assets and terms until payout parameters are met.</p>
            </div>
            <div className="p-5 rounded-2xl bg-white/[0.02] border border-white/5">
              <h3 className="text-sm font-semibold text-white mb-2">Efficiency</h3>
              <p className="text-xs text-slate-500">Locked assets continuously generate yield in secure vaults for beneficiaries.</p>
            </div>
            <div className="p-5 rounded-2xl bg-white/[0.02] border border-white/5">
              <h3 className="text-sm font-semibold text-white mb-2">Accessibility</h3>
              <p className="text-xs text-slate-500">Direct fiat off-ramping to bank accounts and mobile wallets globally.</p>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
