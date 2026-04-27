/**
 * Will Documents API Client
 *
 * Provides type-safe methods for interacting with the will documents API
 */

export interface WillDocument {
  document_id: string;
  plan_id: string;
  template_used: string;
  will_hash: string;
  generated_at: string;
  version: number;
  filename: string;
  pdf_base64?: string;
}

export interface WillEvent {
  event_type: string;
  document_id: string;
  plan_id: string;
  vault_id: string;
  timestamp: string;
  [key: string]: unknown;
}

export interface EventStats {
  plan_id: string;
  will_created_count: number;
  will_updated_count: number;
  will_finalized_count: number;
  will_signed_count: number;
  witness_signed_count: number;
  will_verified_count: number;
  total_events: number;
  first_event_at: string | null;
  last_event_at: string | null;
}

export interface VerificationResult {
  is_valid: boolean;
  document_id: string;
  version?: number;
  hash_match: boolean;
  message: string;
}

export class WillDocumentsAPI {
  private baseUrl: string;
  private getAuthToken: () => string | null;

  constructor(baseUrl: string = "", getAuthToken: () => string | null) {
    this.baseUrl = baseUrl;
    this.getAuthToken = getAuthToken;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
  ): Promise<T> {
    const token = this.getAuthToken();
    if (!token) {
      throw new Error("Authentication required");
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error || `Request failed with status ${response.status}`,
      );
    }

    return response.json();
  }

  /**
   * List all will documents for a plan
   */
  async listDocuments(planId: string): Promise<WillDocument[]> {
    const response = await this.request<{
      status: string;
      data: WillDocument[];
    }>(`/api/plans/${planId}/will/documents`);
    return response.data;
  }

  /**
   * Get a specific will document
   */
  async getDocument(documentId: string): Promise<WillDocument> {
    const response = await this.request<{ status: string; data: WillDocument }>(
      `/api/will/documents/${documentId}`,
    );
    return response.data;
  }

  /**
   * Download a will document by ID
   */
  async downloadDocument(documentId: string, filename?: string): Promise<void> {
    const token = this.getAuthToken();
    if (!token) {
      throw new Error("Authentication required");
    }

    const response = await fetch(
      `${this.baseUrl}/api/will/documents/${documentId}/download`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      },
    );

    if (!response.ok) {
      throw new Error(`Download failed with status ${response.status}`);
    }

    // Extract filename from Content-Disposition header
    const contentDisposition = response.headers.get("Content-Disposition");
    let downloadFilename = filename || "will_document.pdf";
    if (contentDisposition) {
      const filenameMatch = contentDisposition.match(/filename="(.+)"/);
      if (filenameMatch) {
        downloadFilename = filenameMatch[1];
      }
    }

    // Download the file
    const blob = await response.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = downloadFilename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    window.URL.revokeObjectURL(url);
  }

  /**
   * Download a specific version of a will document
   */
  async downloadVersion(
    planId: string,
    version: number,
    filename?: string,
  ): Promise<void> {
    const token = this.getAuthToken();
    if (!token) {
      throw new Error("Authentication required");
    }

    const response = await fetch(
      `${this.baseUrl}/api/plans/${planId}/will/documents/${version}/download`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      },
    );

    if (!response.ok) {
      throw new Error(`Download failed with status ${response.status}`);
    }

    // Extract filename from Content-Disposition header
    const contentDisposition = response.headers.get("Content-Disposition");
    let downloadFilename = filename || `will_v${version}.pdf`;
    if (contentDisposition) {
      const filenameMatch = contentDisposition.match(/filename="(.+)"/);
      if (filenameMatch) {
        downloadFilename = filenameMatch[1];
      }
    }

    // Download the file
    const blob = await response.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = downloadFilename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    window.URL.revokeObjectURL(url);
  }

  /**
   * Verify document integrity
   */
  async verifyDocument(
    documentId: string,
    version?: number,
  ): Promise<VerificationResult> {
    const queryParams = version ? `?version=${version}` : "";
    const response = await this.request<{
      status: string;
      data: VerificationResult;
    }>(`/api/will/documents/${documentId}/verify${queryParams}`);
    return response.data;
  }

  /**
   * Get all events for a document
   */
  async getDocumentEvents(documentId: string): Promise<WillEvent[]> {
    const response = await this.request<{ status: string; data: WillEvent[] }>(
      `/api/will/documents/${documentId}/events`,
    );
    return response.data;
  }

  /**
   * Get all events for a plan
   */
  async getPlanEvents(planId: string): Promise<WillEvent[]> {
    const response = await this.request<{ status: string; data: WillEvent[] }>(
      `/api/plans/${planId}/will/events`,
    );
    return response.data;
  }

  /**
   * Get event statistics for a plan
   */
  async getPlanEventStats(planId: string): Promise<EventStats> {
    const response = await this.request<{ status: string; data: EventStats }>(
      `/api/plans/${planId}/will/events/stats`,
    );
    return response.data;
  }

  /**
   * Generate a new will document
   */
  async generateDocument(
    planId: string,
    data: {
      owner_name: string;
      owner_wallet: string;
      vault_id: string;
      beneficiaries: Array<{
        name: string;
        wallet_address: string;
        allocation_percent: string;
        relationship?: string;
      }>;
      execution_rules?: string;
      template?: string;
      jurisdiction?: string;
      will_hash_reference?: string;
    },
  ): Promise<WillDocument> {
    const response = await this.request<{ status: string; data: WillDocument }>(
      `/api/plans/${planId}/will/generate`,
      {
        method: "POST",
        body: JSON.stringify(data),
      },
    );
    return response.data;
  }

  /**
   * Download all versions of a plan's will documents
   */
  async downloadAllVersions(planId: string): Promise<{
    success: number;
    failed: number;
    errors: string[];
  }> {
    const documents = await this.listDocuments(planId);
    const results = {
      success: 0,
      failed: 0,
      errors: [] as string[],
    };

    for (const doc of documents) {
      try {
        await this.downloadVersion(planId, doc.version);
        results.success++;
        // Add delay between downloads
        await new Promise((resolve) => setTimeout(resolve, 500));
      } catch (error) {
        results.failed++;
        results.errors.push(
          error instanceof Error
            ? error.message
            : `Version ${doc.version} failed`,
        );
      }
    }

    return results;
  }

  /**
   * Verify and download a document (ensures integrity before download)
   */
  async verifyAndDownload(
    documentId: string,
    filename?: string,
  ): Promise<{ verified: boolean; downloaded: boolean; error?: string }> {
    try {
      // First verify
      const verification = await this.verifyDocument(documentId);
      if (!verification.is_valid) {
        return {
          verified: false,
          downloaded: false,
          error: "Document integrity check failed",
        };
      }

      // Then download
      await this.downloadDocument(documentId, filename);
      return { verified: true, downloaded: true };
    } catch (error) {
      return {
        verified: false,
        downloaded: false,
        error: error instanceof Error ? error.message : "Operation failed",
      };
    }
  }
}

/**
 * Create a singleton instance of the API client
 */
export function createWillDocumentsAPI(
  getAuthToken: () => string | null = () => localStorage.getItem("auth_token"),
): WillDocumentsAPI {
  return new WillDocumentsAPI("", getAuthToken);
}

/**
 * Default export for convenience
 */
export default createWillDocumentsAPI;
