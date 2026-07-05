"use client";

import { useState, useCallback, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  X,
  Upload,
  Check,
  Clock,
  AlertCircle,
  CheckCircle,
  XCircle,
  Loader2,
  FileText,
  Trash2,
} from "lucide-react";
import { useKYC } from "@/context/KYCContext";
import { PersonalInfoStep } from "./steps/PersonalInfoStep";
import { IdentityStep } from "./steps/IdentityStep";
import { AddressStep } from "./steps/AddressStep";
import { DocumentsStep } from "./steps/DocumentsStep";
import { ReviewStep } from "./steps/ReviewStep";
import { VerificationApprovedView } from "./VerificationApprovedView";
import { VerificationRejectedView } from "./VerificationRejectedView";
import { VerificationPendingView } from "./VerificationPendingView";

type FormStep = "personal" | "identity" | "address" | "documents" | "review";

export function KYCVerificationModal() {
  const {
    isKYCModalOpen,
    closeKYCModal,
    kycStatus,
    kycResponse,
    formData,
    updateFormData,
    submitKYC,
    uploadDocument,
    uploadedDocuments,
    isSubmitting,
    isLoading,
    error,
  } = useKYC();

  const [currentStep, setCurrentStep] = useState<FormStep>("personal");
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [isUploadingDoc, setIsUploadingDoc] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileUpload = useCallback(
    async (file: File, type: "id" | "address_proof" | "selfie") => {
      if (!file) return;

      setUploadError(null);
      setIsUploadingDoc(true);

      try {
        // Validate file size (5MB max)
        if (file.size > 5 * 1024 * 1024) {
          throw new Error("File must be smaller than 5MB");
        }

        // Validate file type
        const allowedTypes = ["image/jpeg", "image/png", "application/pdf"];
        if (!allowedTypes.includes(file.type)) {
          throw new Error("Only JPG, PNG, or PDF files are allowed");
        }

        await uploadDocument(file, type);
      } catch (err) {
        const msg = err instanceof Error ? err.message : "Upload failed";
        setUploadError(msg);
      } finally {
        setIsUploadingDoc(false);
      }
    },
    [uploadDocument]
  );

  const handleStepChange = (step: FormStep) => {
    setCurrentStep(step);
    setUploadError(null);
  };

  if (!isKYCModalOpen) return null;

  const idDoc = uploadedDocuments.find((d) => d.type === "id");
  const canProceed: Record<FormStep, boolean> = {
    personal: !!(
      formData.fullName && formData.email && formData.dateOfBirth && formData.nationality
    ),
    identity: !!(formData.idType && formData.idNumber && formData.expiryDate && idDoc),
    address: !!(
      formData.streetAddress && formData.city && formData.country && formData.postalCode
    ),
    documents: uploadedDocuments.length >= 1,
    review: true,
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-end md:items-center justify-center"
      role="dialog"
      aria-modal="true"
      aria-label="KYC Verification"
    >
      {/* Backdrop */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={closeKYCModal}
      />

      {/* Modal */}
      <motion.div
        initial={{ opacity: 0, y: 40 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 40 }}
        className="relative w-full max-w-2xl max-h-[90dvh] overflow-hidden mx-4 bg-[#161E22] border border-[#2A3338] rounded-2xl shadow-2xl"
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 bg-[#161E22] border-b border-[#2A3338]">
          <div>
            <h2 className="text-lg font-semibold text-white">KYC Verification</h2>
            <p className="text-xs text-[#92A5A8] mt-0.5">
              Complete identity verification to create and manage plans
            </p>
          </div>
          <button
            onClick={closeKYCModal}
            className="p-2 rounded-lg text-[#92A5A8] hover:text-white hover:bg-[#1C252A] transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Status Tracker */}
        {kycResponse && (
          <div className="px-6 py-3 bg-black/20 border-b border-[#2A3338]">
            <div className="flex items-center gap-2">
              {kycStatus === "pending" && (
                <>
                  <Clock size={16} className="text-[#92A5A8]" />
                  <span className="text-xs text-[#92A5A8]">
                    Status: <span className="font-semibold">Pending</span>
                  </span>
                </>
              )}
              {kycStatus === "submitted" && (
                <>
                  <Loader2 size={16} className="animate-spin text-[#33C5E0]" />
                  <span className="text-xs text-[#33C5E0]">
                    Status: <span className="font-semibold">Under Review</span>
                  </span>
                </>
              )}
              {kycStatus === "approved" && (
                <>
                  <CheckCircle size={16} className="text-[#48BB78]" />
                  <span className="text-xs text-[#48BB78]">
                    Status: <span className="font-semibold">Approved</span>
                  </span>
                </>
              )}
              {kycStatus === "rejected" && (
                <>
                  <XCircle size={16} className="text-[#F56565]" />
                  <span className="text-xs text-[#F56565]">
                    Status: <span className="font-semibold">Rejected</span>
                  </span>
                </>
              )}
            </div>
            {kycResponse.rejection_reason && (
              <p className="text-xs text-[#F56565] mt-2">
                Reason: {kycResponse.rejection_reason}
              </p>
            )}
          </div>
        )}

        {/* Content */}
        <div className="overflow-y-auto max-h-[calc(90dvh-140px)]">
          {kycStatus === "approved" ? (
            <VerificationApprovedView response={kycResponse!} />
          ) : kycStatus === "rejected" ? (
            <VerificationRejectedView response={kycResponse!} />
          ) : kycStatus === "submitted" ? (
            <VerificationPendingView response={kycResponse!} />
          ) : (
            <VerificationFormView
              currentStep={currentStep}
              onStepChange={handleStepChange}
              canProceed={canProceed}
              onFileUpload={handleFileUpload}
              isUploadingDoc={isUploadingDoc}
              uploadError={uploadError}
              isSubmitting={isSubmitting}
              error={error}
            />
          )}
        </div>

        {/* Footer */}
        {kycStatus === "pending" && (
          <div className="px-6 py-4 bg-[#161E22] border-t border-[#2A3338] flex gap-3">
            <button
              onClick={closeKYCModal}
              disabled={isSubmitting}
              className="flex-1 px-4 py-2 text-sm text-[#92A5A8] hover:text-white bg-[#1C252A] hover:bg-[#2A3338] rounded-lg transition-colors disabled:opacity-40"
            >
              Close
            </button>
            <button
              onClick={submitKYC}
              disabled={
                !canProceed.review ||
                isSubmitting ||
                !canProceed.personal ||
                !canProceed.identity ||
                !canProceed.address ||
                !canProceed.documents
              }
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSubmitting ? (
                <>
                  <Loader2 size={14} className="animate-spin" />
                  Submitting…
                </>
              ) : (
                <>
                  <Check size={14} />
                  Submit KYC
                </>
              )}
            </button>
          </div>
        )}
      </motion.div>
    </div>
  );
}

// ─── Form View ────────────────────────────────────────────────────────────────

interface VerificationFormViewProps {
  currentStep: FormStep;
  onStepChange: (step: FormStep) => void;
  canProceed: Record<FormStep, boolean>;
  onFileUpload: (file: File, type: "id" | "address_proof" | "selfie") => void;
  isUploadingDoc: boolean;
  uploadError: string | null;
  isSubmitting: boolean;
  error: string | null;
}

function VerificationFormView({
  currentStep,
  onStepChange,
  canProceed,
  onFileUpload,
  isUploadingDoc,
  uploadError,
  error,
}: VerificationFormViewProps) {
  return (
    <div className="p-6 space-y-6">
      {/* Step Indicator */}
      <div className="flex gap-2">
        {(["personal", "identity", "address", "documents", "review"] as const).map(
          (step, idx) => (
            <button
              key={step}
              onClick={() => onStepChange(step)}
              disabled={!canProceed[step] && step !== "personal"}
              className={`flex-1 h-1 rounded-full transition-all ${
                currentStep === step
                  ? "bg-[#33C5E0]"
                  : canProceed[step]
                    ? "bg-[#48BB78]"
                    : "bg-[#2A3338]"
              }`}
              title={step}
            />
          )
        )}
      </div>

      {/* Error Message */}
      {(error || uploadError) && (
        <motion.div
          initial={{ opacity: 0, y: -4 }}
          animate={{ opacity: 1, y: 0 }}
          className="flex items-start gap-2 p-3 rounded-lg bg-[#F5656514] border border-[#F5656540] text-[#F56565] text-xs"
        >
          <AlertCircle size={14} className="mt-0.5 flex-shrink-0" />
          <span>{error || uploadError}</span>
        </motion.div>
      )}

      {/* Step Content */}
      <AnimatePresence mode="wait">
        {currentStep === "personal" && (
          <PersonalInfoStep key="personal" />
        )}
        {currentStep === "identity" && (
          <IdentityStep
            key="identity"
            onFileUpload={onFileUpload}
            isUploadingDoc={isUploadingDoc}
          />
        )}
        {currentStep === "address" && (
          <AddressStep key="address" />
        )}
        {currentStep === "documents" && (
          <DocumentsStep
            key="documents"
            onFileUpload={onFileUpload}
            isUploadingDoc={isUploadingDoc}
          />
        )}
        {currentStep === "review" && (
          <ReviewStep key="review" />
        )}
      </AnimatePresence>

      {/* Navigation */}
      <div className="flex gap-3">
        <button
          onClick={() => {
            const steps: FormStep[] = ["personal", "identity", "address", "documents", "review"];
            const idx = steps.indexOf(currentStep);
            if (idx > 0) onStepChange(steps[idx - 1]);
          }}
          disabled={currentStep === "personal"}
          className="flex-1 px-4 py-2 text-sm text-[#92A5A8] hover:text-white bg-[#1C252A] hover:bg-[#2A3338] rounded-lg transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          Previous
        </button>
        <button
          onClick={() => {
            const steps: FormStep[] = ["personal", "identity", "address", "documents", "review"];
            const idx = steps.indexOf(currentStep);
            if (idx < steps.length - 1) onStepChange(steps[idx + 1]);
          }}
          disabled={!canProceed[currentStep] || currentStep === "review"}
          className="flex-1 px-4 py-2 text-sm text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Next
        </button>
      </div>
    </div>
  );
}

export default KYCVerificationModal;
