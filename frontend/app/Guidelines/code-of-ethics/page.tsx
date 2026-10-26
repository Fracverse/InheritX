"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import Link from "next/link";
import { ArrowLeft, Menu, X } from "lucide-react";
import { ConnectButton } from "@/components/ConnectButton";
import Footer from "../../components/Footer";

export default function CodeOfEthicsPage() {
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
        <Link href="/Guidelines" className="inline-flex items-center gap-2 text-xs text-[#33C5E0] hover:underline mb-8">
          <ArrowLeft size={12} /> Back to Guidelines
        </Link>

        <h1 className="text-3xl font-bold text-white mb-6">Code of Ethics</h1>
        <p className="text-xs text-slate-500 mb-8">Last Updated: July 15, 2026</p>

        <div className="space-y-6 text-sm leading-relaxed text-slate-400">
          <p>
            At InheritX, we are dedicated to setting the ethical benchmark for digital estate management. Protecting wealth for future generations demands transparency, integrity, and absolute security.
          </p>

          <section>
            <h2 className="text-base font-semibold text-white mb-2">1. Principle of Transparency</h2>
            <p>
              We commit to holding all core smart contracts open-source and auditable. Users must be able to verify how, when, and under what conditions their assets are stored, locked, and released.
            </p>
          </section>

          <section>
            <h2 className="text-base font-semibold text-white mb-2">2. Respect for User Privacy</h2>
            <p>
              We collect only the minimum required information necessary for compliance and KYC off-ramping. User data is treated with the highest standard of cryptographic secrecy, ensuring planning stays private until conditions trigger distribution.
            </p>
          </section>

          <section>
            <h2 className="text-base font-semibold text-white mb-2">3. Integrity of Intention</h2>
            <p>
              InheritX is designed strictly to facilitate the transition of wealth to intended heirs. We actively coordinate with compliance anchors to prevent our protocol from being used to bypass legitimate legal claims or facilitate illicit asset movement.
            </p>
          </section>
        </div>
      </main>

      <Footer />
    </div>
  );
}
