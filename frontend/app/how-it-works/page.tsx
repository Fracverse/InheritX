"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import Link from "next/link";
import { ArrowLeft, Menu, X, ArrowUpRight, Shield, RefreshCw, Landmark, Heart, Users } from "lucide-react";
import { ConnectButton } from "@/components/ConnectButton";
import Footer from "../components/Footer";

interface StepItem {
  number: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  details: string[];
}

export default function HowItWorksPage() {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [isScrolled, setIsScrolled] = useState(false);
  const [activeStep, setActiveStep] = useState(0);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 50);
    };
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const steps: StepItem[] = [
    {
      number: "01",
      title: "Create and Fund Your Plan",
      description: "Initialize your secure inheritance vault on the Soroban smart contract with your chosen digital assets.",
      icon: <Landmark className="text-[#33C5E0]" size={28} />,
      details: [
        "Connect your Stellar wallet (Freighter, Albedo, etc.) securely.",
        "Deposit USDC or other supported Stellar tokens into your dedicated plan.",
        "Opt-in to earn interest automatically through integrated yield-generating pools.",
        "Withdraw or edit your principal amount at any time during your active plan."
      ]
    },
    {
      number: "02",
      title: "Configure Beneficiaries & Rules",
      description: "Define exactly who receives your assets and what proportion they get.",
      icon: <Users className="text-[#33C5E0]" size={28} />,
      details: [
        "Add one or multiple beneficiary wallet addresses.",
        "Specify the exact allocation split (measured in basis points, e.g., 5000 bps for 50%).",
        "Set the custom inactivity watchdog timeout (e.g., 180 days).",
        "Set a grace period to prevent accidental triggers."
      ]
    },
    {
      number: "03",
      title: "Maintain Proof of Life",
      description: "Periodically check in to prove you are active and reset the inactivity countdown.",
      icon: <Heart className="text-[#33C5E0]" size={28} />,
      details: [
        "Check-in easily by sending a simple 'ping' transaction to the smart contract.",
        "Pinging updates the contract's checkpoint timestamp and restarts the countdown.",
        "Receive automated friendly notifications when your check-in deadline approaches.",
        "Accrued yield is compounded and locked securely under your control."
      ]
    },
    {
      number: "04",
      title: "Automated Settlement & Off-Ramp",
      description: "Heirs claim assets seamlessly, with direct conversion to local fiat cash if desired.",
      icon: <RefreshCw className="text-[#33C5E0]" size={28} />,
      details: [
        "If the watchdog timer expires, the contract unlocks the plan for settlement.",
        "Beneficiaries can trigger the claim directly from the dApp dashboard.",
        "Integrate with Stellar Anchors to off-ramp digital assets into local bank accounts or mobile wallets.",
        "No blockchain or crypto knowledge required for beneficiaries to receive funds."
      ]
    }
  ];

  return (
    <div className="relative min-h-screen bg-[#161E22] text-slate-300 selection:text-black overflow-x-hidden">
      {/* Decorative background glow */}
      <div className="w-full absolute top-0 left-0 z-0 opacity-40 pointer-events-none">
        <Image
          src="/tree.svg"
          alt=""
          role="presentation"
          width={2400}
          height={1000}
          className="w-full h-auto"
          priority
        />
      </div>

      {/* Navigation */}
      <header className={`sticky top-0 z-50 backdrop-blur-xs transition-all duration-300 ${isScrolled ? "bg-[#161E22]/80 shadow-lg" : "bg-[#161E22]/40"}`} role="banner">
        <nav className="flex justify-between items-center px-6 md:px-40 py-6 mx-auto" role="navigation" aria-label="Main navigation">
          <div className="flex items-center gap-12 relative z-10">
            <Link href="/" className="focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm">
              <Image src="/logo.svg" alt="InheritX" width={50} height={50} quality={75} />
            </Link>
            <div className="hidden md:flex gap-8 text-xs font-medium uppercase tracking-widest text-slate-400">
              <Link href="/" className="hover:text-cyan-400 transition-colors px-1">Home</Link>
              <Link href="/how-it-works" className="text-cyan-400 transition-colors px-1">How it Works</Link>
              <Link href="/faqs" className="hover:text-cyan-400 transition-colors px-1">FAQs</Link>
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
              <Link href="/how-it-works" onClick={() => setIsMobileMenuOpen(false)} className="text-cyan-400 py-2 uppercase">How it Works</Link>
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
      <main className="max-w-6xl mx-auto px-6 py-16 relative z-10">
        <Link href="/" className="inline-flex items-center gap-2 text-xs text-[#33C5E0] hover:underline mb-8">
          <ArrowLeft size={12} /> Back to Home
        </Link>

        {/* Hero Section */}
        <section className="text-center max-w-3xl mx-auto mb-20">
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-6 leading-tight">
            How <span className="text-[#33C5E0]">InheritX</span> Works
          </h1>
          <p className="text-base text-slate-400 leading-relaxed">
            A secure, decentralized protocol powered by Stellar Soroban smart contracts. 
            Ensure your digital wealth safely reaches the people who matter most, with zero intermediaries.
          </p>
        </section>

        {/* Step-by-Step Interactive Flow */}
        <section className="mb-24">
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
            {/* Step navigation */}
            <div className="lg:col-span-5 space-y-3">
              {steps.map((step, idx) => (
                <button
                  key={idx}
                  onClick={() => setActiveStep(idx)}
                  className={`w-full text-left p-6 rounded-2xl border transition-all duration-300 flex items-center gap-5 ${
                    activeStep === idx
                      ? "bg-white/[0.04] border-[#33C5E0]/40 shadow-[0_4px_20px_rgba(51,197,224,0.08)]"
                      : "bg-white/[0.01] border-white/5 hover:bg-white/[0.02] hover:border-white/10"
                  }`}
                >
                  <span className={`text-xl font-bold ${activeStep === idx ? "text-[#33C5E0]" : "text-slate-600"}`}>
                    {step.number}
                  </span>
                  <div>
                    <h3 className={`font-semibold text-sm ${activeStep === idx ? "text-white" : "text-slate-400"}`}>
                      {step.title}
                    </h3>
                  </div>
                </button>
              ))}
            </div>

            {/* Active Step Details Card */}
            <div className="lg:col-span-7 bg-white/[0.02] border border-white/5 rounded-3xl p-8 md:p-10 min-h-[380px] flex flex-col justify-between shadow-2xl relative overflow-hidden">
              <div className="absolute top-0 right-0 w-32 h-32 bg-[#33C5E0]/5 rounded-full blur-3xl pointer-events-none"></div>
              
              <div>
                <div className="flex items-center justify-between mb-6">
                  <div className="w-14 h-14 bg-white/[0.03] rounded-2xl flex items-center justify-center border border-white/5">
                    {steps[activeStep].icon}
                  </div>
                  <span className="text-5xl font-extrabold text-white/[0.02] tracking-tighter select-none">
                    {steps[activeStep].number}
                  </span>
                </div>

                <h2 className="text-2xl font-bold text-white mb-3">
                  {steps[activeStep].title}
                </h2>
                <p className="text-sm text-slate-400 mb-8 leading-relaxed">
                  {steps[activeStep].description}
                </p>

                <ul className="space-y-3">
                  {steps[activeStep].details.map((detail, dIdx) => (
                    <li key={dIdx} className="flex items-start gap-3 text-xs text-slate-300">
                      <span className="text-[#33C5E0] mt-0.5">•</span>
                      <span>{detail}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </section>

        {/* Security & Protocol Highlights */}
        <section className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-20">
          <div className="bg-white/[0.01] border border-white/5 rounded-2xl p-6 hover:border-[#33C5E0]/20 transition-all duration-300">
            <Shield className="text-[#33C5E0] mb-4" size={32} />
            <h3 className="text-lg font-bold text-white mb-2">Non-Custodial Security</h3>
            <p className="text-xs text-slate-400 leading-relaxed">
              Your assets remain locked in the audited Soroban contract. You retain full ownership, keys, and authorization controls until conditions are met.
            </p>
          </div>

          <div className="bg-white/[0.01] border border-white/5 rounded-2xl p-6 hover:border-[#33C5E0]/20 transition-all duration-300">
            <Landmark className="text-[#33C5E0] mb-4" size={32} />
            <h3 className="text-lg font-bold text-white mb-2">Direct Fiat Settlement</h3>
            <p className="text-xs text-slate-400 leading-relaxed">
              Stellar Anchors enable seamless transfers to local bank accounts, completely eliminating blockchain complexity for your beneficiaries.
            </p>
          </div>

          <div className="bg-white/[0.01] border border-white/5 rounded-2xl p-6 hover:border-[#33C5E0]/20 transition-all duration-300">
            <RefreshCw className="text-[#33C5E0] mb-4" size={32} />
            <h3 className="text-lg font-bold text-white mb-2">Accrued Yield Compounding</h3>
            <p className="text-xs text-slate-400 leading-relaxed">
              Opt-in to yield-harvesting parameters that grow your locked principal continuously over time, ensuring a larger inheritance pool.
            </p>
          </div>
        </section>

        {/* CTA */}
        <section className="bg-gradient-to-r from-white/[0.02] to-white/[0.01] border border-white/5 rounded-3xl p-10 text-center relative overflow-hidden mb-8">
          <div className="absolute inset-0 bg-[#33C5E0]/5 opacity-30 blur-2xl"></div>
          <h2 className="text-2xl md:text-3xl font-bold text-white mb-4 relative z-10">
            Start Planning Your Legacy Today
          </h2>
          <p className="text-xs text-slate-400 max-w-lg mx-auto mb-8 relative z-10">
            Secure, protect, and grow your wealth. Build a robust structure that ensures your assets flow seamlessly to your loved ones.
          </p>
          <Link
            href="/asset-owner/plans/create"
            className="inline-flex items-center gap-2 bg-[#33C5E0] hover:bg-[#2cb2cb] text-black font-semibold px-8 py-3 rounded-xl transition-all duration-300 active:scale-95 z-10 relative"
          >
            CREATE A PLAN
            <ArrowUpRight size={16} />
          </Link>
        </section>
      </main>

      <Footer />
    </div>
  );
}
