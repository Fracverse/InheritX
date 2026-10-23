"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import {
  AlertTriangle,
  ArrowDownRight,
  ExternalLink,
  Menu,
  RefreshCw,
  X,
} from "lucide-react";
import Link from "next/link";
import { ConnectButton } from "@/components/ConnectButton";
import { useWallet } from "@/context/WalletContext";
import { useAccountBalance } from "@/hooks/useAccountBalance";
import { STELLAR_FAUCET_URL } from "@/app/lib/stellar/network";

interface AccountBalanceBadgeProps {
  className?: string;
  isMobile?: boolean;
}

/**
 * Account balance pill displaying real-time native XLM balance.
 * Automatically highlights in orange with a direct link to the testnet
 * faucet when the XLM gas balance is under 5 XLM.
 */
function AccountBalanceBadge({ className = "", isMobile = false }: AccountBalanceBadgeProps) {
  const {
    xlmBalance,
    formattedBalance,
    isLoading,
    isRefreshing,
    isLowGas,
    refresh,
  } = useAccountBalance();

  if (isLoading && xlmBalance === null) {
    return (
      <div
        data-testid="balance-loading"
        className={`h-9 w-28 rounded-xl bg-white/5 border border-white/5 animate-pulse ${className}`}
        aria-label="Loading account balance"
      />
    );
  }

  if (xlmBalance === null) {
    return null;
  }

  // Low Gas Warning State (< 5 XLM)
  if (isLowGas) {
    return (
      <div
        data-testid="low-gas-warning"
        role="alert"
        aria-live="polite"
        title="Low XLM gas balance (< 5 XLM). Transactions may fail without gas."
        className={`flex items-center gap-2 px-3 py-1.5 rounded-xl border border-amber-500/40 bg-amber-500/10 text-amber-300 text-xs font-medium shadow-[0_0_12px_rgba(245,158,11,0.15)] animate-fade-in ${className}`}
      >
        <AlertTriangle
          size={14}
          className="text-amber-400 shrink-0 animate-pulse"
          aria-hidden="true"
        />
        <div className="flex items-center gap-1.5 min-w-0">
          <span className="font-semibold text-amber-200 truncate">{formattedBalance}</span>
          <span className="inline-block text-amber-400/90 text-[10px] uppercase font-bold tracking-wider px-1.5 py-0.2 rounded bg-amber-500/20 border border-amber-500/30 shrink-0">
            Low Gas
          </span>
        </div>

        {/* Link to Testnet Faucet */}
        <a
          href={STELLAR_FAUCET_URL}
          target="_blank"
          rel="noopener noreferrer"
          data-testid="faucet-link"
          className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-amber-500/25 hover:bg-amber-500/40 border border-amber-400/40 text-amber-100 hover:text-white transition-all text-[11px] font-semibold active:scale-95 shrink-0"
          aria-label="Fund testnet wallet with XLM from faucet"
        >
          <span>Faucet</span>
          <ExternalLink size={10} className="shrink-0" aria-hidden="true" />
        </a>

        {/* Manual Refresh Trigger */}
        <button
          onClick={() => refresh()}
          disabled={isRefreshing}
          data-testid="refresh-balance-btn"
          className="text-amber-400/70 hover:text-amber-200 p-1 rounded transition-colors disabled:opacity-50 shrink-0 cursor-pointer"
          title="Refresh balance"
          aria-label="Refresh balance"
        >
          <RefreshCw
            size={12}
            className={`transition-transform ${isRefreshing ? "animate-spin" : ""}`}
            aria-hidden="true"
          />
        </button>
      </div>
    );
  }

  // Normal Balance State (>= 5 XLM)
  return (
    <div
      data-testid="account-balance-badge"
      title="Stellar Account XLM Balance"
      aria-label={`Account balance: ${formattedBalance}`}
      className={`flex items-center gap-2 px-3 py-1.5 rounded-xl border border-[#1e293b] bg-[#0F1621] text-slate-300 text-xs font-medium hover:border-[#33C5E0]/40 transition-colors shadow-sm ${className}`}
    >
      <span
        className="w-1.5 h-1.5 rounded-full bg-[#33C5E0] shadow-[0_0_6px_#33C5E0] shrink-0"
        aria-hidden="true"
      />
      <span className="font-semibold text-white">{formattedBalance}</span>

      {/* Manual Refresh Trigger */}
      <button
        onClick={() => refresh()}
        disabled={isRefreshing}
        data-testid="refresh-balance-btn"
        className="text-slate-400 hover:text-cyan-400 p-1 rounded transition-colors disabled:opacity-50 ml-0.5 cursor-pointer"
        title="Refresh balance"
        aria-label="Refresh balance"
      >
        <RefreshCw
          size={12}
          className={`transition-transform ${isRefreshing ? "animate-spin" : ""}`}
          aria-hidden="true"
        />
      </button>
    </div>
  );
}

const Navbar = () => {
  const { isConnected, address } = useWallet();
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
          className="md:hidden text-slate-300 hover:text-cyan-400 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm p-2 relative z-10 cursor-pointer"
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

            {/* Mobile Account Balance & Gas Warning */}
            {isConnected && address && (
              <div className="pt-2 pb-1 border-t border-white/5 flex flex-col gap-2">
                <span className="text-[11px] uppercase tracking-wider text-slate-500 font-semibold px-2">
                  Account Gas Balance
                </span>
                <AccountBalanceBadge className="w-full justify-between" isMobile={true} />
              </div>
            )}

            <button
              className="flex justify-center items-center gap-4 text-[14px] border-[0.5px] border-[#33C5E03D] bg-[#161E22] px-4 py-3 rounded-lg text-slate-300 hover:border-cyan-400 transition-all w-full focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 active:scale-95 cursor-pointer"
              aria-label="Connect wallet"
            >
              Connect Wallet <ArrowDownRight size={16} aria-hidden={true} />
            </button>
          </div>
        )}

        {/* Desktop Header Actions: Account Balance + ConnectButton */}
        <div className="hidden md:flex items-center gap-3">
          {isConnected && address && <AccountBalanceBadge />}
          <ConnectButton />
        </div>
      </nav>
    </header>
  );
};

export default Navbar;
