"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import Link from "next/link";
import { ArrowLeft, Menu, X, ChevronDown, ChevronUp } from "lucide-react";
import { ConnectButton } from "@/components/ConnectButton";
import Footer from "../components/Footer";

interface FAQItem {
  question: string;
  answer: string;
}

function FAQAccordion({ question, answer }: FAQItem) {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <div className="border-b border-white/5 py-4 last:border-0">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex justify-between items-center text-left text-sm font-semibold text-white py-2 focus:outline-none"
      >
        <span>{question}</span>
        {isOpen ? <ChevronUp size={16} className="text-[#33C5E0]" /> : <ChevronDown size={16} className="text-slate-500" />}
      </button>
      {isOpen && (
        <p className="mt-2 text-xs text-slate-400 leading-relaxed pr-6 animate-fade-in">
          {answer}
        </p>
      )}
    </div>
  );
}

const FAQS_LIST: FAQItem[] = [
  {
    question: "How does liveness verification (Proof of Life) work?",
    answer: "Asset owners configure a liveness period (e.g. 180 days). To prove they are active, they periodically sign a liveness message from their dashboard. If no signature is recorded before the period ends, the plan becomes claimable.",
  },
  {
    question: "Do my beneficiaries need crypto wallets or blockchain literacy?",
    answer: "No. Heirs can configure their settlement using local Stellar Anchor off-ramps. The principal and yield can be transferred directly to bank accounts or mobile wallets in fiat currency (USD, EUR, KES, NGN, BRL, etc.).",
  },
  {
    question: "Is InheritX custodial?",
    answer: "No. InheritX is fully non-custodial. Your assets are secured by the Soroban smart contract, and you can withdraw or modify your plan parameters at any time before the claim conditions are met.",
  },
  {
    question: "How is yield generated on locked assets?",
    answer: "When configuring your plan, you can opt to enable yield-harvesting. Assets are deposited into audit-verified lending pools and liquidity vaults on Stellar, generating interest that accumulates directly to the plan principal.",
  },
];

export default function FAQsPage() {
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
              <Link href="/faqs" className="text-cyan-400 transition-colors px-1">FAQs</Link>
              <Link href="/Guidelines" className="hover:text-cyan-400 transition-colors px-1">Guidelines</Link>
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
              <Link href="/faqs" onClick={() => setIsMobileMenuOpen(false)} className="text-cyan-400 py-2 uppercase">FAQs</Link>
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
      <main className="max-w-3xl mx-auto px-6 py-20 relative z-10">
        <Link href="/" className="inline-flex items-center gap-2 text-xs text-[#33C5E0] hover:underline mb-8">
          <ArrowLeft size={12} /> Back to Home
        </Link>

        <h1 className="text-4xl font-bold text-white mb-6">Frequently Asked Questions</h1>
        <p className="text-sm text-slate-500 mb-10">Find answers to the most common questions about the InheritX protocol.</p>

        <div className="rounded-2xl bg-white/[0.02] border border-white/5 p-6 space-y-2">
          {FAQS_LIST.map((faq, i) => (
            <FAQAccordion key={i} question={faq.question} answer={faq.answer} />
          ))}
        </div>
      </main>

      <Footer />
    </div>
  );
}
