"use client";

import React, { useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { ArrowUpRight, MessageCircle, Menu, X } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { ConnectButton } from "@/components/ConnectButton";
import Footer from "../components/Footer";

const faqs = [
  {
    id: 1,
    question: "What is Inherit X?",
    answer:
      "Inherit X Is A Secure Online Platform That Helps You Plan And Share Your Assets With Your Chosen Beneficiaries — Simply And Stress-Free.",
  },
  {
    id: 2,
    question: "How is this different from traditional inheritance planning?",
    answer:
      "We Remove The Paperwork Headaches, Legal Jargon, And Long Delays By Making The Process Digital, Clear, And Accessible To Anyone.",
  },
  {
    id: 3,
    question: "IS MY INFORMATION SAFE?",
    answer:
      "Your Data And Assets Are Protected With Top-Level Encryption and decentralized storage. We ensure that only authorized beneficiaries can access the information at the specified time.",
  },
  {
    id: 4,
    question: "CAN I CHANGE MY PLAN LATER?",
    answer:
      "Yes, you have full flexibility to modify your inheritance plan, including adding or removing beneficiaries and updating asset distribution rules, as long as you have control of your account.",
  },
  {
    id: 5,
    question: "DO I NEED A LAWYER TO USE INHERIT X?",
    answer:
      "No, Inherit X is designed to be user-friendly and handles the technical complexities of asset sharing. However, we always recommend consulting with a legal professional for complex estate planning needs.",
  },
  {
    id: 6,
    question: "HOW DO BENEFICIARIES GET NOTIFIED?",
    answer:
      "Beneficiaries are notified through secure channels based on the triggers you set in your plan. This ensures they receive the necessary information and access exactly when intended.",
  },
  {
    id: 7,
    question: "WHAT TYPES OF INHERITANCE PLANS CAN I CREATE?",
    answer:
      "You can create various plans ranging from simple asset transfers to complex time-locked or conditional distributions, catering to different family and financial structures.",
  },
];

export default function FAQPage() {
  const [openId, setOpenId] = useState<number | null>(1);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  const toggleAccordion = (id: number) => {
    setOpenId(openId === id ? null : id);
  };

  return (
    <div className="relative min-h-screen bg-[#161E22] text-slate-300 selection:text-black overflow-x-hidden">
      {/* Decorative tree-like background glow */}

      <div className="w-full absolute top-0 left-0 z-0">
        <Image
          src="/faq2.png"
          alt=""
          role="presentation"
          width={2400}
          height={1000}
          className="z-[100] pointer-events-none"
          priority
          quality={75}
        />
      </div>

      <div className="w-full absolute top-0 left-0 z-0">
        <Image
          src="/faq.png"
          alt=""
          role="presentation"
          width={2400}
          height={1000}
          className="z-[200] pointer-events-none"
          priority
          quality={75}
        />
      </div>

      {/* Navigation */}
      <header
        className="sticky top-0 z-50 backdrop-blur-xs bg-[#161E22]/40"
        role="banner"
      >
        <nav
          className="flex justify-between items-center px-6 md:px-40 py-6 mx-auto"
          role="navigation"
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
              <Link href="/" className="hover:text-cyan-400 transition-colors">
                Home
              </Link>
              <Link
                href="/#how-it-works"
                className="hover:text-cyan-400 transition-colors"
              >
                How it Works
              </Link>
              <Link
                href="/faqs"
                className="text-cyan-400 border-b-2 border-cyan-400 pb-1"
              >
                FAQs
              </Link>
              <Link
                href="/#footer"
                className="hover:text-cyan-400 transition-colors"
              >
                Contact
              </Link>
            </div>
          </div>

          <button
            className="md:hidden text-slate-300 hover:text-cyan-400 p-2 relative z-10"
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
          >
            {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
          </button>

          {isMobileMenuOpen && (
            <div className="absolute top-full left-0 w-full bg-[#161E22] border-b border-[#2A3338] p-4 flex flex-col gap-4 md:hidden shadow-2xl z-10">
              <Link
                href="/"
                onClick={() => setIsMobileMenuOpen(false)}
                className="text-slate-300 hover:text-cyan-400 py-2 uppercase"
              >
                Home
              </Link>
              <Link
                href="/#how-it-works"
                onClick={() => setIsMobileMenuOpen(false)}
                className="text-slate-300 hover:text-cyan-400 py-2 uppercase"
              >
                How it Works
              </Link>
              <Link
                href="/faqs"
                onClick={() => setIsMobileMenuOpen(false)}
                className="text-cyan-400 py-2 uppercase"
              >
                FAQs
              </Link>
              <Link
                href="/#footer"
                onClick={() => setIsMobileMenuOpen(false)}
                className="text-slate-300 hover:text-cyan-400 py-2 uppercase"
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

      {/* FAQ Content */}
      <main className="relative z-10 px-6 md:px-40 mx-auto pt-20 pb-32 flex flex-col md:flex-row gap-12">
        <div className="flex-1">
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">
            FAQs
          </h1>
          <p className="text-[#92A5A8] text-lg mb-12">
            Here are some frequently asked questions about InheritX
          </p>

          <div className="space-y-4 max-w-3xl">
            {faqs.map((faq) => (
              <motion.div
                key={faq.id}
                layout
                initial={false}
                animate={{
                  boxShadow:
                    openId === faq.id
                      ? "inset 4px 4px 10px rgba(17, 23, 26, 0.9), inset -4px -4px 8px rgba(27, 37, 42, 0.9)"
                      : "inset 0px 0px 0px rgba(0, 0, 0, 0), inset 0px 0px 0px rgba(0, 0, 0, 0)",
                  borderColor:
                    openId === faq.id
                      ? "rgba(51, 197, 224, 0.3)"
                      : "rgba(28, 37, 42, 1)",
                }}
                transition={{ duration: 0.3, ease: "easeInOut" }}
                className="border-[3px] bg-[#161E22] rounded-2xl overflow-hidden cursor-pointer"
                onClick={() => toggleAccordion(faq.id)}
              >
                <div className="w-full flex items-center justify-between p-6 text-left group">
                  <span
                    className={`text-lg font-bold tracking-tight uppercase transition-colors duration-300 ${
                      openId === faq.id
                        ? "text-cyan-400"
                        : "text-white group-hover:text-cyan-400"
                    }`}
                  >
                    {faq.id}. {faq.question}
                  </span>
                  <motion.div
                    animate={{ rotate: openId === faq.id ? 90 : 0 }}
                    transition={{ duration: 0.3, ease: "easeInOut" }}
                  >
                    <ArrowUpRight
                      className={
                        openId === faq.id
                          ? "text-cyan-400"
                          : "text-slate-500 group-hover:text-cyan-400"
                      }
                      size={24}
                    />
                  </motion.div>
                </div>

                <AnimatePresence initial={false}>
                  {openId === faq.id && (
                    <motion.div
                      key="content"
                      initial="collapsed"
                      animate="open"
                      exit="collapsed"
                      variants={{
                        open: { height: "auto", opacity: 1 },
                        collapsed: { height: 0, opacity: 0 },
                      }}
                      transition={{ duration: 0.3, ease: "easeInOut" }}
                    >
                      <div className="px-6 pb-6 text-[#92A5A8] leading-relaxed text-[16px]">
                        {faq.answer}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </motion.div>
            ))}
          </div>

          <div className="mt-12">
            <button className="flex items-center gap-2 px-10 py-4 rounded-t-lg rounded-b-[20px] bg-[#33C5E0] text-black font-bold uppercase tracking-wider transition-all duration-300 hover:bg-cyan-300 active:scale-95">
              Launch App <ArrowUpRight size={18} />
            </button>
          </div>
        </div>

        {/* Sidebar / Support Link */}
        <div className="md:w-64 absolute bottom-100 right-5 hidden md:block">
          <button className="flex items-center gap-3 text-[#92A5A8] hover:text-cyan-400 transition-colors group">
            <div className="p-2 border border-[#1C252A] rounded-xl group-hover:border-cyan-400/50 transition-colors">
              <MessageCircle size={24} />
            </div>
            <span className="font-semibold uppercase tracking-widest text-xs">
              Contact Support
            </span>
          </button>
        </div>
      </main>

      <Footer />
    </div>
  );
}
