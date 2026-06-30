"use client";

import { ThumbsUp, ShieldCheck, Settings, Zap, ArrowUpRight } from "lucide-react";
import { useTranslations } from "next-intl";

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
    style={{ animationDelay: `${index * 0.1}s` }}
  >
    <div className="bg-transparent p-3 w-fit mb-4 transition-transform duration-300 group-hover:scale-110 group-hover:animate-float">
      <Icon className="text-cyan-400" size={32} aria-hidden={true} />
    </div>
    <h4 className="text-[#FCFFFF] text-[18px] font-bold mb-2">{title}</h4>
    <p className="text-[#92A5A8] text-[14px] leading-relaxed px-2">{desc}</p>
  </div>
);

export default function BenefitsSection() {
  const t = useTranslations("landing");

  return (
    <section
      id="benefits"
      className="py-12 md:py-24 px-4 md:px-8 bg-[#161E22] relative z-10"
      role="region"
      aria-label="Benefits of InheritX"
    >
      <div className="">
        <div className="md:w-[796px] w-full mx-auto bg-[#1a2329] mb-8 md:mb-16 rounded-3xl p-6 md:p-12 border border-[#1C252A] transition-all duration-300 hover:border-cyan-400/30 hover:shadow-[inset_0_2px_40px_rgba(51,197,224,0.05)] animate-slide-up">
          <h2 className="text-[32px] font-bold text-[#FCFFFF] mb-2">
            {t("benefitsTitle")}
          </h2>
          <p className="text-[#92A5A8] text-[14px] uppercase tracking-wider">
            {t("benefitsSubtitle")}
          </p>

          <div className="mt-6 max-w-2xl text-slate-400 leading-relaxed text-sm">
            <p className="mb-4 text-[#FCFFFF] text-[18px]">
              <span className="font-semibold">InheritX </span>
              {t("benefitsBody1")}
            </p>
            <p className="text-[#FCFFFF] text-[18px]">
              {t("benefitsBody2")}
            </p>
          </div>
        </div>

        <div className="max-w-7xl mx-auto relative rounded-3xl overflow-hidden">
          <div className="grid md:grid-cols-4">
            <FeatureCard
              icon={ThumbsUp}
              title={t("benefit1Title")}
              desc={t("benefit1Desc")}
              index={0}
            />
            <FeatureCard
              icon={ShieldCheck}
              title={t("benefit2Title")}
              desc={t("benefit2Desc")}
              index={1}
            />
            <FeatureCard
              icon={Settings}
              title={t("benefit3Title")}
              desc={t("benefit3Desc")}
              index={2}
            />
            <FeatureCard
              icon={Zap}
              title={t("benefit4Title")}
              desc={t("benefit4Desc")}
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
            {t("createYourPlan")}
            <ArrowUpRight size={16} aria-hidden={true} />
          </button>
        </div>
      </div>
    </section>
  );
}
