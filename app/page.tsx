"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import dynamic from "next/dynamic";
import {
  ArrowUpRight,
  Menu,
  X,
} from "lucide-react";
import { ConnectButton } from "@/components/ConnectButton";
import Link from "next/link";
import Footer from "./components/Footer";

// --- Dynamic Imports for Performance ---

const AboutSection = dynamic(() => import("./components/landing/AboutSection"), {
  loading: () => <div className="h-96 animate-pulse bg-slate-800/20 rounded-3xl m-8" />,
  ssr: true,
});

const HowItWorksSection = dynamic(() => import("./components/landing/HowItWorksSection"), {
  loading: () => <div className="h-screen animate-pulse bg-slate-800/10" />,
  ssr: true,
});

const BenefitsSection = dynamic(() => import("./components/landing/BenefitsSection"), {
  loading: () => <div className="h-96 animate-pulse bg-slate-800/20 rounded-3xl m-8" />,
  ssr: true,
});

// --- Main Page ---

export default function InheritXLanding() {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [isScrolled, setIsScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 50);
    };

    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const closeMenu = () => setIsMobileMenuOpen(false);

  return (
    <div className="relative min-h-screen bg-[#161E22] text-slate-300 selection:text-black overflow-x-hidden">
      {/* Decorative tree-like background glow */}
      <div className="w-full absolute top-0 left-0 z-0">
        <Image
          src="/tree.svg"
          alt=""
          role="presentation"
          width={2400}
          height={1000}
          className="opacity-50 pointer-events-none w-full h-auto"
          priority
          quality={75}
          sizes="100vw"
        />
      </div>

      {/* Navigation */}
      <header
        className={`sticky top-0 z-50 backdrop-blur-xs transition-all duration-300 ${
          isScrolled ? "bg-[#161E22]/60 shadow-lg" : "bg-[#161E22]/40"
        }`}
        role="banner"
      >
        <nav
          className="flex justify-between items-center px-6 md:px-40 py-6 mx-auto"
          role="navigation"
          aria-label="Main navigation"
        >
          <div className="flex items-center gap-12 relative z-10">
            <a
              href="#hero"
              className="focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm"
            >
              <Image
                src="/logo.svg"
                alt="InheritX"
                width={50}
                height={50}
                quality={85}
              />
            </a>
            <div className="hidden md:flex gap-8 text-xs font-medium uppercase tracking-widest text-slate-400">
              <a
                href="#hero"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Home
              </a>
              <a
                href="/how-it-works"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                How it Works
              </a>
              <Link
                href="/faqs"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                FAQs
              </Link>
              <Link
                href="/contact"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Contact
              </Link>
            </div>
          </div>

          <button
            className="md:hidden text-slate-300 hover:text-cyan-400 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm p-2 relative z-10"
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
            aria-label={isMobileMenuOpen ? "Close menu" : "Open menu"}
            aria-expanded={isMobileMenuOpen}
            aria-controls="mobile-menu"
          >
            {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
          </button>

          {/* Mobile Navigation Menu */}
          {isMobileMenuOpen && (
            <div
              id="mobile-menu"
              className="absolute top-full left-0 w-full bg-[#161E22] border-b border-[#2A3338] p-4 flex flex-col gap-4 md:hidden shadow-2xl animate-slide-up z-10"
            >
              <a
                href="#hero"
                onClick={closeMenu}
                className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
              >
                Home
              </a>
              <a
                href="/how-it-works"
                onClick={closeMenu}
                className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
              >
                How it Works
              </a>
              <Link
                href="/faqs"
                onClick={closeMenu}
                className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
              >
                FAQs
              </Link>
              <Link
                href="/contact"
                onClick={closeMenu}
                className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
              >
                Contact
              </Link>
              <ConnectButton />
            </div>
          )}
          <div className="md:block hidden">
            <ConnectButton />
          </div>
        </nav>
      </header>

      {/* Hero Section */}
      <section
        id="hero"
        className="w-full h-full relative pb-20 md:pb-32 px-6 md:px-8 bg-transparent"
        role="region"
        aria-label="Hero section"
      >
        <div className="max-w-7xl mx-auto relative z-10 pt-28 md:pt-48 flex flex-col items-start">
          <h1 className="text-4xl md:text-7xl font-bold text-white mb-6 leading-[1.2] md:leading-[1.1] animate-slide-up">
            From your hands, <br />
            to theirs — without a hitch.
          </h1>
          <p
            className="text-[16px] md:text-[18px] text-[#FCFFFF] mb-8 md:mb-10 leading-relaxed max-w-md md:max-w-none animate-slide-up"
            style={{ animationDelay: "0.2s" }}
          >
            Inherit X helps your wealth flow naturally to the people
            <br className="hidden md:block" /> who matter most.
          </p>
          <button
            className="flex items-center gap-2 px-8 py-3 rounded-t-lg rounded-b-[18px] bg-[#33C5E0] text-black font-semibold cursor-pointer transition-all duration-300 hover:bg-cyan-300 active:scale-95 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 animate-slide-up"
            style={{ animationDelay: "0.3s" }}
            aria-label="Start creating your inheritance plan now"
          >
            Start Now <ArrowUpRight size={16} aria-hidden={true} />
          </button>
        </div>
      </section>

      {/* Lazy Loaded Sections */}
      <AboutSection />
      <HowItWorksSection />
      <BenefitsSection />

      {/* Footer */}
      <Footer />
    </div>
  );
}
