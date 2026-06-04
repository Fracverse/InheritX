"use client";

import { useState } from "react";
import {
  createVerificationAPI,
  VerificationResult,
} from "@/app/lib/api/verification";
import VerificationResultDisplay from "./VerificationResultDisplay";

interface Props {
  onVerificationComplete?: (result: VerificationResult) => void;
}

export default function HashVerificationTool({
  onVerificationComplete,
}: Props) {
  const [documentId, setDocumentId] = useState("");
  const [hash, setHash] = useState("");
  const [version, setVersion] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<VerificationResult | null>(null);

  const api = createVerificationAPI();

  const handleVerify = async () => {
    if (!documentId.trim()) {
      setError("Please enter a document ID");
      return;
    }

    if (!hash.trim()) {
      setError("Please enter a document hash");
      return;
    }

    // Validate hash format (64 hex characters for SHA-256)
    if (!/^[a-fA-F0-9]{64}$/.test(hash.trim())) {
      setError(
        "Invalid hash format. Please enter a valid SHA-256 hash (64 hexadecimal characters)",
      );
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const versionNum = version ? parseInt(version) : undefined;
      const verificationResult = await api.verifyDocumentByHash(
        documentId,
        hash.trim(),
        versionNum,
      );

      setResult(verificationResult);
      onVerificationComplete?.(verificationResult);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Verification failed. Please try again.",
      );
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setDocumentId("");
    setHash("");
    setVersion("");
    setError(null);
    setResult(null);
  };

  return (
    <div className="space-y-6">
      {/* Input Form */}
      <div className="bg-white rounded-lg shadow-sm p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-6">
          Hash Verification
        </h2>

        <div className="space-y-4">
          {/* Document ID Input */}
          <div>
            <label
              htmlFor="documentId"
              className="block text-sm font-medium text-gray-700 mb-2"
            >
              Document ID *
            </label>
            <input
              type="text"
              id="documentId"
              value={documentId}
              onChange={(e) => setDocumentId(e.target.value)}
              placeholder="e.g., 550e8400-e29b-41d4-a716-446655440000"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-gray-500">
              The unique identifier for the document you want to verify
            </p>
          </div>

          {/* Hash Input */}
          <div>
            <label
              htmlFor="hash"
              className="block text-sm font-medium text-gray-700 mb-2"
            >
              Document Hash (SHA-256) *
            </label>
            <textarea
              id="hash"
              value={hash}
              onChange={(e) => setHash(e.target.value)}
              placeholder="e.g., a3c5f8d2e1b4c6a9f7e3d8b2c1a5f9e4d7c3b8a6f2e1d9c7b5a3f1e8d6c4b2a0"
              rows={3}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-gray-500">
              64-character hexadecimal SHA-256 hash of the document
            </p>
          </div>

          {/* Version Input (Optional) */}
          <div>
            <label
              htmlFor="version"
              className="block text-sm font-medium text-gray-700 mb-2"
            >
              Version (Optional)
            </label>
            <input
              type="number"
              id="version"
              value={version}
              onChange={(e) => setVersion(e.target.value)}
              placeholder="e.g., 1"
              min="1"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-gray-500">
              Leave empty to verify against the latest version
            </p>
          </div>

          {/* Info Box */}
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div className="flex">
              <div className="flex-shrink-0">
                <svg
                  className="h-5 w-5 text-blue-400"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fillRule="evenodd"
                    d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
              <div className="ml-3">
                <h3 className="text-sm font-medium text-blue-800">
                  Why use hash verification?
                </h3>
                <div className="mt-2 text-sm text-blue-700">
                  <ul className="list-disc list-inside space-y-1">
                    <li>Faster than uploading the full document</li>
                    <li>More private - no document content is shared</li>
                    <li>Ideal for third-party verification</li>
                    <li>Can be used programmatically via API</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>

          {/* Error Message */}
          {error && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <div className="flex">
                <div className="flex-shrink-0">
                  <svg
                    className="h-5 w-5 text-red-400"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fillRule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                      clipRule="evenodd"
                    />
                  </svg>
                </div>
                <div className="ml-3">
                  <p className="text-sm text-red-800">{error}</p>
                </div>
              </div>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex space-x-4">
            <button
              onClick={handleVerify}
              disabled={loading || !documentId || !hash}
              className="flex-1 bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
            >
              {loading ? (
                <span className="flex items-center justify-center">
                  <svg
                    className="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    ></circle>
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                  Verifying...
                </span>
              ) : (
                "Verify Hash"
              )}
            </button>
            {(result || error) && (
              <button
                onClick={handleReset}
                className="px-6 py-3 border border-gray-300 rounded-lg font-medium text-gray-700 hover:bg-gray-50 transition-colors"
              >
                Reset
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Verification Result */}
      {result && <VerificationResultDisplay result={result} />}
    </div>
  );
}
