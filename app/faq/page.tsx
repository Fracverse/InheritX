"use client";

import React from "react";
import Image from "next/image";
import Navbar from "../components/Navbar";
import Footer from "../components/Footer";

const faqs = [
  {
    question: "What is InheritX?",
    answer: "InheritX is a platform that simplifies wealth inheritance and asset planning, ensuring your digital and physical assets are transferred securely to your loved ones according to your rules.",
  },
  {
    question: "How secure is my data?",
    answer: "We use top-level encryption and secure protocols to ensure that your asset information and beneficiary details are protected and only accessible to authorized parties at the right time.",
  },
  {
    question: "Can I change my beneficiaries?",
    answer: "Yes, you can update your beneficiaries and the rules for asset distribution at any time through your dashboard.",
  },
  {
    question: "What happens if I forget my password?",
    answer: "We have secure recovery processes in place to help you regain access to your account while maintaining the highest security standards for your inheritance plan.",
  },
  {
    question: "Are there any legal fees?",
    answer: "InheritX aims to reduce the need for expensive legal jargon and delays. While we are not a law firm, we provide the tools to make your intent clear and the transfer process smooth.",
  },
];

export default function FAQPage() {
  return (
    <div className="relative min-h-screen bg-[#161E22] text-slate-300 selection:text-black overflow-x-hidden pt-12">
      {/* Decorative tree-like background glow at top right */}
      <div className="absolute top-0 right-0 z-0 pointer-events-none w-1/2 md:w-1/3">
        <Image
          src="/tree.svg"
          alt=""
          role="presentation"
          width={1200}
          height={1000}
          className="opacity-40 animate-fade-in"
          priority
        />
      </div>

      <Navbar />

      <main className="max-w-4xl mx-auto px-6 py-20 relative z-10">
        <div className="mb-16 animate-slide-up">
          <h1 className="text-4xl md:text-6xl font-bold text-white mb-6">
            Frequently Asked <span className="text-cyan-400">Questions</span>
          </h1>
          <p className="text-lg text-[#92A5A8] max-w-2xl">
            Everything you need to know about InheritX and how we help you
            secure your legacy for the next generation.
          </p>
        </div>

        <div className="space-y-6">
          {faqs.map((faq, index) => (
            <div
              key={index}
              className="bg-[#1C252A] p-8 border border-[#2A3338] rounded-2xl transition-all duration-300 hover:border-cyan-400/30 hover:shadow-[inset_0_2px_30px_rgba(51,197,224,0.05)] animate-slide-up"
              style={{ animationDelay: `${index * 0.1}s` }}
            >
              <h3 className="text-xl font-bold text-white mb-3">
                {faq.question}
              </h3>
              <p className="text-[#92A5A8] leading-relaxed">
                {faq.answer}
              </p>
            </div>
          ))}
        </div>

        <div className="mt-20 text-center animate-fade-in" style={{ animationDelay: "0.6s" }}>
          <p className="text-[#92A5A8] mb-6">Still have questions?</p>
          <a
            href="/#footer"
            className="inline-flex items-center gap-2 px-8 py-3 rounded-lg bg-[#33C5E0] text-black font-semibold transition-all hover:bg-cyan-300 active:scale-95"
          >
            Contact Support
          </a>
        </div>
      </main>

      <Footer />
    </div>
  );
}
