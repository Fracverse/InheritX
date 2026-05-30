"use client";

import { useState } from "react";
import DocumentUploadVerifier from "./components/DocumentUploadVerifier";
import HashVerificationTool from "./components/HashVerificationTool";
import BatchVerificationInterface from "./components/BatchVerificationInterface";
import { VerificationResult } from "@/app/lib/api/verification";

export default function VerificationPage() {
  const [activeTab, setActiveTab] = useState<
    "upload" | "hash" | "batch" | "about"
  >("upload");
  const [verificationResult, setVerificationResult] =
    useState<VerificationResult | null>(null);

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100">
      {/* Header */}
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">
                Document Verification
              </h1>
              <p className="mt-2 text-sm text-gray-600">
                Verify the integrity and authenticity of legal will documents
              </p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600">Public Service</span>
            </div>
          </div>
        </div>
      </header>

      {/* Navigation Tabs */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8">
        <div className="bg-white rounded-lg shadow-sm p-1 inline-flex space-x-1">
          <button
            onClick={() => setActiveTab("upload")}
            className={`px-6 py-3 rounded-md text-sm font-medium transition-all ${
              activeTab === "upload"
                ? "bg-blue-600 text-white shadow-sm"
                : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
            }`}
          >
            📄 Upload Document
          </button>
          <button
            onClick={() => setActiveTab("hash")}
            className={`px-6 py-3 rounded-md text-sm font-medium transition-all ${
              activeTab === "hash"
                ? "bg-blue-600 text-white shadow-sm"
                : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
            }`}
          >
            🔐 Hash Verification
          </button>
          <button
            onClick={() => setActiveTab("batch")}
            className={`px-6 py-3 rounded-md text-sm font-medium transition-all ${
              activeTab === "batch"
                ? "bg-blue-600 text-white shadow-sm"
                : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
            }`}
          >
            📊 Batch Verification
          </button>
          <button
            onClick={() => setActiveTab("about")}
            className={`px-6 py-3 rounded-md text-sm font-medium transition-all ${
              activeTab === "about"
                ? "bg-blue-600 text-white shadow-sm"
                : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
            }`}
          >
            ℹ️ About
          </button>
        </div>
      </div>

      {/* Content Area */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {activeTab === "upload" && (
          <DocumentUploadVerifier
            onVerificationComplete={setVerificationResult}
          />
        )}
        {activeTab === "hash" && (
          <HashVerificationTool
            onVerificationComplete={setVerificationResult}
          />
        )}
        {activeTab === "batch" && <BatchVerificationInterface />}
        {activeTab === "about" && <AboutSection />}
      </div>

      {/* Footer */}
      <footer className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 mt-16">
        <div className="border-t pt-8">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div>
              <h3 className="text-sm font-semibold text-gray-900 mb-3">
                Security
              </h3>
              <p className="text-sm text-gray-600">
                All verifications use SHA-256 cryptographic hashing. Documents
                are never stored during verification.
              </p>
            </div>
            <div>
              <h3 className="text-sm font-semibold text-gray-900 mb-3">
                Privacy
              </h3>
              <p className="text-sm text-gray-600">
                Verification is performed locally in your browser. No document
                content is transmitted to our servers.
              </p>
            </div>
            <div>
              <h3 className="text-sm font-semibold text-gray-900 mb-3">
                Support
              </h3>
              <p className="text-sm text-gray-600">
                For assistance with document verification, contact our support
                team or refer to the documentation.
              </p>
            </div>
          </div>
          <div className="mt-8 pt-8 border-t text-center text-sm text-gray-500">
            © {new Date().getFullYear()} InheritX. All rights reserved.
          </div>
        </div>
      </footer>
    </div>
  );
}

function AboutSection() {
  return (
    <div className="bg-white rounded-lg shadow-sm p-8">
      <h2 className="text-2xl font-bold text-gray-900 mb-6">
        About Document Verification
      </h2>

      <div className="space-y-6 text-gray-600">
        <section>
          <h3 className="text-lg font-semibold text-gray-900 mb-3">
            What is Document Verification?
          </h3>
          <p>
            Document verification ensures that a legal will document has not
            been altered or tampered with since its creation. Each document is
            assigned a unique cryptographic hash (fingerprint) that changes if
            even a single character is modified.
          </p>
        </section>

        <section>
          <h3 className="text-lg font-semibold text-gray-900 mb-3">
            How It Works
          </h3>
          <ol className="list-decimal list-inside space-y-2">
            <li>
              When a will document is generated, a SHA-256 hash is computed and
              stored securely
            </li>
            <li>
              To verify a document, you can upload it or provide its hash
            </li>
            <li>
              The system computes the hash of your document and compares it
              with the stored hash
            </li>
            <li>
              If the hashes match, the document is authentic and unmodified
            </li>
          </ol>
        </section>

        <section>
          <h3 className="text-lg font-semibold text-gray-900 mb-3">
            Verification Methods
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="border rounded-lg p-4">
              <h4 className="font-semibold text-gray-900 mb-2">
                📄 Upload Document
              </h4>
              <p className="text-sm">
                Upload a PDF document to verify its integrity. The hash is
                computed locally in your browser.
              </p>
            </div>
            <div className="border rounded-lg p-4">
              <h4 className="font-semibold text-gray-900 mb-2">
                🔐 Hash Verification
              </h4>
              <p className="text-sm">
                Verify using a document hash without uploading the full
                document. Faster and more private.
              </p>
            </div>
            <div className="border rounded-lg p-4">
              <h4 className="font-semibold text-gray-900 mb-2">
                📊 Batch Verification
              </h4>
              <p className="text-sm">
                Verify all versions of a plan&apos;s documents at once.
                Requires authentication.
              </p>
            </div>
          </div>
        </section>

        <section>
          <h3 className="text-lg font-semibold text-gray-900 mb-3">
            Security & Privacy
          </h3>
          <ul className="list-disc list-inside space-y-2">
            <li>
              All hash computations are performed locally in your browser using
              the Web Crypto API
            </li>
            <li>
              Document content is never transmitted to our servers during
              verification
            </li>
            <li>
              Only the computed hash is sent for comparison with stored records
            </li>
            <li>
              Verification results include timestamps and can be downloaded as
              certificates
            </li>
          </ul>
        </section>

        <section className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h3 className="text-lg font-semibold text-blue-900 mb-2">
            🔒 Cryptographic Security
          </h3>
          <p className="text-blue-800">
            We use SHA-256, a cryptographic hash function that produces a unique
            256-bit signature for each document. It is computationally
            infeasible to create two different documents with the same hash,
            ensuring document authenticity.
          </p>
        </section>
      </div>
    </div>
  );
}
