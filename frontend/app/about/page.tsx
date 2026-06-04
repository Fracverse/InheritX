"use client";

import { motion } from "framer-motion";
import { ArrowUpRight } from "lucide-react";
import Image from "next/image";
import Footer from "../components/Footer";
import Navbar from "../components/Navbar";

export default function AboutPage() {
  return (
    <div className="min-h-screen bg-[#161E22] text-slate-300 relative overflow-hidden">
      {/* Decorative background */}
      <div className="w-full absolute top-0 left-0 z-0 overflow-hidden">
        <Image
          src="/tree.svg"
          alt=""
          role="presentation"
          width={2400}
          height={1000}
          className="opacity-50 pointer-events-none w-full h-auto"
          priority
          quality={75}
        />
      </div>

      <Navbar />

      {/* Hero Section */}
      <section className="max-w-4xl mx-auto px-6 pt-16 pb-12 relative z-10">
        <motion.h1
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          className="text-4xl md:text-5xl font-bold leading-tight text-[#FCFFFF]"
        >
          About InheritX
        </motion.h1>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.1 }}
          className="mt-6 text-base md:text-lg text-[#FCFFFF] leading-relaxed"
        >
          InheritX is a blockchain-powered platform dedicated to redefining how digital assets are secured and passed down. We combine security, transparency, and automation to ensure your crypto and NFTs are protected for generations.
        </motion.p>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2 }}
          className="mt-4 text-base text-[#92A5A8] leading-relaxed"
        >
          Founded by a team of blockchain experts at Skill Mind, InheritX was created to bridge the gap between traditional estate planning and the digital economy. We provide a secure, decentralized solution that ensures digital assets are protected, preserved, and passed on with efficiency and transparency.
        </motion.p>
      </section>

      {/* Contact Support Button */}
      <div className="fixed bottom-8 right-8 z-50">
        <motion.button
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.3, delay: 0.5 }}
          className="flex items-center gap-2 px-6 py-3 bg-[#161E22] border border-[#33C5E03D] rounded-full text-[#92A5A8] hover:text-cyan-400 hover:border-cyan-400 transition-all duration-300 shadow-lg"
        >
          <span className="text-sm">Contact Support</span>
        </motion.button>
      </div>

      {/* Vision & Mission Section */}
      <section className="max-w-4xl mx-auto px-6 py-12 relative z-10">
        <div className="grid md:grid-cols-2 gap-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="border-[13px] border-[#1C252A] bg-[#1C252A] rounded-3xl p-8 transition-all duration-300 hover:border-cyan-400/30"
          >
            <h2 className="text-2xl md:text-3xl font-semibold mb-3 text-[#FCFFFF]">Our Vision</h2>
            <p className="text-base text-[#92A5A8] mb-4 font-medium">
              To Make Digital Inheritance Seamless, Secure, & Automated.
            </p>
            <p className="text-sm text-[#92A5A8] leading-relaxed">
              We envision a world where passing down cryptocurrencies and NFTs is automated, secure, and effortless. By leveraging cutting-edge blockchain technology, we aim to be the trusted partner for individuals, families, and businesses in digital estate planning.
            </p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.1 }}
            className="border-[13px] border-[#1C252A] bg-[#1C252A] rounded-3xl p-8 transition-all duration-300 hover:border-cyan-400/30"
          >
            <h2 className="text-2xl md:text-3xl font-semibold mb-3 text-[#FCFFFF]">Our Mission</h2>
            <p className="text-base text-[#92A5A8] mb-4 font-medium">
              To Secure Digital Wealth for Generations
            </p>
            <p className="text-sm text-[#92A5A8] leading-relaxed">
              We believe digital assets deserve the same trust, security, and transparency as traditional assets. InheritX is dedicated to building a decentralized inheritance platform that empowers users to protect and transfer their digital wealth with confidence.
            </p>
          </motion.div>
        </div>

        {/* Separator Line */}
        <div className="mt-4 h-[10px] bg-[#1C252A] rounded-full"></div>
      </section>

      {/* Core Values Section */}
      <section className="py-10 relative z-10">
        <div className="max-w-4xl mx-auto px-6">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="bg-[#161E22] rounded-3xl p-8 md:p-12 shadow-[inset_0_2px_20px_rgba(0,0,0,0.15)]"
          >
            <h2 className="text-2xl md:text-3xl font-semibold mb-3 text-[#33C5E0]">
              Our Core Values
            </h2>
            <p className="text-sm text-[#92A5A8] mb-10 leading-relaxed">
              At InheritX, we uphold security, transparency, innovation, user-centricity, and compliance as core values. We prioritize robust security measures, ensure clear and verifiable processes, embrace cutting-edge blockchain advancements, and design intuitive experiences—all while adhering to the highest legal standards.
            </p>

            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: 0.1 }}
              className="flex justify-start"
            >
              <button className="flex items-center gap-2 px-8 py-3 rounded-t-lg rounded-b-[18px] bg-[#33C5E0] text-black font-semibold cursor-pointer transition-all duration-300 hover:bg-cyan-300 active:scale-95 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400">
                LAUNCH APP
                <ArrowUpRight size={16} aria-hidden={true} />
              </button>
            </motion.div>
          </motion.div>
        </div>
      </section>
      <Footer />
    </div>
  );
}
