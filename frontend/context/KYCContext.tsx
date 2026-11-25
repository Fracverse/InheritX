"use client";

import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from "react";
import { kycAPI, type KYCStatus, type KYCResponse } from "@/app/lib/api/kyc";
import { useWallet } from "./WalletContext";
import { getKYCStatus } from "@/lib/api/dataSource";
import { useKYCWebSocket } from "@/hooks/useKYCWebSocket";

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
  resetForResubmission: () => void;
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
  const { address } = useWallet();
  const [isKYCModalOpen, setIsKYCModalOpen] = useState(false);
  const [kycStatus, setKycStatus] = useState<KYCStatus>("pending");
  const [kycResponse, setKycResponse] = useState<KYCResponse | null>(null);
  const [formData, setFormData] = useState<KYCFormData>(initialFormData);
  const [uploadedDocuments, setUploadedDocuments] = useState<UploadedDocument[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const requestCounterRef = useRef(0);
  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Handle WebSocket status updates
  const handleWebSocketUpdate = useCallback((status: KYCStatus) => {
    setKycStatus(status);
    setKycResponse((prev) => prev ? { ...prev, kyc_status: status } : null);

    // Stop polling for terminal states
    if ((status === "approved" || status === "rejected") && pollIntervalRef.current) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
  }, []);

  // Connect to WebSocket
  const { isHealthy: wsHealthy } = useKYCWebSocket({
    walletAddress: address,
    onStatusUpdate: handleWebSocketUpdate,
    enabled: !!address,
  });

  // Load status for the connected wallet and revalidate it while the app is open.
  useEffect(() => {
    const loadKYCStatus = async () => {
      try {
        setIsLoading(true);
        if (!address) {
          setKycResponse(null);
          setKycStatus("pending");
          return;
        }

        // Track request order to prevent stale responses
        const requestId = ++requestCounterRef.current;

        const response = await getKYCStatus(address);

        // Discard stale responses
        if (requestId !== requestCounterRef.current) {
          return;
        }

        setKycResponse(response);
        setKycStatus(response.kyc_status);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load KYC status.");
      } finally {
        setIsLoading(false);
      }
    };

    // Initial load
    loadKYCStatus();

    // Stop polling for terminal states
    if (kycStatus === "approved" || kycStatus === "rejected") {
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
        pollIntervalRef.current = null;
      }
      return;
    }

    // Start polling (fallback when WebSocket is disconnected)
    const startPolling = () => {
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
      }

      pollIntervalRef.current = setInterval(() => {
        // Skip polling when WebSocket is healthy
        if (wsHealthy) {
          return;
        }
        loadKYCStatus();
      }, 5000);
    };

    startPolling();

    return () => {
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
        pollIntervalRef.current = null;
      }
    };
  }, [address, kycStatus, wsHealthy]);

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
      const mockDocId = `doc_${Math.random().toString(36).substr(2, 9)}`;
      const newDocument: UploadedDocument = {
        id: mockDocId,
        type,
        name: file.name,
        uploadedAt: new Date().toISOString(),
      };
      setUploadedDocuments((prev) => [...prev, newDocument]);
    } catch (err) {
      setError("Failed to upload document");
      throw err;
    }
  };

  const submitKYC = async () => {
    setIsSubmitting(true);
    setError(null);
    try {
      await kycAPI.submitKYC({
        wallet_address: address ?? "",
        full_name: formData.fullName,
        email: formData.email,
        date_of_birth: formData.dateOfBirth,
        nationality: formData.nationality,
        id_type: formData.idType,
        id_number: formData.idNumber,
        expiry_date: formData.expiryDate,
        street_address: formData.streetAddress,
        city: formData.city,
        country: formData.country,
        postal_code: formData.postalCode,
      });
      await refreshKYCStatus();

      setTimeout(() => {
        closeKYCModal();
      }, 1500);

      setFormData(initialFormData);
      setUploadedDocuments([]);
    } catch (err) {
      setError("Failed to submit KYC");
    } finally {
      setIsSubmitting(false);
    }
  };

  const refreshKYCStatus = useCallback(async () => {
    try {
      setIsLoading(true);

      // Track request order to prevent stale responses
      const requestId = ++requestCounterRef.current;

      const response = await getKYCStatus(address ?? undefined);

      // Discard stale responses
      if (requestId !== requestCounterRef.current) {
        return;
      }

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
  }, [address]);

  const resetForResubmission = useCallback(() => {
    // Reset form data to initial state
    setFormData(initialFormData);

    // Clear uploaded documents
    setUploadedDocuments([]);

    // Set status to pending to allow new submission
    setKycStatus("pending");
    setKycResponse((prev) => prev ? { ...prev, kyc_status: "pending" } : null);

    // Clear any errors
    setError(null);

    // Open the KYC modal to restart the flow
    setIsKYCModalOpen(true);

    // Refresh status from backend
    refreshKYCStatus();
  }, [refreshKYCStatus]);

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
        resetForResubmission,
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

