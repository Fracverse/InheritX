import Image from "next/image";
import Link from "next/link";
import { ArrowDownRight, Globe, Shield, Zap, Menu, X } from "lucide-react";

const Footer = () => {
  return (
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
              <Link
                href="/"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Home
              </Link>
              <Link
                href="/#about"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                About
              </Link>
              <Link
                href="#"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                FAQs
              </Link>
              <Link
                href="/Guidelines"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Guidelines
              </Link>
              <Link
                href="#"
                className="hover:text-cyan-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
              >
                Support
              </Link>
            </nav>
          </div>

          {/* Logo Center */}
          <div className="text-cyan-400 font-bold text-4xl italic tracking-tighter">
            <Link
              href="/"
              className="focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm"
            >
              <Image
                src="/logo.svg"
                alt="InheritX"
                width={93}
                height={93}
                quality={85}
              />
            </Link>
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
            <Link
              href="#"
              className="hover:text-slate-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
            >
              Privacy Policy
            </Link>
            <Link
              href="#"
              className="hover:text-slate-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
            >
              Terms & Conditions
            </Link>
            <Link
              href="#"
              className="hover:text-slate-400 transition-colors focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 rounded-sm px-1"
            >
              Code of Ethics
            </Link>
          </nav>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
