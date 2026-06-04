"use client";

import { useState } from "react";
import { VerificationResult } from "@/app/lib/api/verification";
import VerificationCertificate from "./VerificationCertificate";

interface Props {
  result: VerificationResult;
  computedHash?: string;
}

export default function VerificationResultDisplay({
  result,
  computedHash,
}: Props) {
  const [showCertificate, setShowCertificate] = useState(false);

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString("en-US", {
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      timeZoneName: "short",
    });
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <div className="space-y-6">
      {/* Status Banner */}
      <div
        className={`rounded-lg p-6 ${
          result.is_valid
            ? "bg-green-50 border-2 border-green-500"
            : "bg-red-50 border-2 border-red-500"
        }`}
      >
        <div className="flex items-center">
          <div className="flex-shrink-0">
            {result.is_valid ? (
              <svg
                className="h-12 w-12 text-green-500"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            ) : (
              <svg
                className="h-12 w-12 text-red-500"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            )}
          </div>
          <div className="ml-4">
            <h3
              className={`text-2xl font-bold ${
                result.is_valid ? "text-green-900" : "text-red-900"
              }`}
            >
              {result.is_valid
                ? "✓ Document Verified"
                : "✗ Verification Failed"}
            </h3>
            <p
              className={`mt-1 ${
                result.is_valid ? "text-green-700" : "text-red-700"
              }`}
            >
              {result.is_valid
                ? "The document is authentic and has not been modified"
                : "The document hash does not match the stored record"}
            </p>
          </div>
        </div>
      </div>

      {/* Verification Details */}
      <div className="bg-white rounded-lg shadow-sm p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          Verification Details
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Document Information */}
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium text-gray-500">
                Document ID
              </label>
              <div className="mt-1 flex items-center space-x-2">
                <code className="text-sm bg-gray-100 px-2 py-1 rounded font-mono">
                  {result.document_id}
                </code>
                <button
                  onClick={() => copyToClipboard(result.document_id)}
                  className="text-blue-600 hover:text-blue-800"
                  title="Copy to clipboard"
                >
                  <svg
                    className="h-4 w-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                    />
                  </svg>
                </button>
              </div>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-500">
                Plan ID
              </label>
              <div className="mt-1">
                <code className="text-sm bg-gray-100 px-2 py-1 rounded font-mono">
                  {result.plan_id}
                </code>
              </div>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-500">
                Version
              </label>
              <div className="mt-1 flex items-center space-x-2">
                <span className="text-sm font-semibold">{result.version}</span>
                {result.details.is_active_version && (
                  <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                    Active
                  </span>
                )}
              </div>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-500">
                Status
              </label>
              <div className="mt-1">
                <span
                  className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                    result.details.status === "finalized"
                      ? "bg-green-100 text-green-800"
                      : "bg-yellow-100 text-yellow-800"
                  }`}
                >
                  {result.details.status}
                </span>
              </div>
            </div>
          </div>

          {/* Verification Information */}
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium text-gray-500">
                Template
              </label>
              <div className="mt-1 text-sm">{result.details.template}</div>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-500">
                Generated At
              </label>
              <div className="mt-1 text-sm">
                {formatDate(result.details.generated_at)}
              </div>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-500">
                Verified At
              </label>
              <div className="mt-1 text-sm">
                {formatDate(result.verified_at)}
              </div>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-500">
                Encryption
              </label>
              <div className="mt-1">
                <span
                  className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                    result.details.is_encrypted
                      ? "bg-purple-100 text-purple-800"
                      : "bg-gray-100 text-gray-800"
                  }`}
                >
                  {result.details.is_encrypted ? "Encrypted" : "Not Encrypted"}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Hash Comparison */}
      <div className="bg-white rounded-lg shadow-sm p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          Hash Comparison
        </h3>

        <div className="space-y-4">
          <div>
            <label className="text-sm font-medium text-gray-500">
              Stored Hash (Database)
            </label>
            <div className="mt-1 flex items-center space-x-2">
              <code className="text-xs bg-gray-100 px-3 py-2 rounded font-mono break-all flex-1">
                {result.stored_hash}
              </code>
              <button
                onClick={() => copyToClipboard(result.stored_hash)}
                className="text-blue-600 hover:text-blue-800 flex-shrink-0"
                title="Copy to clipboard"
              >
                <svg
                  className="h-4 w-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                  />
                </svg>
              </button>
            </div>
          </div>

          {result.computed_hash && (
            <div>
              <label className="text-sm font-medium text-gray-500">
                Computed Hash (Provided Document)
              </label>
              <div className="mt-1 flex items-center space-x-2">
                <code
                  className={`text-xs px-3 py-2 rounded font-mono break-all flex-1 ${
                    result.is_valid
                      ? "bg-green-100 text-green-900"
                      : "bg-red-100 text-red-900"
                  }`}
                >
                  {result.computed_hash}
                </code>
                <button
                  onClick={() =>
                    copyToClipboard(result.computed_hash as string)
                  }
                  className="text-blue-600 hover:text-blue-800 flex-shrink-0"
                  title="Copy to clipboard"
                >
                  <svg
                    className="h-4 w-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                    />
                  </svg>
                </button>
              </div>
            </div>
          )}

          {computedHash && (
            <div>
              <label className="text-sm font-medium text-gray-500">
                Client-Side Computed Hash
              </label>
              <div className="mt-1 flex items-center space-x-2">
                <code className="text-xs bg-blue-50 px-3 py-2 rounded font-mono break-all flex-1">
                  {computedHash}
                </code>
                <button
                  onClick={() => copyToClipboard(computedHash)}
                  className="text-blue-600 hover:text-blue-800 flex-shrink-0"
                  title="Copy to clipboard"
                >
                  <svg
                    className="h-4 w-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                    />
                  </svg>
                </button>
              </div>
            </div>
          )}

          <div
            className={`flex items-center space-x-2 p-3 rounded-lg ${
              result.is_valid ? "bg-green-50" : "bg-red-50"
            }`}
          >
            <span
              className={`text-2xl ${
                result.is_valid ? "text-green-600" : "text-red-600"
              }`}
            >
              {result.is_valid ? "=" : "≠"}
            </span>
            <span
              className={`text-sm font-medium ${
                result.is_valid ? "text-green-900" : "text-red-900"
              }`}
            >
              {result.is_valid ? "Hashes Match" : "Hashes Do Not Match"}
            </span>
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="flex space-x-4">
        <button
          onClick={() => setShowCertificate(true)}
          className="flex-1 bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 transition-colors"
        >
          📜 Generate Certificate
        </button>
        <button
          onClick={() => window.print()}
          className="px-6 py-3 border border-gray-300 rounded-lg font-medium text-gray-700 hover:bg-gray-50 transition-colors"
        >
          🖨️ Print
        </button>
      </div>

      {/* Certificate Modal */}
      {showCertificate && (
        <VerificationCertificate
          result={result}
          onClose={() => setShowCertificate(false)}
        />
      )}
    </div>
  );
}
