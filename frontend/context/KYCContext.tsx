"use client";

import React, { createContext, useContext, useState, useEffect, useCallback } from "react";
import { kycAPI, type KYCStatus, type KYCResponse } from "@/app/lib/api/kyc";

export type { KYCStatus };

interface KYCFormData {
  // Personal Information
  fullName: string;
  email: string;
  dateOfBirth: string;
  nationality: string;
  // Identity Document
  idType: string;
  idNumber: string;
  expiryDate: string;
  idDocument: File | null;
  // Address
  streetAddress: string;
  city: string;
  country: string;
  postalCode: string;
}

interface UploadedDocument {
  id: string;
  type: "id" | "address_proof" | "selfie";
  name: string;
  uploadedAt: string;
}

interface KYCContextType {
  isKYCModalOpen: boolean;
  kycStatus: KYCStatus;
  kycResponse: KYCResponse | null;
  formData: KYCFormData;
  uploadedDocuments: UploadedDocument[];
  openKYCModal: () => void;
  closeKYCModal: () => void;
  updateFormData: (data: Partial<KYCFormData>) => void;
  submitKYC: () => Promise<void>;
  uploadDocument: (file: File, type: "id" | "address_proof" | "selfie") => Promise<void>;
  isSubmitting: boolean;
  isLoading: boolean;
  error: string | null;
  refreshKYCStatus: () => Promise<void>;
  canCreatePlan: boolean;
}

const initialFormData: KYCFormData = {
  fullName: "",
  email: "",
  dateOfBirth: "",
  nationality: "",
  idType: "international_passport",
  idNumber: "",
  expiryDate: "",
  idDocument: null,
  streetAddress: "",
  city: "",
  country: "",
  postalCode: "",
};

const KYCContext = createContext<KYCContextType | undefined>(undefined);

export const useKYC = () => {
  const context = useContext(KYCContext);
  if (!context) {
    throw new Error("useKYC must be used within a KYCProvider");
  }
  return context;
};

export const KYCProvider = ({ children }: { children: React.ReactNode }) => {
  const [isKYCModalOpen, setIsKYCModalOpen] = useState(false);
  const [kycStatus, setKycStatus] = useState<KYCStatus>("pending");
  const [kycResponse, setKycResponse] = useState<KYCResponse | null>(null);
  const [formData, setFormData] = useState<KYCFormData>(initialFormData);
  const [uploadedDocuments, setUploadedDocuments] = useState<UploadedDocument[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Load persisted KYC status on mount and poll for updates
  useEffect(() => {
    const loadKYCStatus = async () => {
      try {
        setIsLoading(true);
        const response = await kycAPI.getKYCStatus();
        setKycResponse(response);
        setKycStatus(response.kyc_status);
        setError(null);
      } catch (err) {
        console.error("Failed to load KYC status:", err);
        setError(err instanceof Error ? err.message : "Failed to load KYC status");
        // Keep using local status on error
      } finally {
        setIsLoading(false);
      }
    };

    loadKYCStatus();

    // Poll for status updates every 10 seconds when modal is open or status is pending
    const pollInterval = setInterval(async () => {
      if (kycStatus === "pending" || kycStatus === "submitted") {
        try {
          const response = await kycAPI.getKYCStatus();
          setKycResponse(response);
          setKycStatus(response.kyc_status);
        } catch (err) {
          console.error("Failed to poll KYC status:", err);
        }
      }
    }, 10000);

    return () => clearInterval(pollInterval);
  }, [kycStatus]);

  const openKYCModal = () => setIsKYCModalOpen(true);
  const closeKYCModal = () => setIsKYCModalOpen(false);

  const updateFormData = (data: Partial<KYCFormData>) => {
    setFormData((prev) => ({ ...prev, ...data }));
  };

  const uploadDocument = async (
    file: File,
    type: "id" | "address_proof" | "selfie"
  ) => {
    try {
      setError(null);
      const result = await kycAPI.uploadDocument(file, type);
      const newDocument: UploadedDocument = {
        id: result.document_id,
        type,
        name: file.name,
        uploadedAt: new Date().toISOString(),
      };
      setUploadedDocuments((prev) => [...prev, newDocument]);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Failed to upload document";
      setError(errorMsg);
      throw err;
    }
  };

  const submitKYC = async () => {
    setIsSubmitting(true);
    setError(null);
    try {
      const idDocId = uploadedDocuments.find((d) => d.type === "id")?.id;

      const response = await kycAPI.submitKYC({
        fullName: formData.fullName,
        email: formData.email,
        dateOfBirth: formData.dateOfBirth,
        nationality: formData.nationality,
        idType: formData.idType,
        idNumber: formData.idNumber,
        expiryDate: formData.expiryDate,
        streetAddress: formData.streetAddress,
        city: formData.city,
        country: formData.country,
        postalCode: formData.postalCode,
        documentId: idDocId,
      });

      setKycResponse(response);
      setKycStatus(response.kyc_status);

      // Close modal after submission
      setTimeout(() => {
        closeKYCModal();
      }, 1500);

      // Reset form data
      setFormData(initialFormData);
      setUploadedDocuments([]);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "KYC submission failed";
      setError(errorMsg);
      console.error("KYC submission failed:", err);
      throw err;
    } finally {
      setIsSubmitting(false);
    }
  };

  const refreshKYCStatus = useCallback(async () => {
    try {
      setIsLoading(true);
      const response = await kycAPI.getKYCStatus();
      setKycResponse(response);
      setKycStatus(response.kyc_status);
      setError(null);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Failed to refresh KYC status";
      setError(errorMsg);
      console.error("Failed to refresh KYC status:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const canCreatePlan = kycStatus === "approved";

  return (
    <KYCContext.Provider
      value={{
        isKYCModalOpen,
        kycStatus,
        kycResponse,
        formData,
        uploadedDocuments,
        openKYCModal,
        closeKYCModal,
        updateFormData,
        submitKYC,
        uploadDocument,
        isSubmitting,
        isLoading,
        error,
        refreshKYCStatus,
        canCreatePlan,
      }}
    >
      {children}
    </KYCContext.Provider>
  );
};

