"use client";

import React, { createContext, useContext, useState, useEffect } from "react";

export type KYCStatus = "pending" | "submitted" | "approved" | "rejected";

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

interface KYCContextType {
  isKYCModalOpen: boolean;
  kycStatus: KYCStatus;
  formData: KYCFormData;
  openKYCModal: () => void;
  closeKYCModal: () => void;
  updateFormData: (data: Partial<KYCFormData>) => void;
  submitKYC: () => Promise<void>;
  isSubmitting: boolean;
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
  const [formData, setFormData] = useState<KYCFormData>(initialFormData);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Load persisted KYC status on mount
  useEffect(() => {
    const savedStatus = localStorage.getItem("inheritx_kyc_status");
    if (savedStatus) {
      setKycStatus(savedStatus as KYCStatus);
    }
  }, []);

  const openKYCModal = () => setIsKYCModalOpen(true);
  const closeKYCModal = () => setIsKYCModalOpen(false);

  const updateFormData = (data: Partial<KYCFormData>) => {
    setFormData((prev) => ({ ...prev, ...data }));
  };

  const submitKYC = async () => {
    setIsSubmitting(true);
    try {
      // Simulate API call
      await new Promise((resolve) => setTimeout(resolve, 1500));

      // Update status to submitted
      setKycStatus("submitted");
      localStorage.setItem("inheritx_kyc_status", "submitted");

      // Close modal after submission
      closeKYCModal();

      // Reset form data
      setFormData(initialFormData);
    } catch (error) {
      console.error("KYC submission failed:", error);
      throw error;
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <KYCContext.Provider
      value={{
        isKYCModalOpen,
        kycStatus,
        formData,
        openKYCModal,
        closeKYCModal,
        updateFormData,
        submitKYC,
        isSubmitting,
      }}
    >
      {children}
    </KYCContext.Provider>
  );
};
