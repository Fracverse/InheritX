"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import Link from "next/link";
import { ArrowLeft, Menu, X } from "lucide-react";
import { ConnectButton } from "@/components/ConnectButton";
import Footer from "../components/Footer";

export default function PrivacyPolicyPage() {
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
              <Link href="/about" className="hover:text-cyan-400 transition-colors px-1">About</Link>
              <Link href="/faqs" className="hover:text-cyan-400 transition-colors px-1">FAQs</Link>
              <Link href="/Guidelines" className="text-cyan-400 transition-colors px-1">Guidelines</Link>
            </div>
          </div>

          <button
            className="md:hidden text-slate-300 hover:text-cyan-400 p-2 relative z-10"
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
            aria-label={isMobileMenuOpen ? "Close menu" : "Open menu"}
          >
            {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
          </button>

          {isMobileMenuOpen && (
            <div className="absolute top-full left-0 w-full bg-[#161E22] border-b border-[#2A3338] p-4 flex flex-col gap-4 md:hidden shadow-2xl z-10">
              <Link href="/" onClick={() => setIsMobileMenuOpen(false)} className="text-slate-300 hover:text-cyan-400 py-2 uppercase">Home</Link>
              <Link href="/about" onClick={() => setIsMobileMenuOpen(false)} className="text-slate-300 hover:text-cyan-400 py-2 uppercase">About</Link>
              <Link href="/faqs" onClick={() => setIsMobileMenuOpen(false)} className="text-slate-300 hover:text-cyan-400 py-2 uppercase">FAQs</Link>
              <Link href="/Guidelines" onClick={() => setIsMobileMenuOpen(false)} className="text-cyan-400 py-2 uppercase">Guidelines</Link>
              <ConnectButton />
            </div>
          )}
          <div className="md:block hidden">
            <ConnectButton />
          </div>
        </nav>
      </header>

      {/* Main Content */}
      <main className="max-w-3xl mx-auto px-6 py-20 relative z-10">
        <Link href="/" className="inline-flex items-center gap-2 text-xs text-[#33C5E0] hover:underline mb-8">
          <ArrowLeft size={12} /> Back to Home
        </Link>

        <h1 className="text-3xl font-bold text-white mb-6">Privacy Policy</h1>
        <p className="text-xs text-slate-500 mb-8">Last Updated: July 15, 2026</p>

        <div className="space-y-6 text-sm leading-relaxed text-slate-400">
          <section>
            <h2 className="text-base font-semibold text-white mb-2">1. Information We Collect</h2>
            <p>
              When you use our service, we record on-chain addresses, transaction hashes, and plan configuration details. In addition, when you submit identity documents for KYC processing, we collect verification details (such as full name, date of birth, and scan of ID documents) through secure compliant integrations.
            </p>
          </section>

          <section>
            <h2 className="text-base font-semibold text-white mb-2">2. How We Use Information</h2>
            <p>
              Your information is used solely to facilitate decentralized digital estate planning, comply with financial off-ramping regulations, and prevent fraudulent activity. We do not sell or monetize user details.
            </p>
          </section>

          <section>
            <h2 className="text-base font-semibold text-white mb-2">3. Storage & Security</h2>
            <p>
              All sensitive KYC data is encrypted at rest and in transit. On-chain plan settings are public by design on the Stellar Network, but personal identity records are stored and managed off-chain under strict access controls.
            </p>
          </section>
        </div>
      </main>

      <Footer />
    </div>
  );
}
