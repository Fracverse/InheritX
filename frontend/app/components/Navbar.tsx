"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import { Menu, X } from "lucide-react";
import Link from "next/link";
import { ConnectButton } from "@/components/ConnectButton";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { useTranslations } from "next-intl";

const Navbar = () => {
  const t = useTranslations("navbar");
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
              {t("home")}
            </Link>
            <Link
              href="/how-it-works"
              className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
            >
              {t("howItWorks")}
            </Link>
            <Link
              href="/faqs"
              className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
            >
              {t("faqs")}
            </Link>
            <Link
              href="/contact"
              className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
            >
              {t("contact")}
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
              {t("home")}
            </Link>
            <Link
              href="/how-it-works"
              onClick={closeMenu}
              className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
            >
              {t("howItWorks")}
            </Link>
            <Link
              href="/faqs"
              onClick={closeMenu}
              className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
            >
              {t("faqs")}
            </Link>
            <Link
              href="/contact"
              onClick={closeMenu}
              className="text-slate-300 hover:text-cyan-400 py-2 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-2 uppercase"
            >
              {t("contact")}
            </Link>
            <LanguageSwitcher />
          </div>
        )}

        <div className="flex items-center gap-3">
          <div className="hidden md:block">
            <LanguageSwitcher />
          </div>
          <ConnectButton />
        </div>
      </nav>
    </header>
  );
};

export default Navbar;
