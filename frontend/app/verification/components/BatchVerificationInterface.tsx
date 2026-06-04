"use client";

import { useState } from "react";
import {
  createVerificationAPI,
  BatchVerificationResult,
  VerificationResult,
} from "@/app/lib/api/verification";

export default function BatchVerificationInterface() {
  const [planId, setPlanId] = useState("");
  const [authToken, setAuthToken] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<BatchVerificationResult | null>(null);
  const [selectedResult, setSelectedResult] =
    useState<VerificationResult | null>(null);

  const api = createVerificationAPI();

  const handleVerify = async () => {
    if (!planId.trim()) {
      setError("Please enter a plan ID");
      return;
    }

    if (!authToken.trim()) {
      setError("Please enter an authentication token");
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);
    setSelectedResult(null);

    try {
      const batchResult = await api.verifyAllVersions(
        planId.trim(),
        authToken.trim(),
      );
      setResult(batchResult);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Batch verification failed. Please check your credentials and try again.",
      );
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setPlanId("");
    setAuthToken("");
    setError(null);
    setResult(null);
    setSelectedResult(null);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <div className="space-y-6">
      {/* Input Form */}
      <div className="bg-white rounded-lg shadow-sm p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-6">
          Batch Verification
        </h2>

        <div className="space-y-4">
          {/* Plan ID Input */}
          <div>
            <label
              htmlFor="planId"
              className="block text-sm font-medium text-gray-700 mb-2"
            >
              Plan ID *
            </label>
            <input
              type="text"
              id="planId"
              value={planId}
              onChange={(e) => setPlanId(e.target.value)}
              placeholder="e.g., 550e8400-e29b-41d4-a716-446655440000"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-gray-500">
              The plan ID for which to verify all document versions
            </p>
          </div>

          {/* Auth Token Input */}
          <div>
            <label
              htmlFor="authToken"
              className="block text-sm font-medium text-gray-700 mb-2"
            >
              Authentication Token *
            </label>
            <input
              type="password"
              id="authToken"
              value={authToken}
              onChange={(e) => setAuthToken(e.target.value)}
              placeholder="Enter your authentication token"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-gray-500">
              Required for batch verification (plan owner only)
            </p>
          </div>

          {/* Info Box */}
          <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
            <div className="flex">
              <div className="flex-shrink-0">
                <svg
                  className="h-5 w-5 text-yellow-400"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fillRule="evenodd"
                    d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
              <div className="ml-3">
                <h3 className="text-sm font-medium text-yellow-800">
                  Authentication Required
                </h3>
                <div className="mt-2 text-sm text-yellow-700">
                  <p>
                    Batch verification requires authentication and can only be
                    performed by the plan owner. This ensures privacy and
                    security of document records.
                  </p>
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
              disabled={loading || !planId || !authToken}
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
                  Verifying All Versions...
                </span>
              ) : (
                "Verify All Versions"
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

      {/* Results Summary */}
      {result && (
        <div className="bg-white rounded-lg shadow-sm p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">
            Verification Summary
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
            <div className="bg-blue-50 rounded-lg p-4">
              <div className="text-2xl font-bold text-blue-900">
                {result.count}
              </div>
              <div className="text-sm text-blue-700">Total Versions</div>
            </div>
            <div className="bg-green-50 rounded-lg p-4">
              <div className="text-2xl font-bold text-green-900">
                {result.valid_count}
              </div>
              <div className="text-sm text-green-700">Valid Documents</div>
            </div>
            <div className="bg-red-50 rounded-lg p-4">
              <div className="text-2xl font-bold text-red-900">
                {result.invalid_count}
              </div>
              <div className="text-sm text-red-700">Invalid Documents</div>
            </div>
          </div>

          {/* Results Table */}
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Version
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Validity
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Generated
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {result.results.map((r) => (
                  <tr
                    key={r.document_id}
                    className={
                      r.is_valid ? "hover:bg-gray-50" : "bg-red-50 hover:bg-red-100"
                    }
                  >
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center space-x-2">
                        <span className="text-sm font-medium text-gray-900">
                          v{r.version}
                        </span>
                        {r.details.is_active_version && (
                          <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                            Active
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span
                        className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                          r.details.status === "finalized"
                            ? "bg-green-100 text-green-800"
                            : "bg-yellow-100 text-yellow-800"
                        }`}
                      >
                        {r.details.status}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {r.is_valid ? (
                        <span className="inline-flex items-center text-green-600">
                          <svg
                            className="h-5 w-5 mr-1"
                            fill="currentColor"
                            viewBox="0 0 20 20"
                          >
                            <path
                              fillRule="evenodd"
                              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                              clipRule="evenodd"
                            />
                          </svg>
                          Valid
                        </span>
                      ) : (
                        <span className="inline-flex items-center text-red-600">
                          <svg
                            className="h-5 w-5 mr-1"
                            fill="currentColor"
                            viewBox="0 0 20 20"
                          >
                            <path
                              fillRule="evenodd"
                              d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                              clipRule="evenodd"
                            />
                          </svg>
                          Invalid
                        </span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {formatDate(r.details.generated_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      <button
                        onClick={() => setSelectedResult(r)}
                        className="text-blue-600 hover:text-blue-900 font-medium"
                      >
                        View Details
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Selected Result Details Modal */}
      {selectedResult && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto p-6">
            <div className="flex justify-between items-start mb-4">
              <h3 className="text-xl font-semibold text-gray-900">
                Version {selectedResult.version} Details
              </h3>
              <button
                onClick={() => setSelectedResult(null)}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg
                  className="h-6 w-6"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium text-gray-500">
                  Document ID
                </label>
                <code className="block text-xs bg-gray-100 px-3 py-2 rounded font-mono mt-1">
                  {selectedResult.document_id}
                </code>
              </div>

              <div>
                <label className="text-sm font-medium text-gray-500">
                  Stored Hash
                </label>
                <code className="block text-xs bg-gray-100 px-3 py-2 rounded font-mono mt-1 break-all">
                  {selectedResult.stored_hash}
                </code>
              </div>

              {selectedResult.computed_hash && (
                <div>
                  <label className="text-sm font-medium text-gray-500">
                    Computed Hash
                  </label>
                  <code
                    className={`block text-xs px-3 py-2 rounded font-mono mt-1 break-all ${
                      selectedResult.is_valid
                        ? "bg-green-100 text-green-900"
                        : "bg-red-100 text-red-900"
                    }`}
                  >
                    {selectedResult.computed_hash}
                  </code>
                </div>
              )}

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-sm font-medium text-gray-500">
                    Template
                  </label>
                  <div className="text-sm mt-1">
                    {selectedResult.details.template}
                  </div>
                </div>
                <div>
                  <label className="text-sm font-medium text-gray-500">
                    Encryption
                  </label>
                  <div className="text-sm mt-1">
                    {selectedResult.details.is_encrypted ? "Yes" : "No"}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
