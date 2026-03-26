"use client";

import { useParams } from "next/navigation";
import { useState } from "react";
import WillDocumentList from "../../components/WillDocumentList";
import { ArrowLeft, Download, FileCheck } from "lucide-react";
import Link from "next/link";
import {
  downloadMultipleVersions,
  verifyAndDownload,
} from "@/app/hooks/useWillDownload";

export default function WillDocumentsPage() {
  const params = useParams();
  const planId = params.id as string;
  const [batchDownloading, setBatchDownloading] = useState(false);
  const [batchResult, setBatchResult] = useState<string>("");

  const handleDownloadAll = async () => {
    try {
      setBatchDownloading(true);
      setBatchResult("");

      const token = localStorage.getItem("auth_token");
      if (!token) {
        setBatchResult("Authentication required");
        return;
      }

      // Fetch all document versions
      const response = await fetch(`/api/plans/${planId}/will/documents`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        throw new Error("Failed to fetch documents");
      }

      const data = await response.json();
      const versions = data.data.map((doc: any) => doc.version);

      if (versions.length === 0) {
        setBatchResult("No documents to download");
        return;
      }

      // Download all versions
      const result = await downloadMultipleVersions(planId, versions, token);

      setBatchResult(
        `Downloaded ${result.success} of ${versions.length} documents. ${
          result.failed > 0 ? `Failed: ${result.failed}` : ""
        }`,
      );
    } catch (error) {
      setBatchResult(
        error instanceof Error ? error.message : "Batch download failed",
      );
    } finally {
      setBatchDownloading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <Link
            href={`/asset-owner/plans/${planId}`}
            className="inline-flex items-center gap-2 text-blue-600 hover:text-blue-700 mb-4"
          >
            <ArrowLeft className="h-4 w-4" />
            Back to Plan
          </Link>

          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">
                Will Documents
              </h1>
              <p className="text-gray-600 mt-2">
                View and download all versions of your will documents
              </p>
            </div>

            <div className="flex gap-3">
              <button
                onClick={handleDownloadAll}
                disabled={batchDownloading}
                className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {batchDownloading ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    <span>Downloading...</span>
                  </>
                ) : (
                  <>
                    <Download className="h-4 w-4" />
                    <span>Download All</span>
                  </>
                )}
              </button>
            </div>
          </div>

          {batchResult && (
            <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded-lg text-blue-700">
              {batchResult}
            </div>
          )}
        </div>

        {/* Document List */}
        <WillDocumentList planId={planId} />

        {/* Info Section */}
        <div className="mt-8 bg-white border border-gray-200 rounded-lg p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            About Will Documents
          </h2>
          <div className="space-y-4 text-sm text-gray-600">
            <div className="flex items-start gap-3">
              <FileCheck className="h-5 w-5 text-blue-600 mt-0.5 flex-shrink-0" />
              <div>
                <p className="font-medium text-gray-900">Version Control</p>
                <p>
                  Each time you generate a new will, a new version is created.
                  All previous versions are preserved for your records.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <FileCheck className="h-5 w-5 text-blue-600 mt-0.5 flex-shrink-0" />
              <div>
                <p className="font-medium text-gray-900">Document Integrity</p>
                <p>
                  Each document has a unique SHA-256 hash that ensures its
                  integrity. You can verify that a document hasn't been tampered
                  with using the Verify button.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <FileCheck className="h-5 w-5 text-blue-600 mt-0.5 flex-shrink-0" />
              <div>
                <p className="font-medium text-gray-900">Secure Downloads</p>
                <p>
                  All downloads are authenticated and logged for security. Only
                  you can access your will documents.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <FileCheck className="h-5 w-5 text-blue-600 mt-0.5 flex-shrink-0" />
              <div>
                <p className="font-medium text-gray-900">Legal Validity</p>
                <p>
                  Downloaded documents are legally formatted PDFs that can be
                  printed, signed, and witnessed according to your
                  jurisdiction's requirements.
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Security Notice */}
        <div className="mt-6 bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <div className="flex items-start gap-3">
            <svg
              className="h-5 w-5 text-yellow-600 mt-0.5 flex-shrink-0"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fillRule="evenodd"
                d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                clipRule="evenodd"
              />
            </svg>
            <div className="text-sm text-yellow-800">
              <p className="font-medium">Security Notice</p>
              <p className="mt-1">
                Store downloaded will documents securely. These are legal
                documents containing sensitive information about your estate
                planning.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
