"use client";

import Image from "next/image";
import { useTranslations } from "next-intl";

export default function AboutSection() {
  const t = useTranslations("landing");

  return (
    <section
      id="about"
      className="mt-24 md:mt-100 py-16 md:py-24 px-8 relative z-10"
      role="region"
      aria-label="About InheritX"
    >
      <div className="max-w-3xl mx-auto">
        <h2 className="text-[#FCFFFF] uppercase tracking-[0.3em] text-[32px] mb-4 animate-slide-up">
          {t("aboutTitle")}
        </h2>
        <h3
          className="text-[#92A5A8] text-[14px] font-bold mb-6 animate-slide-up"
          style={{ animationDelay: "0.1s" }}
        >
          {t("aboutTagline")}
        </h3>
        <p
          className="text-[18px] text-[#FCFFFF] leading-relaxed mb-8 animate-slide-up"
          style={{ animationDelay: "0.2s" }}
        >
          {t("aboutBody1")}
        </p>
        <div
          className="text-[18px] text-[#FCFFFF] animate-slide-up"
          style={{ animationDelay: "0.3s" }}
        >
          {t("aboutBody2")}
        </div>
      </div>
      {/* Decorative tree-like background glow */}
      <div className="w-full absolute top-0 left-0 pointer-events-none">
        <Image
          src="/Vector (1).svg"
          alt=""
          role="presentation"
          width={500}
          height={100}
          className="opacity-50 pointer-events-none"
          quality={75}
        />
      </div>
    </section>
  );
}
