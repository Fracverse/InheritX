"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import {
  ArrowUpRight,
  ArrowDownRight,
  ShieldCheck,
  Settings,
  Zap,
  ThumbsUp,
  TreePine,
  Leaf,
  Sprout,
  Globe,
  Shield,
  Menu,
  X,
} from "lucide-react";

// --- Reusable Components ---

const FeatureCard = ({
  icon: Icon,
  title,
  desc,
  index = 0,
}: {
  icon: React.ComponentType<{
    className?: string;
    size?: number;
    "aria-hidden"?: boolean;
  }>;
  title: string;
  desc: string;
  index?: number;
}) => (
  <div
    className="py-15 px-8 border-13 border-[#1C252A] bg-[#161E22] group text-center flex flex-col items-center shadow-[inset_0_2px_20px_rgba(0,0,0,0.15)] transition-all duration-300 hover:border-cyan-400/30 hover:shadow-[inset_0_2px_30px_rgba(51,197,224,0.1)] animate-slide-up focus-within:outline-offset-2 focus-within:outline-2 focus-within:outline-cyan-400"
    style={{
      animationDelay: `${index * 0.1}s`,
    }}
  >
    <div className="bg-transparent p-3 w-fit mb-4 transition-transform duration-300 group-hover:scale-110 group-hover:animate-float">
      <Icon className="text-cyan-400" size={32} aria-hidden={true} />
    </div>
    <h4 className="text-[#FCFFFF] text-[18px] font-bold mb-2">{title}</h4>
    <p className="text-[#92A5A8] text-[14px] leading-relaxed px-2">{desc}</p>
  </div>
);

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
          className="opacity-50 pointer-events-none"
          priority
          quality={75}
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
                href="#how-it-works"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                How it Works
              </a>
              <a
                href="#"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                FAQs
              </a>
              <a
                href="#footer"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Contact
              </a>
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
                href="#how-it-works"
                onClick={closeMenu}
                className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
              >
                How it Works
              </a>
              <a
                href="#"
                onClick={closeMenu}
                className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
              >
                FAQs
              </a>
              <a
                href="#footer"
                onClick={closeMenu}
                className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
              >
                Contact
              </a>
              <button
                className="flex justify-center items-center gap-4 text-[14px] border-[0.5px] border-[#33C5E03D] bg-[#161E22] px-4 py-3 rounded-lg text-slate-300 hover:border-cyan-400 transition-all w-full focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 active:scale-95"
                aria-label="Connect wallet"
              >
                Connect Wallet <ArrowDownRight size={16} aria-hidden={true} />
              </button>
            </div>
          )}

          <button
            className="hidden md:flex justify-center items-center gap-4 text-[14px] border-[0.5px] border-[#33C5E03D] bg-[#161E22] px-4 py-3 rounded-l-xl rounded-r-md text-slate-300 hover:border-cyan-400 transition-all cursor-pointer focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 active:scale-95 relative z-10"
            aria-label="Connect wallet"
          >
            Connect Wallet <ArrowDownRight size={16} aria-hidden={true} />
          </button>
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

      {/* About Section */}
      <section
        id="about"
        className="mt-24 md:mt-100 py-16 md:py-24 px-8 relative z-10"
        role="region"
        aria-label="About InheritX"
      >
        <div className="max-w-3xl mx-auto">
          <h2 className="text-[#FCFFFF] uppercase tracking-[0.3em] text-[32px] mb-4 animate-slide-up">
            What is InheritX?
          </h2>
          <h3
            className="text-[#92A5A8] text-[14px] font-bold mb-6 animate-slide-up"
            style={{ animationDelay: "0.1s" }}
          >
            Without roots, nothing grows.
          </h3>
          <p
            className="text-[18px] text-[#FCFFFF] leading-relaxed mb-8 animate-slide-up"
            style={{ animationDelay: "0.2s" }}
          >
            Inherit X helps you plan and share your assets with the right
            people, at the right time. We make inheritance simple, secure, and
            stress-free — without unnecessary delays or complications. Think of
            it as planting a tree: your roots are the assets you&apos;ve built,
            and we make sure the branches grow to those you care about most.
          </p>
          <div
            className="text-[18px] text-[#FCFFFF] animate-slide-up"
            style={{ animationDelay: "0.3s" }}
          >
            Think of it as planting a tree: your roots are the assets
            you&apos;ve built, and we make sure the branches grow to those you
            care about most.
          </div>
        </div>
        {/* Decorative tree-like background glow */}
        <div className="w-full absolute top-0 left-0 pointer-events-none">
          <Image
            src="/Vector (1).svg"
            alt=""
            role="presentation"
            width={500}
            height={100}
            className="opacity-50 pointer-events-none"
            quality={75}
          />
        </div>
      </section>

      {/* How it Works Section */}
      <section
        id="how-it-works"
        className="py-24 px-8 mx-auto relative z-10"
        role="region"
        aria-label="How it works"
      >
        {/* Decorative tree-like background glow */}
        <div className="w-full absolute top-110 left-0 pointer-events-none">
          <Image
            src="/tree2.svg"
            alt=""
            role="presentation"
            width={1000}
            height={1000}
            className="opacity-50 pointer-events-none"
            quality={75}
          />
        </div>

        <div className="relative max-w-7xl mx-auto px-4">
          {/* Vertical/Horizontal connector line */}
          <div
            className="md:hidden absolute top-0 bottom-0 left-1/2 w-0.5 bg-gradient-to-b from-slate-800/30 via-slate-700/50 to-slate-800/30 -translate-x-1/2 h-full z-0 will-change-transform"
            style={{ filter: "blur(0.5px)" }}
            role="presentation"
            aria-hidden={true}
          ></div>
          <div
            className="hidden md:block absolute top-1/2 left-0 w-full h-[2px] bg-gradient-to-r from-slate-800/30 via-slate-700/50 to-slate-800/30 -translate-y-1/2 will-change-transform"
            style={{ filter: "blur(1px)" }}
            role="presentation"
            aria-hidden={true}
          ></div>

          <div className="grid grid-cols-1 md:grid-cols-4 gap-8 md:gap-4 items-center relative stagger-children">
            {/* How It Works Circle - Left Side */}
            <div className="text-center relative z-10 animate-scale-in">
              <div className="w-[250px] h-[250px] md:w-[324px] md:h-[324px] rounded-full border-[13px] border-[#1C252A] bg-[#161E22] flex flex-col items-center justify-center mx-auto shadow-[inset_0_2px_20px_rgba(0,0,0,0.15)] transition-all duration-300 hover:border-cyan-400/30 hover:shadow-[inset_0_2px_40px_rgba(51,197,224,0.15)]">
                <TreePine
                  className="text-[#33C5E0] mb-2 transition-transform duration-300 hover:animate-float"
                  size={40}
                  aria-hidden={true}
                />
                <span className="text-[#FCFFFF] font-bold text-[18px]">
                  How It Works
                </span>
                <span className="text-[#FCFFFF] text-[14px] mt-1">
                  Here&apos;s how your legacy flows
                </span>
              </div>
            </div>

            {/* Step 1 - Bottom Position */}
            <div className="text-center relative group z-10 animate-scale-in">
              {/* Connecting dot on line */}
              <div
                className="w-4 h-4 bg-slate-700 rounded-full mx-auto md:absolute md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 group-hover:bg-cyan-400 transition-all duration-300 z-20 mb-4 md:mb-0 will-change-colors"
                role="presentation"
                aria-hidden={true}
              ></div>

              {/* Content below line */}
              <div className="flex flex-col items-center">
                <div className="mb-4 md:mb-50 border-[3px] border-[#1C252A] bg-[#161E22] w-[80px] h-[80px] rounded-full flex items-center justify-center mx-auto shadow-lg transition-all duration-300 group-hover:border-cyan-400/50 group-hover:shadow-[0_0_20px_rgba(51,197,224,0.3)]">
                  <Sprout
                    className="text-cyan-400 transition-transform duration-300 group-hover:scale-110"
                    size={24}
                    aria-hidden={true}
                  />
                </div>
                <h4 className="text-[#FCFFFF] font-semibold text-[18px] mb-2">
                  1. Plant the Roots
                </h4>
                <p className="text-[14px] text-[#92A5A8] max-w-[200px] mx-auto">
                  Add Your Assets And Choose Your Beneficiaries.
                </p>
              </div>
            </div>

            {/* Step 2 - Top Position */}
            <div className="text-center relative group z-10 animate-scale-in">
              {/* Connecting dot on line */}
              <div
                className="w-4 h-4 bg-slate-700 rounded-full mx-auto md:absolute md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 group-hover:bg-cyan-400 transition-all duration-300 z-20 mb-4 md:mb-0 will-change-colors"
                role="presentation"
                aria-hidden={true}
              ></div>

              {/* Content above line */}
              <div className="flex flex-col md:flex-col-reverse items-center">
                <div className="mb-4 md:mt-50 border-[3px] border-[#1C252A] bg-[#161E22] w-[80px] h-[80px] rounded-full flex items-center justify-center mx-auto shadow-lg transition-all duration-300 group-hover:border-cyan-400/50 group-hover:shadow-[0_0_20px_rgba(51,197,224,0.3)]">
                  <Leaf
                    className="text-cyan-400 transition-transform duration-300 group-hover:scale-110"
                    size={24}
                    aria-hidden={true}
                  />
                </div>
                <p className="text-[14px] text-[#92A5A8] max-w-[200px] mx-auto">
                  Set Clear Rules For Who Gets What And When.
                </p>
                <h4 className="text-[#FCFFFF] font-semibold text-[18px] mb-2">
                  2. Grow the Branches
                </h4>
              </div>
            </div>

            {/* Step 3 - Bottom Position */}
            <div className="text-center relative group z-10 animate-scale-in">
              {/* Connecting dot on line */}
              <div
                className="w-4 h-4 bg-slate-700 rounded-full mx-auto md:absolute md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 group-hover:bg-cyan-400 transition-all duration-300 z-20 mb-4 md:mb-0 will-change-colors"
                role="presentation"
                aria-hidden={true}
              ></div>

              {/* Content below line */}
              <div className="flex flex-col items-center">
                <div className="mb-4 md:mb-50 border-[3px] border-[#1C252A] bg-[#161E22] w-[80px] h-[80px] rounded-full flex items-center justify-center mx-auto shadow-lg transition-all duration-300 group-hover:border-cyan-400/50 group-hover:shadow-[0_0_20px_rgba(51,197,224,0.3)]">
                  <TreePine
                    className="text-cyan-400 transition-transform duration-300 group-hover:scale-110"
                    size={24}
                    aria-hidden={true}
                  />
                </div>
                <h4 className="text-[#FCFFFF] font-semibold text-[18px] mb-2">
                  3. Watch It Bloom
                </h4>
                <p className="text-[14px] text-[#92A5A8] max-w-[200px] mx-auto">
                  We Handle The Rest — Ensuring Smooth, Secure Transfers.
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Why This Works Section */}
        <div className="relative my-24 md:my-50 flex flex-col items-center md:items-end justify-center p-4 z-10">
          <div className="static md:absolute md:-right-10 space-y-8 max-w-full animate-slide-up">
            <div className="border-r-[0px] border-[13px] border-[#1C252A] rounded-l-[18px] py-6 px-14 shadow-[inset_0_2px_20px_rgba(0,0,0,0.15)] transition-all duration-300 hover:border-cyan-400/30 hover:shadow-[inset_0_2px_30px_rgba(51,197,224,0.1)]">
              <h3 className="text-[18px] font-bold text-[#FCFFFF] mb-4">
                Why this works:
              </h3>
              <ul className="space-y-2 text-[14px] text-[#92A5A8]">
                <li className="flex gap-2 transition-all duration-300 hover:text-cyan-400">
                  <span className="">•</span>
                  <span>
                    Starts with purpose &mdash; &ldquo;Helping your legacy reach
                    the people who matter&rdquo; sets an emotional tone
                  </span>
                </li>
                <li className="flex gap-2 transition-all duration-300 hover:text-cyan-400">
                  <span>•</span>
                  <span>One short paragraph — easy to skim.</span>
                </li>
                <li className="flex gap-2 transition-all duration-300 hover:text-cyan-400">
                  <span>•</span>
                  <span>
                    Metaphor tie-in &mdash; connects to your tree concept in a
                    natural way.
                  </span>
                </li>
              </ul>
            </div>

            <button
              className="flex flex-row justify-center items-center gap-4 bg-cyan-400 text-black px-8 py-2 rounded-t-[8px] rounded-b-[16px] cursor-pointer transition-all duration-300 hover:bg-cyan-300 active:scale-95 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 font-semibold"
              aria-label="Get started with InheritX"
            >
              GET STARTED
              <ArrowUpRight size={16} aria-hidden={true} />
            </button>
          </div>
        </div>
      </section>

      {/* Benefits Section */}
      <section
        id="benefits"
        className="py-12 md:py-24 px-4 md:px-8 bg-[#161E22] relative z-10"
        role="region"
        aria-label="Benefits of InheritX"
      >
        <div className="">
          <div className="md:w-[796px] w-full mx-auto bg-[#1a2329] mb-8 md:mb-16 rounded-3xl p-6 md:p-12 border border-[#1C252A] transition-all duration-300 hover:border-cyan-400/30 hover:shadow-[inset_0_2px_40px_rgba(51,197,224,0.05)] animate-slide-up">
            <h2 className="text-[32px] font-bold text-[#FCFFFF] mb-2">
              Benefits Of InheritX
            </h2>
            <p className="text-[#92A5A8] text-[14px] uppercase tracking-wider">
              Here&apos;s why you should choose us
            </p>

            <div className="mt-6 max-w-2xl text-slate-400 leading-relaxed text-sm">
              <p className="mb-4 text-[#FCFFFF] text-[18px]">
                <span className="font-semibold">Inherit X</span>
                helps you plan and share your assets with the right people, at
                the right time. We make inheritance simple, secure, and
                stress-free — without unnecessary delays or complications.
              </p>
              <p className="text-[#FCFFFF] text-[18px]">
                Think of it as planting a tree: your roots are the assets
                you&apos;ve built, and we make sure the branches grow to those
                you care about most.
              </p>
            </div>
          </div>

          <div className="max-w-7xl mx-auto relative rounded-3xl overflow-hidden">
            {/* The Grid Content */}
            <div className="grid md:grid-cols-4">
              <FeatureCard
                icon={ThumbsUp}
                title="Easy to Use"
                desc="No Confusing Legal Jargon — Just Clear Steps Everyone Can Follow."
                index={0}
              />
              <FeatureCard
                icon={ShieldCheck}
                title="Secure & Private"
                desc="Your Data And Assets Are Protected With Top-Level Encryption."
                index={1}
              />
              <FeatureCard
                icon={Settings}
                title="Custom Plans"
                desc="Get Your Rules For Who Gets What, And When."
                index={2}
              />
              <FeatureCard
                icon={Zap}
                title="Stress-free Transfers"
                desc="We Handle The Details So Your Legacy Reaches The Right Hands."
                index={3}
              />
            </div>
          </div>

          <div
            className="mt-16 flex justify-center animate-fade-in"
            style={{ animationDelay: "0.5s" }}
          >
            <button
              className="flex flex-row justify-center items-center gap-4 bg-cyan-400 text-black px-8 py-2 rounded-t-[8px] rounded-b-[16px] cursor-pointer transition-all duration-300 hover:bg-cyan-300 active:scale-95 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 font-semibold"
              aria-label="Create your inheritance plan"
            >
              CREATE YOUR PLAN
              <ArrowUpRight size={16} aria-hidden={true} />
            </button>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer
        id="footer"
        className="w-full py-20 relative z-10"
        role="contentinfo"
        aria-label="Footer"
      >
        {/* Decorative tree-like background glow */}
        <div className="w-full flex justify-between items-center pointer-events-none">
          <div className="">
            <Image
              src="/Vector (3).svg"
              alt=""
              role="presentation"
              width={500}
              height={100}
              className="opacity-50 pointer-events-none"
              quality={75}
            />
          </div>
          <div className="">
            <Image
              src="/Vector (2).svg"
              alt=""
              role="presentation"
              width={500}
              height={100}
              className="opacity-50 pointer-events-none"
              quality={75}
            />
          </div>
        </div>
        <div className="md:w-[1280px] w-full h-auto md:h-[300px] mx-auto rounded-[60px] py-12 px-8 bg-[#182024] transition-all duration-300 hover:bg-[#1a2a2f] animate-fade-in">
          {/* Top Footer Part */}
          <div className="flex flex-col md:flex-row justify-between items-center gap-8 mb-12 border-b border-[#2A3338] pb-4">
            {/* Quick Links */}
            <div className="text-center md:text-left">
              <h5 className="text-[14px] text-[#92A5A8] uppercase tracking-widest mb-4">
                Quick Links
              </h5>
              <nav
                className="flex gap-4 text-[14px] uppercase font-bold text-[#FCFFFF]"
                aria-label="Footer navigation"
              >
                <a
                  href="#hero"
                  className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
                >
                  Home
                </a>
                <a
                  href="#about"
                  className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
                >
                  About
                </a>
                <a
                  href="#"
                  className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
                >
                  FAQs
                </a>
                <a
                  href="#"
                  className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
                >
                  Guidelines
                </a>
                <a
                  href="#"
                  className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
                >
                  Support
                </a>
              </nav>
            </div>

            {/* Logo Center */}
            <div className="text-cyan-400 font-bold text-4xl italic tracking-tighter">
              <a
                href="#hero"
                className="focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm"
              >
                <Image
                  src="/logo.svg"
                  alt="InheritX"
                  width={93}
                  height={93}
                  quality={85}
                />
              </a>
            </div>

            {/* Social Links */}
            <div className="text-center md:text-right">
              <h5 className="text-[14px] text-[#92A5A8] uppercase tracking-widest mb-4">
                Social Links
              </h5>
              <nav
                className="flex gap-4 justify-center md:justify-end text-slate-300"
                aria-label="Social media links"
              >
                <a
                  href="https://example.com"
                  className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm p-1"
                  aria-label="Website"
                >
                  <Globe size={16} aria-hidden={true} />
                </a>
                <a
                  href="https://example.com"
                  className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm p-1"
                  aria-label="Security"
                >
                  <Shield size={16} aria-hidden={true} />
                </a>
                <a
                  href="https://example.com"
                  className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm p-1"
                  aria-label="Blog"
                >
                  <Zap size={16} aria-hidden={true} />
                </a>
              </nav>
            </div>
          </div>

          {/* Bottom Footer Part */}
          <div className="flex flex-col md:flex-row justify-between items-center text-[12px] text-[#92A5A8] gap-4">
            <div>Copyright © InheritX 2026. All Rights Reserved.</div>
            <nav className="flex gap-6" aria-label="Legal and policy links">
              <a
                href="#"
                className="hover:text-slate-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Privacy Policy
              </a>
              <a
                href="#"
                className="hover:text-slate-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Terms & Conditions
              </a>
              <a
                href="#"
                className="hover:text-slate-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Code of Ethics
              </a>
            </nav>
          </div>
        </div>
      </footer>
    </div>
  );
}
