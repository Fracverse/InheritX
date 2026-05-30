/**
 * Public Document Verification API Client
 *
 * Provides type-safe methods for verifying will documents
 * No authentication required - public verification interface
 */

export interface VerificationDetails {
  is_active_version: boolean;
  status: string;
  template: string;
  generated_at: string;
  is_encrypted: boolean;
}

export interface VerificationResult {
  is_valid: boolean;
  document_id: string;
  plan_id: string;
  version: number;
  stored_hash: string;
  computed_hash: string | null;
  verified_at: string;
  details: VerificationDetails;
}

export interface BatchVerificationResult {
  results: VerificationResult[];
  count: number;
  valid_count: number;
  invalid_count: number;
}

export class DocumentVerificationAPI {
  private baseUrl: string;

  constructor(baseUrl: string = "") {
    this.baseUrl = baseUrl;
  }

  /**
   * Compute SHA-256 hash of a file client-side
   */
  async computeFileHash(file: File): Promise<string> {
    const arrayBuffer = await file.arrayBuffer();
    const hashBuffer = await crypto.subtle.digest("SHA-256", arrayBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    return hashHex;
  }

  /**
   * Compute SHA-256 hash of base64 content
   */
  async computeBase64Hash(base64Content: string): Promise<string> {
    const encoder = new TextEncoder();
    const data = encoder.encode(base64Content);
    const hashBuffer = await crypto.subtle.digest("SHA-256", data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    return hashHex;
  }

  /**
   * Convert file to base64
   */
  async fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const base64 = (reader.result as string).split(",")[1];
        resolve(base64);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  /**
   * Verify document by ID (retrieves from database and compares hashes)
   */
  async verifyDocumentById(
    documentId: string,
    version?: number,
  ): Promise<VerificationResult> {
    const queryParams = version ? `?page=${version}` : "";
    const response = await fetch(
      `${this.baseUrl}/api/will/documents/${documentId}/verify${queryParams}`,
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error || `Verification failed with status ${response.status}`,
      );
    }

    const data = await response.json();
    return data.data;
  }

  /**
   * Verify document by hash
   */
  async verifyDocumentByHash(
    documentId: string,
    hash: string,
    version?: number,
  ): Promise<VerificationResult> {
    const response = await fetch(
      `${this.baseUrl}/api/will/documents/${documentId}/verify/hash`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ hash, version }),
      },
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error || `Hash verification failed with status ${response.status}`,
      );
    }

    const data = await response.json();
    return data.data;
  }

  /**
   * Verify document by content (uploads document and compares)
   */
  async verifyDocumentByContent(
    documentId: string,
    content: string,
    version?: number,
  ): Promise<VerificationResult> {
    const response = await fetch(
      `${this.baseUrl}/api/will/documents/${documentId}/verify/content`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ content, version }),
      },
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error ||
          `Content verification failed with status ${response.status}`,
      );
    }

    const data = await response.json();
    return data.data;
  }

  /**
   * Verify document by uploading file
   */
  async verifyDocumentByFile(
    documentId: string,
    file: File,
    version?: number,
  ): Promise<{
    result: VerificationResult;
    computedHash: string;
  }> {
    const base64Content = await this.fileToBase64(file);
    const computedHash = await this.computeBase64Hash(base64Content);
    const result = await this.verifyDocumentByContent(
      documentId,
      base64Content,
      version,
    );

    return { result, computedHash };
  }

  /**
   * Verify all versions of a plan's documents (requires authentication)
   */
  async verifyAllVersions(
    planId: string,
    authToken: string,
  ): Promise<BatchVerificationResult> {
    const response = await fetch(
      `${this.baseUrl}/api/plans/${planId}/will/verify-all`,
      {
        headers: {
          Authorization: `Bearer ${authToken}`,
        },
      },
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error ||
          `Batch verification failed with status ${response.status}`,
      );
    }

    const data = await response.json();
    const results = data.data as VerificationResult[];
    const validCount = results.filter((r) => r.is_valid).length;
    const invalidCount = results.length - validCount;

    return {
      results,
      count: data.count,
      valid_count: validCount,
      invalid_count: invalidCount,
    };
  }

  /**
   * Generate verification certificate data
   */
  generateCertificate(result: VerificationResult): {
    certificateData: string;
    qrCodeData: string;
  } {
    const certificateData = JSON.stringify(
      {
        document_id: result.document_id,
        version: result.version,
        is_valid: result.is_valid,
        stored_hash: result.stored_hash,
        verified_at: result.verified_at,
        status: result.details.status,
      },
      null,
      2,
    );

    // QR code data contains verification URL
    const qrCodeData = `${window.location.origin}/verification?document_id=${result.document_id}&version=${result.version}`;

    return { certificateData, qrCodeData };
  }
}

/**
 * Create a singleton instance of the verification API client
 */
export function createVerificationAPI(): DocumentVerificationAPI {
  return new DocumentVerificationAPI("");
}

/**
 * Default export for convenience
 */
export default createVerificationAPI;
