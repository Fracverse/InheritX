import Image from "next/image";

export default function AboutSection() {
  return (
    <section
      id="about"
      className="mt-24 md:mt-100 py-16 md:py-24 px-8 relative z-10"
      role="region"
      aria-label="About InheritX"
    >
      <div className="max-w-3xl mx-auto">
        <h2 className="text-[#FCFFFF] uppercase tracking-[0.3em] text-[32px] mb-4 animate-slide-up">
          What is InheritX?
        </h2>
        <h3
          className="text-[#92A5A8] text-[14px] font-bold mb-6 animate-slide-up"
          style={{ animationDelay: "0.1s" }}
        >
          Without roots, nothing grows.
        </h3>
        <p
          className="text-[18px] text-[#FCFFFF] leading-relaxed mb-8 animate-slide-up"
          style={{ animationDelay: "0.2s" }}
        >
          Inherit X helps you plan and share your assets with the right
          people, at the right time. We make inheritance simple, secure, and
          stress-free — without unnecessary delays or complications. Think of
          it as planting a tree: your roots are the assets you&apos;ve built,
          and we make sure the branches grow to those you care about most.
        </p>
        <div
          className="text-[18px] text-[#FCFFFF] animate-slide-up"
          style={{ animationDelay: "0.3s" }}
        >
          Think of it as planting a tree: your roots are the assets
          you&apos;ve built, and we make sure the branches grow to those you
          care about most.
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
