"use client";

import Image from "next/image";
import Footer from "../components/Footer";
import Navbar from "../components/Navbar";

export default function PrivacyPolicyPage() {
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
        aria-label="Privacy Policy content"
      >
        <div className="pt-24 md:pt-32">
          {/* Page Header */}
          <div className="mb-12">
            <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">
              Privacy Policy
            </h1>
            <p className="text-slate-400 text-lg">
              How we collect, use, and protect your information
            </p>
          </div>

          {/* Main Content */}
          <div className="space-y-8">
            {/* Data Collection and Usage Section */}
            <div className="bg-slate-800/30 backdrop-blur-sm border border-slate-700 rounded-2xl p-8 hover:border-slate-600 transition-colors">
              <h2 className="text-2xl md:text-3xl font-bold text-white mb-8">
                Data Collection and Usage
              </h2>

              {/* Two Column Layout */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
                {/* Personal Information */}
                <div>
                  <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                    PERSONAL INFORMATION:
                  </h3>
                  <div className="space-y-3">
                    <p className="text-slate-300">We Collect:</p>
                    <ul className="space-y-2 text-slate-300">
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Legal Name And Contact Information</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Government-Issued Identification</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Blockchain Wallet Addresses</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Authentication Credentials</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Communication Preferences</span>
                      </li>
                    </ul>
                  </div>
                </div>

                {/* Technical Data */}
                <div>
                  <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                    TECHNICAL DATA
                  </h3>
                  <div className="space-y-3">
                    <p className="text-slate-300">We Record:</p>
                    <ul className="space-y-2 text-slate-300">
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Device Information</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>IP Addresses</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Browser Type And Version</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Network Information</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>System Logs</span>
                      </li>
                    </ul>
                  </div>
                </div>
              </div>

              {/* Usage Data */}
              <div className="border-t border-slate-700 pt-8">
                <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                  USAGE DATA
                </h3>
                <div className="space-y-3">
                  <p className="text-slate-300">We Monitor:</p>
                  <ul className="space-y-2 text-slate-300">
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Platform Interaction Patterns</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Security Event Logs</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Transaction History</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Access Locations And Times</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Feature Utilization Metrics</span>
                    </li>
                  </ul>
                </div>
              </div>
            </div>

            {/* Data Protection Section */}
            <div className="bg-slate-800/30 backdrop-blur-sm border border-slate-700 rounded-2xl p-8 hover:border-slate-600 transition-colors">
              <h2 className="text-2xl md:text-3xl font-bold text-white mb-8">
                Data Protection
              </h2>

              {/* Two Column Layout */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
                {/* Security Measures */}
                <div>
                  <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                    SECURITY MEASURES
                  </h3>
                  <ul className="space-y-2 text-slate-300">
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>End-to-End Encryption For All Sensitive Data</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Regular Security Audits of Blockchain Storage</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Multi-Factor Authentication Requirements</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Regular Security Audits and Penetration Testing</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Backup and Disaster Recovery Systems</span>
                    </li>
                  </ul>
                </div>

                {/* Storage and Retention */}
                <div>
                  <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                    STORAGE AND RETENTION
                  </h3>
                  <ul className="space-y-2 text-slate-300">
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Encrypted Storage On Distributed Systems</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Regular Data Backups and Redundancy</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Secure Data Centers with Physical Security</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Backup Maintenance Procedures</span>
                    </li>
                  </ul>
                </div>
              </div>
            </div>

            {/* User Rights and Controls Section */}
            <div className="bg-slate-800/30 backdrop-blur-sm border border-slate-700 rounded-2xl p-8 hover:border-slate-600 transition-colors">
              <h2 className="text-2xl md:text-3xl font-bold text-white mb-8">
                User Rights and Controls
              </h2>

              {/* Two Column Layout */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                {/* Access Rights */}
                <div>
                  <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                    ACCESS RIGHTS
                  </h3>
                  <ul className="space-y-2 text-slate-300">
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>View and Download Your Data</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Request Data Copies</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Correct Inaccurate Information</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Export Transaction History</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Review Security Logs</span>
                    </li>
                  </ul>
                </div>

                {/* Privacy Controls */}
                <div>
                  <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                    PRIVACY CONTROLS
                  </h3>
                  <ul className="space-y-2 text-slate-300">
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Limit Data Collection</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Visibility Settings</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Communication Preferences</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Authorization Methods</span>
                    </li>
                    <li className="flex items-start">
                      <span className="mr-2">•</span>
                      <span>Activity Notifications</span>
                    </li>
                  </ul>
                </div>
              </div>
            </div>

            {/* Third-Party Sharing Section */}
            <div className="bg-slate-800/30 backdrop-blur-sm border border-slate-700 rounded-2xl p-8 hover:border-slate-600 transition-colors">
              <h2 className="text-2xl md:text-3xl font-bold text-white mb-8">
                Third-Party Sharing
              </h2>

              {/* Two Column Layout */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                {/* Service Providers */}
                <div>
                  <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                    SERVICE PROVIDERS
                  </h3>
                  <div className="space-y-3">
                    <p className="text-slate-300">Limited Sharing With:</p>
                    <ul className="space-y-2 text-slate-300">
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Identity Verification Partners</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Blockchain Infrastructure Providers</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Legal Compliance Partners</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Technical Infrastructure Providers</span>
                      </li>
                    </ul>
                  </div>
                </div>

                {/* Legal Requirements */}
                <div>
                  <h3 className="text-lg font-semibold text-slate-400 mb-4 uppercase tracking-wide">
                    LEGAL REQUIREMENTS
                  </h3>
                  <div className="space-y-3">
                    <p className="text-slate-300">Data Shared If Required:</p>
                    <ul className="space-y-2 text-slate-300">
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Law Enforcement Requests</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Court Orders</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Regulatory Compliance</span>
                      </li>
                      <li className="flex items-start">
                        <span className="mr-2">•</span>
                        <span>Fraud Prevention</span>
                      </li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </div>
  );
}
