/**
 * KYC/KYB API Service
 * Handles all KYC/KYB verification API calls
 */

import { apiClient, ApiResponse } from "./client";

export type KYCStatus = "pending" | "submitted" | "approved" | "rejected";

export interface KYCData {
  fullName: string;
  email: string;
  dateOfBirth: string;
  nationality: string;
  idType: string;
  idNumber: string;
  expiryDate: string;
  streetAddress: string;
  city: string;
  country: string;
  postalCode: string;
}

export interface KYCResponse {
  wallet_address: string;
  kyc_status: KYCStatus;
  submitted_at?: string;
  approved_at?: string;
  rejected_at?: string;
  rejection_reason?: string;
  provider_reference?: string;
}

export interface KYCSubmissionRequest {
  fullName: string;
  email: string;
  dateOfBirth: string;
  nationality: string;
  idType: string;
  idNumber: string;
  expiryDate: string;
  streetAddress: string;
  city: string;
  country: string;
  postalCode: string;
  documentId?: string; // Reference to uploaded document
}

export class KycAPI {
  /**
   * Get current user's KYC status
   */
  async getKYCStatus(): Promise<KYCResponse> {
    const response = await apiClient.get<ApiResponse<KYCResponse>>(
      "/api/kyc/status"
    );
    return response.data!;
  }

  /**
   * Submit KYC verification data
   */
  async submitKYC(data: KYCSubmissionRequest): Promise<KYCResponse> {
    const response = await apiClient.post<ApiResponse<KYCResponse>>(
      "/api/kyc/submit",
      data
    );
    return response.data!;
  }

  /**
   * Upload KYC document (ID, proof of address, etc.)
   */
  async uploadDocument(
    file: File,
    documentType: "id" | "address_proof" | "selfie"
  ): Promise<{ document_id: string; url: string }> {
    const formData = new FormData();
    formData.append("file", file);
    formData.append("document_type", documentType);

    const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080"}/api/kyc/upload`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${
          typeof window !== "undefined"
            ? localStorage.getItem("auth_token")
            : ""
        }`,
      },
      body: formData,
    });

    if (!response.ok) {
      throw new Error("Failed to upload document");
    }

    return response.json();
  }

  /**
   * Check if KYC is required before plan creation
   */
  async isKYCRequired(): Promise<{ required: boolean; reason?: string }> {
    const response = await apiClient.get<
      ApiResponse<{ required: boolean; reason?: string }>
    >("/api/kyc/required");
    return response.data!;
  }

  /**
   * Get KYC approval requirements
   */
  async getKYCRequirements(): Promise<{
    requires_id: boolean;
    requires_address_proof: boolean;
    requires_selfie: boolean;
    supported_id_types: string[];
    supported_countries: string[];
  }> {
    const response = await apiClient.get<
      ApiResponse<{
        requires_id: boolean;
        requires_address_proof: boolean;
        requires_selfie: boolean;
        supported_id_types: string[];
        supported_countries: string[];
      }>
    >("/api/kyc/requirements");
    return response.data!;
  }
}

export const kycAPI = new KycAPI();
export default kycAPI;
