import React, { useState, useEffect } from "react";
import Image from "next/image";
import { ArrowDownRight, Globe, Shield, Zap, Menu, X } from "lucide-react";
import Link from "next/link";
import { ConnectButton } from "@/components/ConnectButton";

const Navbar = () => {
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
          <Link
            href="/"
            className="focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm"
          >
            <Image
              src="/logo.svg"
              alt="InheritX"
              width={50}
              height={50}
              quality={85}
            />
          </Link>
          <div className="hidden md:flex gap-8 text-xs font-medium uppercase tracking-widest text-slate-400">
            <Link
              href="/"
              className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
            >
              Home
            </Link>
            <Link
              href="/how-it-works"
              className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
            >
              How it Works
            </Link>
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
            <Link
              href="/"
              onClick={closeMenu}
              className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
            >
              Home
            </Link>
            <Link
              href="/how-it-works"
              onClick={closeMenu}
              className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
            >
              How it Works
            </Link>
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

            <button
              className="flex justify-center items-center gap-4 text-[14px] border-[0.5px] border-[#33C5E03D] bg-[#161E22] px-4 py-3 rounded-lg text-slate-300 hover:border-cyan-400 transition-all w-full focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 active:scale-95"
              aria-label="Connect wallet"
            >
              Connect Wallet <ArrowDownRight size={16} aria-hidden={true} />
            </button>
          </div>
        )}

        <ConnectButton />
      </nav>
    </header>
  );
};

export default Navbar;
