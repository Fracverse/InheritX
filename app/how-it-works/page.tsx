"use client";

import React, { useState } from "react";
import Image from "next/image";
import Navbar from "../components/Navbar";
import Footer from "../components/Footer";
import { ArrowUpRight, Headphones } from "lucide-react";

type Step = {
  title: string;
  description: string | React.ReactNode;
};

type TabContent = {
  id: string;
  label: string;
  steps: Step[];
};

const content: TabContent[] = [
  {
    id: "plan-types",
    label: "Inheritance Plan Types",
    steps: [
      {
        title: "Step 1: Go to Inheritance Plans",
        description: "Navigate to the \"Inheritance Plans\" section on your dashboard.",
      },
      {
        title: "Step 2: Select Plan Type",
        description: (
          <div>
            <p>Choose the type of plan you want to create:</p>
            <ul className="list-disc ml-5 mt-2 space-y-1">
              <li>Single Beneficiary Plan — one person receives the assets.</li>
              <li>Multiple Beneficiary Plan — split assets across several people with chosen percentages.</li>
            </ul>
          </div>
        ),
      },
      {
        title: "Step 3: Add Beneficiaries",
        description: "Enter names, contact details, and wallet addresses for each beneficiary.",
      },
      {
        title: "Step 4: Define Rules & Conditions",
        description: "Set rules such as conditions for release (e.g., claim code or disbursment settings).",
      },
      {
        title: "Step 5: Save & Activate",
        description: "Review all details and confirm. Your inheritance plan is now active and will work based on the conditions you've set.",
      },
    ],
  },
  {
    id: "claims",
    label: "Claims",
    steps: [
      {
        title: "Step 1: Initiate a Claim",
        description: "Go to the Claims section and click \"New Claim.\" Enter your details and select the inheritance plan you're claiming from.",
      },
      {
        title: "Step 2: Provide Required Documents",
        description: "Upload identity verification and any documents requested by the plan (e.g., death certificate, legal proof).",
      },
      {
        title: "Step 3: Submit for Review",
        description: "Double-check your entries and submit. The system logs your request and notifies the plan owner or guardians.",
      },
      {
        title: "Step 4: Verification & Approval",
        description: "The platform verifies your information. You'll receive updates on your claim status (Pending -> Under Review -> Approved/Rejected).",
      },
      {
        title: "Step 5: Receive Assets",
        description: "Once approved, assets are automatically transferred to the wallet or account specified in the inheritance plan.",
      },
    ],
  },
  {
    id: "kyc",
    label: "KYC",
    steps: [
      {
        title: "Step 1: Open KYC Section",
        description: "Go to the \"KYC Verification\" (Know Your Customer) pop up from your dashboard.",
      },
      {
        title: "Step 2: Fill Out Personal Information",
        description: "Enter your full name, date of birth, contact details, and any other requested personal info.",
      },
      {
        title: "Step 3: Upload Required Documents",
        description: "Provide the necessary identification documents (e.g., ID card, passport, or driver's license).",
      },
      {
        title: "Step 4: Submit for Verification",
        description: "Review all entered information, confirm it's correct, and click \"Submit.\"",
      },
      {
        title: "Step 5: Wait for Approval",
        description: "Your KYC request will be reviewed. You'll be notified when it's approved, unlocking features that require verification.",
      },
    ],
  },
];

export default function HowToUsePage() {
  const [activeTab, setActiveTab] = useState(content[0].id);

  const activeContent = content.find((tab) => tab.id === activeTab) || content[0];

  return (
    <div className="relative min-h-screen bg-[#161E22] text-slate-300 selection:text-black overflow-x-hidden">
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
    <div className="flex justify-between items-center">
      <main className="max-w-4xl mx-16  py-20 relative z-10">
        <div className="mb-12 animate-slide-up">
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">
            How To Use <span className="text-cyan-400">InheritX</span>
          </h1>
          <p className="text-lg text-[#92A5A8] max-w-2xl">
            Follow these steps to get the most out of your digital inheritance platform.
          </p>
        </div>

        {/* Tab Section with Bottom Border */}
        <div className="relative border-b border-[#2A3338] mb-12 flex items-center justify-between pb-0.5">
          <div className="flex gap-2">
            {content.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`relative px-6 py-3 text-sm font-semibold transition-all duration-300 ${
                  activeTab === tab.id
                    ? "bg-[#1C252A] text-cyan-400 rounded-t-xl border-x border-t border-[#2A3338] -mb-[1px] shadow-sm"
                    : "text-[#92A5A8] hover:text-white"
                }`}
              >
                {tab.label}
                {/* Active Tab Overlap for the Border */}
                {activeTab === tab.id && (
                  <div className="absolute bottom-[-1px] left-0 right-0 h-[2px] bg-[#1C252A]" />
                )}
              </button>
            ))}
          </div>


        </div>

        {/* Steps Content */}
        <div className="space-y-6 min-h-[500px]">
          {activeContent.steps.map((step, index) => (
            <div
              key={`${activeTab}-${index}`}
              className="bg-[#1C252A] p-8 border border-[#2A3338] rounded-2xl transition-all duration-300 hover:border-cyan-400/10 hover:shadow-[inset_0_2px_30px_rgba(51,197,224,0.02)] animate-slide-up"
              style={{ animationDelay: `${index * 0.05}s` }}
            >
              <h3 className="text-xl font-bold text-white mb-3">
                {step.title}
              </h3>
              <div className="text-[#92A5A8] leading-relaxed">
                {step.description}
              </div>
            </div>
          ))}
        </div>

        {/* Pill-shaped Launch App Button */}
        <div className="mt-16 text-center animate-fade-in" style={{ animationDelay: "0.4s" }}>
          <button
            className="group flex items-center gap-12 px-24 py-5 rounded-b-[40px] bg-[#33C5E0] text-black font-bold transition-all hover:bg-cyan-300 active:scale-95 shadow-[0_0_30px_rgba(51,197,224,0.4)]"
          >
            <span className="text-xl tracking-tight">LAUNCH APP</span>
            <ArrowUpRight size={28} className="transition-transform group-hover:translate-x-1 group-hover:-translate-y-1" strokeWidth={2.5} />
          </button>
        </div>
      </main>
      <div className="flex justify-end mr-16 h-16">
          <button className="flex items-center gap-2 px-5 py-2.5 rounded-full bg-[#1C252A] text-[#92A5A8] text-sm font-medium hover:text-cyan-400 transition-colors border border-transparent hover:border-[#2A3338]">
            <Headphones size={18} />
            Contact Support
          </button>
      </div>
</div>

      <Footer />
    </div>
  );
}