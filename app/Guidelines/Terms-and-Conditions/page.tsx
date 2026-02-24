"use client";

import Image from "next/image";
import Link from "next/link";
import Footer from "../../components/Footer";
import Navbar from "../../components/Navbar";
import { Headset, ArrowUpRight } from "lucide-react";

export default function TermsAndConditionsPage() {
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
        aria-label="Terms and Conditions content"
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

              {/* Code of Ethics Link */}
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

              {/* Terms and Conditions */}
              <div className="space-y-6">
                <div className="flex justify-between">
                  <h2 className="text-2xl pl-2 md:pl-10 font-bold text-[#FCFFFF]">
                    Terms and Conditions
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
                  {/* Acceptance of Terms */}
                  <div className="mb-10">
                    <h2 className="text-xl mb-10 font-bold text-white">
                      Acceptance of Terms
                    </h2>
                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          AGREEMENT
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          By Using InheritX, You Agree To These Terms And
                          Conditions. If You Do Not Agree, You May Not Use The
                          Platform. Your Continued Use Constitutes Acceptance Of
                          Any Future Modifications.
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              These Terms Govern Your Use Of The InheritX
                              Platform
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              You Must Be At Least 18 Years Old To Use This
                              Service
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              By Creating An Account, You Accept All Terms
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              We May Update These Terms; Continued Use Implies
                              Acceptance
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              You Are Responsible For Reviewing Terms
                              Periodically
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          MODIFICATIONS
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          InheritX Reserves The Right To Modify These Terms At
                          Any Time. We Will Notify Users Of Significant Changes.
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Changes Become Effective Upon Posting</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Material Changes Will Be Communicated Via Email
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Continued Use After Changes Constitutes Acceptance
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              You May Terminate Your Account If You Disagree
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Version History Available Upon Request</span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Using Rules */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Using Rules
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          PERMITTED USES
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Users Agree To Comply With The Following Guidelines:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Use The Platform Only For Lawful Inheritance
                              Planning
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Provide Accurate And Truthful Information At All
                              Times
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Maintain The Security Of Your Account Credentials
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
                              Comply With All Applicable Laws And Regulations
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Report Any Suspicious Activity Or Security
                              Breaches
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          PROHIBITED ACTIVITIES
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Users Must Not Engage In The Following:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Fraudulent Activities Or Misrepresentation
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Unauthorized Access To Other Users&apos; Accounts
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Interference With Platform Operations Or Security
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Transmission Of Malware Or Harmful Code</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Violation Of Intellectual Property Rights
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Harassment Or Abuse Of Other Users Or Staff
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Security Responsibilities */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Security Responsibilities
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          USER OBLIGATIONS
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Users Are Responsible For Maintaining The Security Of
                          Their Accounts:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Keep Login Credentials Confidential And Secure
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Use Strong, Unique Passwords For Your Account
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Enable Two-Factor Authentication When Available
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Notify InheritX Immediately Of Any Unauthorized
                              Access
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Regularly Review Account Activity For Suspicious
                              Behavior
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Do Not Share Account Access With Unauthorized
                              Parties
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          PLATFORM SECURITY
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          InheritX Commits To Protecting User Data Through:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Industry-Standard Encryption For All Data
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
                            <span>
                              Secure Data Storage On Distributed Systems
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Continuous Monitoring For Security Threats
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Prompt Response To Security Incidents</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Compliance With Data Protection Regulations
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Liability */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Liability
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          PLATFORM LIABILITY
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          InheritX&apos;s Liability Is Limited As Follows:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              We Provide The Platform &quot;As Is&quot; Without Warranties
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Not Liable For User Errors Or Misuse Of The
                              Platform
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Limited Liability For Technical Issues Or Downtime
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Not Responsible For Third-Party Service Failures
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Maximum Liability Limited To Fees Paid In Last 12
                              Months
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Not Liable For Indirect Or Consequential Damages
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          USER LIABILITY
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Users Are Liable For:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Accuracy Of Information Provided To The Platform
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Compliance With All Applicable Laws And
                              Regulations
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Actions Taken Using Their Account Credentials
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Any Damages Resulting From Violation Of These
                              Terms
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Indemnifying InheritX Against Claims Arising From
                              User Actions
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Dispute Resolution */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Dispute Resolution
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          RESOLUTION PROCESS
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Disputes Should Be Resolved Through The Following
                          Process:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Contact InheritX Support Team First</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Attempt Good Faith Negotiation To Resolve Issues
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Mediation May Be Required Before Legal Action
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Arbitration Clause May Apply To Certain Disputes
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Class Action Waivers May Apply Where Permitted
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          GOVERNING LAW
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          These Terms Are Governed By:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Laws Of The Jurisdiction Where InheritX Is
                              Registered
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>International Laws Where Applicable</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Data Protection And Privacy Regulations</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Consumer Protection Laws In Your Jurisdiction
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Inheritance And Estate Planning Regulations
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Service Termination */}
                  <div className="border-t-2 mb-10 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Service Termination
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          BY USER
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Users May Terminate Their Account At Any Time By:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Submitting A Termination Request Through The
                              Platform
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Contacting Customer Support For Account Closure
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Completing Any Outstanding Obligations Before
                              Closure
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Requesting Data Export Before Account Deletion
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Understanding That Some Data May Be Retained For
                              Legal Compliance
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          BY INHERITX
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          InheritX May Terminate Accounts For:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Violation Of These Terms And Conditions</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Fraudulent Or Illegal Activity</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Extended Period Of Account Inactivity</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Non-Payment Of Fees Or Services</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              At Our Discretion With Notice Where Legally
                              Required
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* Modification of Terms */}
                  <div className="border-t-2 border-[#1C252A] pt-2">
                    <h2 className="text-xl mt-5 mb-10 font-bold text-white">
                      Modification of Terms
                    </h2>

                    <div className="flex flex-col md:flex-row space-y-[30px] md:space-y-0 md:gap-8">
                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          UPDATES
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          We May Update These Terms And Conditions As Needed
                          For:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Legal Compliance</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Service Improvements</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Security Enhancements</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Policy Changes</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Changes Will Be Posted With Effective Date
                            </span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Continued Use Of The Platform Constitutes
                              Acceptance Of Updated Terms
                            </span>
                          </li>
                        </ul>
                      </div>

                      <div className="flex-1">
                        <h3 className="text-base font-semibold text-[#92A5A8] mb-4 border-b border-[#1C252A] pb-2">
                          NOTIFICATION
                        </h3>
                        <p className="text-slate-400 text-sm mb-3">
                          Users Will Be Notified Of Changes Through:
                        </p>
                        <ul className="space-y-2 text-slate-300">
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Email Notifications</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Platform Announcements</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>In-App Messages</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>Updated Terms Page</span>
                          </li>
                          <li className="flex items-start">
                            <span className="mr-2">›</span>
                            <span>
                              Notice Period Before Changes Take Effect
                            </span>
                          </li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
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
