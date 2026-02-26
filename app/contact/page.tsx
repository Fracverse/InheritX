"use client";

import { useState } from "react";
import {
  ArrowUpRight,
  Headphones,
  ChevronDown,
} from "lucide-react";
import Navbar from "../components/Navbar";
import Footer from "../components/Footer";

export default function ContactPage() {
  const [fullName, setFullName] = useState("");
   const[email, setEmail] = useState("");
  const [subject, setSubject] = useState("");
  const [message, setMessage] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Handle form submission
    console.log({ fullName, subject, message });
  };

  return (
    <div className="relative min-h-screen bg-[#161E22] text-slate-300 selection:text-black overflow-x-hidden">
      {/* Background image with reduced opacity */}
      <div 
        className="absolute inset-0 z-0 pointer-events-none"
        style={{
          backgroundImage: 'url(/tree.svg)',
          backgroundPosition: 'top left',
          backgroundRepeat: 'no-repeat',
          backgroundSize: 'contain',
          opacity: 0.5,
        }}
      />
      <Navbar />
      <div className="w-full max-w-3xl mx-auto px-6 md:px-8 py-20 md:py-32 relative z-10">
      <div className="bg-[#182024] rounded-2xl p-6 md:p-8">
      <div className="mb-12 text-left">
        <h1 className="text-white text-2xl md:text-3xl font-light mb-4">
          Contact Support
        </h1>
        <p className="text-white/50 text-sm font-light">
          We&apos;re here if you need help or clarity on things
          concerning InheritX
        </p>
      </div>

      <form onSubmit={handleSubmit} className="space-y-6">
        {/* Full Name Field */}
        <div>
          <label
            htmlFor="fullName"
            className="block text-white/ text-sm font-light mb-3"
          >
            Full Name
          </label>
          <input
            type="text"
            id="fullName"
            value={fullName}
            onChange={(e) => setFullName(e.target.value)}
            placeholder="John Doe"
            className="w-full px-4 py-3.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder:text-white/30 focus:outline-none focus:border-cyan-400/50 focus:bg-white/[0.07] transition-all text-sm font-light"
          />
        </div>

          {/* Email Field */}
        <div>
          <label
            htmlFor="fullName"
            className="block text-white/ text-sm font-light mb-3"
          >
            Email
          </label>
          <input
            type="email"
            id="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="Enter Email"
            className="w-full px-4 py-3.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder:text-white/30 focus:outline-none focus:border-cyan-400/50 focus:bg-white/[0.07] transition-all text-sm font-light"
          />
        </div>

        {/* Subject Field */}
        <div>
          <label
            htmlFor="subject"
            className="block text-white/70 text-sm font-light mb-3"
          >
            Subject
          </label>
          <div className="relative">
            <select
              id="subject"
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
              className="w-full px-4 py-3.5 bg-white/5 border border-white/10 rounded-lg text-white/30 focus:outline-none focus:border-cyan-400/50 focus:bg-white/[0.07] transition-all appearance-none text-sm font-light cursor-pointer"
            >
              <option value="">Select A Subject</option>
              <option value="technical">
                Technical Support
              </option>
              <option value="account">Account Issues</option>
              <option value="inheritance">Inheritance Questions</option>
              <option value="general">General Inquiry</option>
            </select>
            <ChevronDown className="absolute right-4 top-1/2 -translate-y-1/2 w-5 h-5 text-white/30 pointer-events-none" />
          </div>
        </div>

        {/* Message Field */}
        <div>
          <label
            htmlFor="message"
            className="block text-white/70 text-sm font-light mb-3"
          >
            Message
          </label>
          <textarea
            id="message"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder="Text"
            rows={8}
            className="w-full px-4 py-3.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder:text-white/30 focus:outline-none focus:border-cyan-400/50 focus:bg-white/[0.07] transition-all resize-none text-sm font-light"
          />
        </div>

        {/* Submit Button */}
        <div className="pt-4 flex justify-center">
          <button
            type="submit"
            disabled
            className="flex items-center justify-center gap-2 px-8 py-3 rounded-t-lg rounded-b-[18px] bg-[#425558] text-white font-semibold cursor-not-allowed transition-all duration-300 focus-visible:outline-offset-2 focus-visible:outline-2 focus-visible:outline-cyan-400 opacity-60"
          >
            SEND MESSAGE
            <ArrowUpRight size={16} aria-hidden={true} />
          </button>
        </div>
      </form>
      </div>
      </div>
      
      {/* Contact Support Link - Outside form, close to container */}
      <div className="absolute right-4  md:right-[calc(50%-24rem-10rem)] top-[600px] md:top-[650px] z-10">
        <div className="flex items-center gap-2 text-white/60 text-sm font-light">
          <Headphones className="w-4 h-4" />
          <span>Contact Support</span>
        </div>
      </div>
      
      <Footer />
    </div>
  );
}