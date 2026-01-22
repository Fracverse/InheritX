"use client";

import Link from "next/link";
import { ConnectButton } from "./ConnectButton";
import logo from "../public/logo.png";
import Image from "next/image";
export function Navbar() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-40 bg-black/50 backdrop-blur-md border-b border-white/10">
      <div className="max-w-7xl mx-auto px-6 h-20 flex items-center justify-between">
        <div className="hidden md:flex items-center gap-8 text-xs font-bold tracking-widest text-[#92A5A8]">
          <Link href="/" className="flex items-center gap-2 group">
            <Image src={logo} alt="Logo" className="w-10 h-10" />
          </Link>

          {/* Desktop Nav */}

          <Link
            href="#"
            className="hover:text-[#33C5E0] transition-colors uppercase border-b-2 border-[#33C5E0] pb-1"
          >
            Home
          </Link>
          <Link
            href="#"
            className="hover:text-[#33C5E0] transition-colors uppercase pb-1 border-b-2 border-transparent hover:border-[#33C5E0]"
          >
            How it works
          </Link>
          <Link
            href="#"
            className="hover:text-[#33C5E0] transition-colors uppercase pb-1 border-b-2 border-transparent hover:border-[#33C5E0]"
          >
            FAQs
          </Link>
          <Link
            href="#"
            className="hover:text-[#33C5E0] transition-colors uppercase pb-1 border-b-2 border-transparent hover:border-[#33C5E0]"
          >
            Contact
          </Link>
        </div>

        {/* Action Area */}
        <div className="flex items-center gap-4">
          <ConnectButton />
        </div>
      </div>
    </nav>
  );
}
