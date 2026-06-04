"use client";

import { useState, useEffect } from "react";
import { FileText, Calendar, Hash, Shield, Eye } from "lucide-react";
import DownloadWillButton from "./DownloadWillButton";

interface WillDocument {
  document_id: string;
  plan_id: string;
  template_used: string;
  will_hash: string;
  generated_at: string;
  version: number;
  filename: string;
}

interface WillDocumentListProps {
  planId: string;
}

export default function WillDocumentList({ planId }: WillDocumentListProps) {
  const [documents, setDocuments] = useState<WillDocument[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [selectedDoc, setSelectedDoc] = useState<string | null>(null);

  useEffect(() => {
    fetchDocuments();
  }, [planId]);

  const fetchDocuments = async () => {
    try {
      setLoading(true);
      const token = localStorage.getItem("auth_token");

      const response = await fetch(`/api/plans/${planId}/will/documents`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        throw new Error("Failed to fetch documents");
      }

      const data = await response.json();
      setDocuments(data.data || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load documents");
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString("en-US", {
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const getTemplateLabel = (template: string) => {
    const labels: Record<string, string> = {
      "Simple Will": "Simple",
      "Formal Legal Will": "Formal",
      "US Jurisdiction Will": "US Legal",
      "UK Jurisdiction Will": "UK Legal",
      "Global Generic Will": "Global",
    };
    return labels[template] || template;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
        {error}
      </div>
    );
  }

  if (documents.length === 0) {
    return (
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-8 text-center">
        <FileText className="mx-auto h-12 w-12 text-gray-400 mb-4" />
        <h3 className="text-lg font-medium text-gray-900 mb-2">
          No Will Documents
        </h3>
        <p className="text-gray-600">
          Generate your first will document to get started.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900">Will Documents</h2>
        <span className="text-sm text-gray-600">
          {documents.length} {documents.length === 1 ? "version" : "versions"}
        </span>
      </div>

      <div className="grid gap-4">
        {documents.map((doc) => (
          <div
            key={doc.document_id}
            className={`
              bg-white border rounded-lg p-6 transition-all duration-200
              ${selectedDoc === doc.document_id ? "border-blue-500 shadow-lg" : "border-gray-200 hover:border-gray-300"}
            `}
          >
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className="bg-blue-100 p-3 rounded-lg">
                  <FileText className="h-6 w-6 text-blue-600" />
                </div>
                <div>
                  <h3 className="text-lg font-semibold text-gray-900">
                    Version {doc.version}
                  </h3>
                  <p className="text-sm text-gray-600">
                    {getTemplateLabel(doc.template_used)}
                  </p>
                </div>
              </div>

              {doc.version === documents[0].version && (
                <span className="bg-green-100 text-green-800 text-xs font-medium px-2.5 py-1 rounded">
                  Latest
                </span>
              )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
              <div className="flex items-center gap-2 text-sm text-gray-600">
                <Calendar className="h-4 w-4" />
                <span>{formatDate(doc.generated_at)}</span>
              </div>

              <div className="flex items-center gap-2 text-sm text-gray-600">
                <Hash className="h-4 w-4" />
                <span className="font-mono truncate" title={doc.will_hash}>
                  {doc.will_hash.substring(0, 16)}...
                </span>
              </div>
            </div>

            <div className="flex items-center gap-3 pt-4 border-t border-gray-200">
              <DownloadWillButton
                documentId={doc.document_id}
                filename={doc.filename}
                variant="primary"
                size="sm"
                className="flex-1"
              />

              <button
                onClick={() => setSelectedDoc(doc.document_id)}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
              >
                <Eye className="h-4 w-4" />
                Details
              </button>

              <button
                onClick={() => {
                  window.open(
                    `/api/will/documents/${doc.document_id}/verify`,
                    "_blank",
                  );
                }}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-blue-700 bg-blue-50 hover:bg-blue-100 rounded-lg transition-colors"
              >
                <Shield className="h-4 w-4" />
                Verify
              </button>
            </div>

            {selectedDoc === doc.document_id && (
              <div className="mt-4 pt-4 border-t border-gray-200">
                <h4 className="text-sm font-semibold text-gray-900 mb-3">
                  Document Details
                </h4>
                <dl className="grid grid-cols-1 gap-3 text-sm">
                  <div>
                    <dt className="text-gray-600">Document ID</dt>
                    <dd className="font-mono text-gray-900 break-all">
                      {doc.document_id}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-gray-600">Plan ID</dt>
                    <dd className="font-mono text-gray-900 break-all">
                      {doc.plan_id}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-gray-600">SHA-256 Hash</dt>
                    <dd className="font-mono text-gray-900 break-all">
                      {doc.will_hash}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-gray-600">Filename</dt>
                    <dd className="text-gray-900">{doc.filename}</dd>
                  </div>
                </dl>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
