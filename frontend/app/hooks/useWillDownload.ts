"use client";

import { useState, useCallback } from "react";

interface DownloadOptions {
  documentId?: string;
  planId?: string;
  version?: number;
  filename?: string;
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

interface DownloadState {
  isDownloading: boolean;
  error: string | null;
  success: boolean;
}

export function useWillDownload() {
  const [state, setState] = useState<DownloadState>({
    isDownloading: false,
    error: null,
    success: false,
  });

  const download = useCallback(async (options: DownloadOptions) => {
    const {
      documentId,
      planId,
      version,
      filename = "will_document.pdf",
      onSuccess,
      onError,
    } = options;

    try {
      setState({ isDownloading: true, error: null, success: false });

      // Validate inputs
      if (!documentId && (!planId || !version)) {
        throw new Error(
          "Either documentId or (planId + version) must be provided",
        );
      }

      // Get auth token
      const token = localStorage.getItem("auth_token");
      if (!token) {
        throw new Error("Authentication required. Please log in.");
      }

      // Construct URL
      const url = documentId
        ? `/api/will/documents/${documentId}/download`
        : `/api/plans/${planId}/will/documents/${version}/download`;

      // Fetch document
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
          throw new Error(`Download failed with status ${response.status}`);
        }
      }

      // Extract filename from Content-Disposition header
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

      setState({ isDownloading: false, error: null, success: true });
      onSuccess?.();

      // Reset success state after 3 seconds
      setTimeout(() => {
        setState((prev) => ({ ...prev, success: false }));
      }, 3000);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Download failed";
      setState({ isDownloading: false, error: errorMessage, success: false });
      onError?.(error instanceof Error ? error : new Error(errorMessage));

      // Reset error state after 5 seconds
      setTimeout(() => {
        setState((prev) => ({ ...prev, error: null }));
      }, 5000);
    }
  }, []);

  const reset = useCallback(() => {
    setState({ isDownloading: false, error: null, success: false });
  }, []);

  return {
    download,
    reset,
    ...state,
  };
}

// Utility function for batch downloads
export async function downloadMultipleVersions(
  planId: string,
  versions: number[],
  token: string,
): Promise<{ success: number; failed: number; errors: string[] }> {
  const results = {
    success: 0,
    failed: 0,
    errors: [] as string[],
  };

  for (const version of versions) {
    try {
      const url = `/api/plans/${planId}/will/documents/${version}/download`;
      const response = await fetch(url, {
        method: "GET",
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        throw new Error(`Version ${version} download failed`);
      }

      const blob = await response.blob();
      const downloadUrl = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = downloadUrl;
      a.download = `will_v${version}.pdf`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.URL.revokeObjectURL(downloadUrl);

      results.success++;

      // Add delay between downloads to avoid overwhelming the browser
      await new Promise((resolve) => setTimeout(resolve, 500));
    } catch (error) {
      results.failed++;
      results.errors.push(
        error instanceof Error ? error.message : `Version ${version} failed`,
      );
    }
  }

  return results;
}

// Utility function to verify document before download
export async function verifyAndDownload(
  documentId: string,
  token: string,
): Promise<{ verified: boolean; downloaded: boolean; error?: string }> {
  try {
    // First verify the document
    const verifyResponse = await fetch(
      `/api/will/documents/${documentId}/verify`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      },
    );

    if (!verifyResponse.ok) {
      throw new Error("Verification failed");
    }

    const verifyData = await verifyResponse.json();
    if (!verifyData.data.is_valid) {
      return {
        verified: false,
        downloaded: false,
        error: "Document integrity check failed",
      };
    }

    // If verified, proceed with download
    const downloadResponse = await fetch(
      `/api/will/documents/${documentId}/download`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      },
    );

    if (!downloadResponse.ok) {
      throw new Error("Download failed");
    }

    const blob = await downloadResponse.blob();
    const downloadUrl = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = downloadUrl;
    a.download = `will_${documentId}.pdf`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    window.URL.revokeObjectURL(downloadUrl);

    return { verified: true, downloaded: true };
  } catch (error) {
    return {
      verified: false,
      downloaded: false,
      error: error instanceof Error ? error.message : "Operation failed",
    };
  }
}
