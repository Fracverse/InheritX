"use client";

import { TreePine, Sprout, Leaf, ArrowUpRight } from "lucide-react";
import Image from "next/image";
import { useTranslations } from "next-intl";

export default function HowItWorksSection() {
  const t = useTranslations("landing");

  return (
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
                {t("howItWorksTitle")}
              </span>
              <span className="text-[#FCFFFF] text-[14px] mt-1">
                {t("howItWorksSubtitle")}
              </span>
            </div>
          </div>

          {/* Step 1 - Bottom Position */}
          <div className="text-center relative group z-10 animate-scale-in">
            <div
              className="w-4 h-4 bg-slate-700 rounded-full mx-auto md:absolute md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 group-hover:bg-cyan-400 transition-all duration-300 z-20 mb-4 md:mb-0 will-change-colors"
              role="presentation"
              aria-hidden={true}
            ></div>
            <div className="flex flex-col items-center">
              <div className="mb-4 md:mb-50 border-[3px] border-[#1C252A] bg-[#161E22] w-[80px] h-[80px] rounded-full flex items-center justify-center mx-auto shadow-lg transition-all duration-300 group-hover:border-cyan-400/50 group-hover:shadow-[0_0_20px_rgba(51,197,224,0.3)]">
                <Sprout
                  className="text-cyan-400 transition-transform duration-300 group-hover:scale-110"
                  size={24}
                  aria-hidden={true}
                />
              </div>
              <h4 className="text-[#FCFFFF] font-semibold text-[18px] mb-2">
                {t("step1Title")}
              </h4>
              <p className="text-[14px] text-[#92A5A8] max-w-[200px] mx-auto">
                {t("step1Desc")}
              </p>
            </div>
          </div>

          {/* Step 2 - Top Position */}
          <div className="text-center relative group z-10 animate-scale-in">
            <div
              className="w-4 h-4 bg-slate-700 rounded-full mx-auto md:absolute md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 group-hover:bg-cyan-400 transition-all duration-300 z-20 mb-4 md:mb-0 will-change-colors"
              role="presentation"
              aria-hidden={true}
            ></div>
            <div className="flex flex-col md:flex-col-reverse items-center">
              <div className="mb-4 md:mt-50 border-[3px] border-[#1C252A] bg-[#161E22] w-[80px] h-[80px] rounded-full flex items-center justify-center mx-auto shadow-lg transition-all duration-300 group-hover:border-cyan-400/50 group-hover:shadow-[0_0_20px_rgba(51,197,224,0.3)]">
                <Leaf
                  className="text-cyan-400 transition-transform duration-300 group-hover:scale-110"
                  size={24}
                  aria-hidden={true}
                />
              </div>
              <p className="text-[14px] text-[#92A5A8] max-w-[200px] mx-auto">
                {t("step2Desc")}
              </p>
              <h4 className="text-[#FCFFFF] font-semibold text-[18px] mb-2">
                {t("step2Title")}
              </h4>
            </div>
          </div>

          {/* Step 3 - Bottom Position */}
          <div className="text-center relative group z-10 animate-scale-in">
            <div
              className="w-4 h-4 bg-slate-700 rounded-full mx-auto md:absolute md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 group-hover:bg-cyan-400 transition-all duration-300 z-20 mb-4 md:mb-0 will-change-colors"
              role="presentation"
              aria-hidden={true}
            ></div>
            <div className="flex flex-col items-center">
              <div className="mb-4 md:mb-50 border-[3px] border-[#1C252A] bg-[#161E22] w-[80px] h-[80px] rounded-full flex items-center justify-center mx-auto shadow-lg transition-all duration-300 group-hover:border-cyan-400/50 group-hover:shadow-[0_0_20px_rgba(51,197,224,0.3)]">
                <TreePine
                  className="text-cyan-400 transition-transform duration-300 group-hover:scale-110"
                  size={24}
                  aria-hidden={true}
                />
              </div>
              <h4 className="text-[#FCFFFF] font-semibold text-[18px] mb-2">
                {t("step3Title")}
              </h4>
              <p className="text-[14px] text-[#92A5A8] max-w-[200px] mx-auto">
                {t("step3Desc")}
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
              {t("whyWorksTitle")}
            </h3>
            <ul className="space-y-2 text-[14px] text-[#92A5A8]">
              <li className="flex gap-2 transition-all duration-300 hover:text-cyan-400">
                <span>•</span>
                <span>{t("whyWorks1")}</span>
              </li>
              <li className="flex gap-2 transition-all duration-300 hover:text-cyan-400">
                <span>•</span>
                <span>{t("whyWorks2")}</span>
              </li>
              <li className="flex gap-2 transition-all duration-300 hover:text-cyan-400">
                <span>•</span>
                <span>{t("whyWorks3")}</span>
              </li>
            </ul>
          </div>

          <button
            className="flex flex-row justify-center items-center gap-4 bg-cyan-400 text-black px-8 py-2 rounded-t-[8px] rounded-b-[16px] cursor-pointer transition-all duration-300 hover:bg-cyan-300 active:scale-95 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 font-semibold"
            aria-label="Get started with InheritX"
          >
            {t("getStarted")}
            <ArrowUpRight size={16} aria-hidden={true} />
          </button>
        </div>
      </div>
    </section>
  );
}
