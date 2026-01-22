"use client";

import Image from "next/image";
import Link from "next/link";
import Footer from "../components/Footer";
import Navbar from "../components/Navbar";
import { Headset, ArrowUpRight } from "lucide-react";

export default function GuidelinesPage() {
  return (
    <div className="relative min-h-screen bg-[#161E22] text-slate-300 selection:text-black overflow-x-hidden">
      {/* Decorative tree-like background glow */}
      <div className="w-full absolute top-0 left-0 z-0">
        <Image
          src="/tree.svg"
          alt=""
          role="presentation"
          width={2400}
          height={1000}
          className="opacity-50 pointer-events-none"
          priority
          quality={75}
        />
      </div>

      <Navbar />

      {/* Main Content Section */}
      <section
        className="w-full h-full relative pb-20 md:pb-32 px-6 md:px-40 bg-transparent"
        role="region"
        aria-label="Guidelines content"
      >
        <div className="pt-5 md:pt-10">
          {/* Page Header */}
          <div className="mb-12">
            <h1 className="text-4xl  font-bold text-white mb-4">Guidelines</h1>
            <p className=" text-[#92A5A8]">
              Here are some important social dynamics about InheritX
            </p>
          </div>

          {/* Two Column Layout */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {/* Left Column - Main Content */}
            <div className="lg:col-span-2 space-y-12 ">
              {/* Privacy Policy */}
              <div className="space-y-6">
                <div className="flex justify-between">
                  <h2 className="text-2xl pl-2 md:pl-10 font-bold text-[#FCFFFF]">
                    Privacy Policy
                  </h2>
                  <span>
                    <Image
                      src="/arrowdown.png"
                      alt="arrow"
                      height={16}
                      width={16}
                    />
                  </span>
                </div>

                {/* Personal Information */}
                <div className="relative rounded-tr-[50px] rounded-br-[50px] p-4 md:p-10 bg-[#161E22] border-2 border-[#161E22] shadow-[0_0_30px_rgba(0,0,0,0.9)]">
                  {/* first */}

                  <div className="mb-10">
                    <h2 className="text-xl  mb-10  font-bold text-white">
                      Data Collection and Usage
                    </h2>
                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base  font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2 ">
                          PERSONAL INFORMATION
                        </h3>
                        <h2 className="flex items-start">
                          <span>We Collect</span>
                          <span className="mr-2">:</span>
                        </h2>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-center">
                            <span className="mr-2">›</span>
                            <span className="text-balance">
                              Legal Name and Contact Information
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Government-Issued Identification</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Biometric Data (if applicable)</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Authentication Credentials</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Communication Preferences</span>
                          </li>
                        </ul>
                      </div>

                      {/* Technical Data */}
                      <div className="flex-1">
                        <h3 className="text-base  font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2 ">
                          TECHNICAL DATA
                        </h3>
                        <h2 className="flex items-start">
                          <span>We Record</span>
                          <span className="mr-2">:</span>
                        </h2>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Device Information</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>IP Address</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Browser and OS Details</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Network Information</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Cookies and Tracking Data</span>
                          </li>
                        </ul>
                      </div>

                      {/* usage data */}
                    </div>
                    <div>
                      <h3 className="text-base  font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2 ">
                        Usage Data
                      </h3>
                      <ul className="space-y-2 text-slate-300">
                        <h2 className="flex items-start">
                          <span>We monitor</span>
                          <span className="mr-2">:</span>
                        </h2>
                        <li className="flex items-start">
                          <span className="mr-2">›</span>
                          <span>Platform Interactions/Patterns</span>
                        </li>
                        <li className="flex items-start">
                          <span className="mr-2">›</span>
                          <span>Security Event Logs</span>
                        </li>
                        <li className="flex items-start">
                          <span className="mr-2">›</span>
                          <span>Feature Usage Metrics</span>
                        </li>
                      </ul>
                    </div>
                  </div>
                  {/* second */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Data Protection
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      {/* Security Measures */}
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          SECURITY MEASURES
                        </h3>

                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              End-To-End Encryption For All Sensitive Data
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Regular Security Audits And Penetration Testing
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Zero-Knowledge Proof Implementation</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Secure Key Management Systems</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Regular Backup And Recovery Testing</span>
                          </li>
                        </ul>
                      </div>

                      {/* Storage and Retention */}
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          STORAGE AND RETENTION
                        </h3>

                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Encrypted Storage On Distributed System</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Regular Data Minimization Reviews</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Compliance With Retention Regulations</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Secure Data Destruction Protocols</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Backup Maintenance Procedures</span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* thrd */}

                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Third-Party Sharing
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      {/* Service Providers */}
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          SERVICE PROVIDERS
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Limited Sharing With:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Identity Verification Services</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Security Audit Firms</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Legal Compliance Partners</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Technical Infrastructure Providers</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Customer Support Systems</span>
                          </li>
                        </ul>
                      </div>

                      {/* Legal Requirements */}
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          LEGAL REQUIREMENTS
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Data Shared For:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Regulatory Compliance</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Legal Proceedings</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Law Enforcement Requests</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Court Orders</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Fraud Prevention</span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* 4th */}

                  <div className="border-t-2  border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      User Rights and Controls
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      {/* Access Rights */}
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          ACCESS RIGHTS
                        </h3>

                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Request Data Copies</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Correct Personal Information</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Delete Account Data</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Export Transaction History</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Review Security Logs</span>
                          </li>
                        </ul>
                      </div>

                      {/* Privacy Controls */}
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          PRIVACY CONTROLS
                        </h3>

                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Visibility Settings</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Communication Preferences</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Data Sharing Options</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Authentication Methods</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Activity Notifications</span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>

                <Link
                  href="/Guidelines/Code-of-ethics"
                  className="flex justify-between border-b-2 border-[#1C252A] pt-2 hover:border-cyan-400/30 transition-colors cursor-pointer"
                >
                  <h2 className="text-2xl mb-5 font-bold text-[#FCFFFF]">
                    Code of Ethics
                  </h2>
                  <span>
                    <Image
                      src="/ArrowUp.png"
                      alt="arrow"
                      height={30}
                      width={30}
                    />
                  </span>
                </Link>

                <Link
                  href="/Guidelines/Terms-and-Conditions"
                  className="flex justify-between border-b-2 border-[#1C252A] pt-2 hover:border-cyan-400/30 transition-colors cursor-pointer"
                >
                  <h2 className="text-2xl mb-5 font-bold text-[#FCFFFF]">
                    Terms and Conditions
                  </h2>
                  <span>
                    <Image
                      src="/ArrowUp.png"
                      alt="arrow"
                      height={30}
                      width={30}
                    />
                  </span>
                </Link>
              </div>

              <button className="bg-[#33C5E0] hover:bg-[#2ab5cf] transition-colors px-6 py-3 rounded-b-2xl flex items-center gap-2 font-semibold text-black text-sm uppercase tracking-wide w-fit">
                <span>Launch App</span>
                <ArrowUpRight className="w-4 h-4" />
              </button>
            </div>

            {/* Right Column - Contact Support */}
            <div className="lg:col-span-1 relative left-[200px] top-[600px]">
              <div className="flex items-center justify-center gap-2 bg-[#182024] px-4 py-2 rounded-xl w-fit">
                <Headset className="w-5 h-5" />
                <p className="text-sm">Contact Us</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </div>
  );
}
