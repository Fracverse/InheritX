"use client";

import Image from "next/image";
import Link from "next/link";
import Footer from "../../components/Footer";
import Navbar from "../../components/Navbar";
import { Headset, ArrowUpRight } from "lucide-react";

export default function CodeOfEthicsPage() {
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
        aria-label="Code of Ethics content"
      >
        <div className="pt-5 md:pt-10">
          {/* Page Header */}
          <div className="mb-12">
            <h1 className="text-4xl font-bold text-white mb-4">Guidelines</h1>
            <p className="text-[#92A5A8]">
              Here are some important social dynamics about InheritX
            </p>
          </div>

          {/* Two Column Layout */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {/* Left Column - Main Content */}
            <div className="lg:col-span-2 space-y-12">
              {/* Code of Ethics */}

              <Link
                href="/Guidelines"
                className="flex justify-between border-b-2 border-[#1C252A] pt-2 hover:border-cyan-400/30 transition-colors cursor-pointer"
              >
                <h2 className="text-2xl mb-5 font-bold text-[#FCFFFF]">
                  Privacy Policy
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
              <div className="space-y-6">
                <div className="flex justify-between">
                  <h2 className="text-2xl pl-2 md:pl-10 font-bold text-[#FCFFFF]">
                    Code of Ethics
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

                {/* Main Content Box */}
                <div className="relative rounded-tr-[50px] rounded-br-[50px] p-4 md:p-10 bg-[#161E22] border-2 border-[#161E22] shadow-[0_0_30px_rgba(0,0,0,0.9)]">
                  {/* Core Principles */}
                  <div className="mb-10">
                    <h2 className="text-xl mb-10 font-bold text-white">
                      Core Principles
                    </h2>
                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          TRANSPARENCY
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We Are Committed To Open And Honest:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Clear Communication About All Processes</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Honest Disclosure Of All Costs</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Open Sharing Of Information With Stakeholders
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Transparent Decision-Making In All Operations
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Clear Explanation Of Rights And Obligations
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Offering No Surprises In Legal Or Financial
                              Matters
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          INTEGRITY
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We Uphold The Ideals Of User Assets Through:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Ethical Conduct In All Interactions</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Respect For User Privacy And Autonomy</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Prioritizing Beneficiaries&apos; Interests Above All
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Maintaining Professional Standards In Service
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Adhering Strictly To Legal And Regulatory
                              Requirements
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Ensuring Upmost User Security And Data</span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Fairness */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Fairness
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          EQUITABLE TREATMENT
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We Strive For Fairness By:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Ensuring Fair And Impartial Treatment Of All Users
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Providing Equal Access To Services Regardless Of
                              Background
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Eliminating Bias In All Processes</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Respecting Cultural And Individual Differences
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Ensuring Equitable Distribution Of Resources And
                              Benefits
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          IMPARTIALITY
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We Maintain Neutrality Through:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Avoiding Conflicts Of Interest In All Dealings
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Making Decisions Based On Merit And Facts
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Treating All Parties With Equal Respect And
                              Consideration
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Ensuring Objective Evaluation Of All Situations
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Maintaining Independence In Decision-Making
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Confidentiality / Stewardship */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Confidentiality / Stewardship
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          PRIVACY PROTECTION
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We Value All Shared And Safeguarded User Information
                          By:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Protecting Sensitive User Information At All Times
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Limiting Access To Authorized Personnel Only
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Upholding The Confidentiality Of All User
                              Communications
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Implementing Robust Security Measures For Data
                              Protection
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Ensuring Strict Adherence To Privacy Laws And
                              Regulations
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Preventing Ethical Data Leaks And Negligence
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          RESPONSIBLE MANAGEMENT
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We Manage All Assets Carefully And Ethically By:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Acting As Responsible Custodians Of All User
                              Assets
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Ensuring Prudent And Ethical Management Of
                              Resources
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Prioritizing Long-Term Sustainability Over
                              Short-Term Gains
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Maintaining Accountability For All Actions And
                              Decisions
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Demonstrating Fiscal Responsibility In All
                              Operations
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* User & Platform Responsibilities */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      User & Platform Responsibilities
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          USER RESPONSIBILITIES
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          To Maintain Platform Integrity, Users Must:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Provide Accurate And Truthful Information In All
                              Interactions
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Respect The Rights And Privacy Of Other Users
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Report Suspicious Activity To Authorities
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Keep Login Credentials Secure And Confidential
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Use The Platform Only For Its Intended Purposes
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Comply With All Applicable Laws And Regulations
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          PLATFORM RESPONSIBILITIES
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          As A Responsible Service Provider, We Commit To:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Providing Clear Instructions Through The
                              Inheritance Process
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Maintaining Robust Security Infrastructure
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Responding To User Inquiries In A Timely Manner
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Continuously Improving Our Services</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Protecting User Data From Unauthorized Access
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Ensuring Reliable Infrastructure And Support
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* User Rights and Controls */}
                  <div className="border-t-2 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      User Rights and Controls
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          USER RIGHTS
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Users Have:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Request And Access Copies</span>
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
                            <span>Withdraw Consent</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Lodge Complaints</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Portability Of Data</span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          LIMITATIONS
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Users May Not:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Violate Terms Of Use</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Engage In Fraudulent Activities</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Share Accounts</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Interfere With Platform Operations</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Attempt Unauthorized Access</span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Compliance Framework */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Compliance Framework
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          REGULATORY ADHERENCE
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We Commit To Full Compliance With All Applicable
                          Standards By:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Following All Relevant Laws And Regulations
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Maintaining Up-To-Date Knowledge Of Legal
                              Requirements
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Cooperating With Regulatory Bodies And Auditors
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Implementing Industry Best Practices And Standards
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Conducting Regular Compliance Reviews And Audits
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          ETHICAL STANDARDS
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We Uphold The Highest Professional And Ethical
                          Standards By:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Adhering To Industry Codes Of Conduct</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Promoting Ethical Behavior At All Levels Of The
                              Organization
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Providing Ethics Training To All Employees
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Encouraging Reporting Of Ethical Concerns
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Addressing Ethical Breaches Promptly And
                              Comprehensively
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Maintaining Ethical Business Practices</span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>

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
