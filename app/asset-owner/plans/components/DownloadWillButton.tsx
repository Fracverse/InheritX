"use client";

import { useState } from "react";
import { Download, Loader2, CheckCircle, AlertCircle } from "lucide-react";

interface DownloadWillButtonProps {
  documentId?: string;
  planId?: string;
  version?: number;
  filename?: string;
  variant?: "primary" | "secondary" | "outline";
  size?: "sm" | "md" | "lg";
  className?: string;
}

export default function DownloadWillButton({
  documentId,
  planId,
  version,
  filename = "will_document.pdf",
  variant = "primary",
  size = "md",
  className = "",
}: DownloadWillButtonProps) {
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadStatus, setDownloadStatus] = useState<
    "idle" | "success" | "error"
  >("idle");
  const [errorMessage, setErrorMessage] = useState("");

  const getDownloadUrl = () => {
    if (documentId) {
      return `/api/will/documents/${documentId}/download`;
    } else if (planId && version) {
      return `/api/plans/${planId}/will/documents/${version}/download`;
    }
    throw new Error("Either documentId or (planId + version) must be provided");
  };

  const handleDownload = async () => {
    try {
      setIsDownloading(true);
      setDownloadStatus("idle");
      setErrorMessage("");

      // Get auth token from localStorage or your auth provider
      const token = localStorage.getItem("auth_token");
      if (!token) {
        throw new Error("Authentication required. Please log in.");
      }

      const url = getDownloadUrl();
      const response = await fetch(url, {
        method: "GET",
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        if (response.status === 401) {
          throw new Error("Authentication failed. Please log in again.");
        } else if (response.status === 404) {
          throw new Error("Document not found or access denied.");
        } else {
          throw new Error("Failed to download document. Please try again.");
        }
      }

      // Get filename from Content-Disposition header if available
      const contentDisposition = response.headers.get("Content-Disposition");
      let downloadFilename = filename;
      if (contentDisposition) {
        const filenameMatch = contentDisposition.match(/filename="(.+)"/);
        if (filenameMatch) {
          downloadFilename = filenameMatch[1];
        }
      }

      // Download the file
      const blob = await response.blob();
      const downloadUrl = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = downloadUrl;
      a.download = downloadFilename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.URL.revokeObjectURL(downloadUrl);

      setDownloadStatus("success");
      setTimeout(() => setDownloadStatus("idle"), 3000);
    } catch (error) {
      console.error("Download error:", error);
      setDownloadStatus("error");
      setErrorMessage(
        error instanceof Error ? error.message : "Download failed",
      );
      setTimeout(() => setDownloadStatus("idle"), 5000);
    } finally {
      setIsDownloading(false);
    }
  };

  const variantClasses = {
    primary: "bg-blue-600 hover:bg-blue-700 text-white",
    secondary: "bg-gray-600 hover:bg-gray-700 text-white",
    outline: "border-2 border-blue-600 text-blue-600 hover:bg-blue-50",
  };

  const sizeClasses = {
    sm: "px-3 py-1.5 text-sm",
    md: "px-4 py-2 text-base",
    lg: "px-6 py-3 text-lg",
  };

  const iconSizes = {
    sm: 16,
    md: 20,
    lg: 24,
  };

  return (
    <div className="flex flex-col gap-2">
      <button
        onClick={handleDownload}
        disabled={isDownloading}
        className={`
          flex items-center justify-center gap-2 rounded-lg font-medium
          transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed
          ${variantClasses[variant]}
          ${sizeClasses[size]}
          ${className}
        `}
      >
        {isDownloading ? (
          <>
            <Loader2 size={iconSizes[size]} className="animate-spin" />
            <span>Downloading...</span>
          </>
        ) : downloadStatus === "success" ? (
          <>
            <CheckCircle size={iconSizes[size]} />
            <span>Downloaded</span>
          </>
        ) : downloadStatus === "error" ? (
          <>
            <AlertCircle size={iconSizes[size]} />
            <span>Failed</span>
          </>
        ) : (
          <>
            <Download size={iconSizes[size]} />
            <span>Download Will</span>
          </>
        )}
      </button>

      {downloadStatus === "error" && errorMessage && (
        <div className="text-sm text-red-600 bg-red-50 px-3 py-2 rounded-md">
          {errorMessage}
        </div>
      )}

      {downloadStatus === "success" && (
        <div className="text-sm text-green-600 bg-green-50 px-3 py-2 rounded-md">
          Document downloaded successfully
        </div>
      )}
    </div>
  );
}
