"use client";

import { motion } from "framer-motion";
import { Upload, CheckCircle, FileText, Loader2, Trash2 } from "lucide-react";
import { useKYC } from "@/context/KYCContext";
import { useRef } from "react";

interface DocumentsStepProps {
  onFileUpload: (file: File, type: "address_proof" | "selfie") => void;
  isUploadingDoc: boolean;
}

export function DocumentsStep({ onFileUpload, isUploadingDoc }: DocumentsStepProps) {
  const { uploadedDocuments } = useKYC();
  const addressFileRef = useRef<HTMLInputElement>(null);
  const selfieFileRef = useRef<HTMLInputElement>(null);

  const addressDoc = uploadedDocuments.find((d) => d.type === "address_proof");
  const selfieDoc = uploadedDocuments.find((d) => d.type === "selfie");

  const handleFileSelect = (
    e: React.ChangeEvent<HTMLInputElement>,
    type: "address_proof" | "selfie"
  ) => {
    const file = e.target.files?.[0];
    if (file) {
      onFileUpload(file, type);
    }
    if (type === "address_proof" && addressFileRef.current) {
      addressFileRef.current.value = "";
    }
    if (type === "selfie" && selfieFileRef.current) {
      selfieFileRef.current.value = "";
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      className="space-y-4"
    >
      <h3 className="text-sm font-semibold text-[#33C5E0] uppercase tracking-wider">
        Supporting Documents
      </h3>

      <p className="text-xs text-[#92A5A8]">
        Upload additional documents to verify your identity. At least one document is
        required.
      </p>

      {/* Address Proof */}
      <div>
        <label className="text-xs text-[#92A5A8] block mb-2">
          Address Proof (Optional)
        </label>
        <div
          onClick={() => addressFileRef.current?.click()}
          className="relative border-2 border-dashed border-[#2A3338] rounded-lg p-6 text-center cursor-pointer hover:border-[#33C5E0] transition-colors bg-[#0A0F1114]"
        >
          <input
            ref={addressFileRef}
            type="file"
            onChange={(e) => handleFileSelect(e, "address_proof")}
            accept="image/jpeg,image/png,application/pdf"
            className="hidden"
            disabled={isUploadingDoc}
          />

          {isUploadingDoc ? (
            <>
              <Loader2 size={20} className="mx-auto mb-2 animate-spin text-[#33C5E0]" />
              <p className="text-xs text-[#92A5A8]">Uploading...</p>
            </>
          ) : addressDoc ? (
            <>
              <CheckCircle size={20} className="mx-auto mb-2 text-[#48BB78]" />
              <p className="text-xs text-[#48BB78] font-medium">{addressDoc.name}</p>
            </>
          ) : (
            <>
              <Upload size={20} className="mx-auto mb-2 text-[#92A5A8]" />
              <p className="text-xs text-slate-200">Utility bill, lease agreement, etc.</p>
            </>
          )}
        </div>
      </div>

      {/* Selfie */}
      <div>
        <label className="text-xs text-[#92A5A8] block mb-2">
          Selfie Verification (Optional)
        </label>
        <div
          onClick={() => selfieFileRef.current?.click()}
          className="relative border-2 border-dashed border-[#2A3338] rounded-lg p-6 text-center cursor-pointer hover:border-[#33C5E0] transition-colors bg-[#0A0F1114]"
        >
          <input
            ref={selfieFileRef}
            type="file"
            onChange={(e) => handleFileSelect(e, "selfie")}
            accept="image/jpeg,image/png"
            className="hidden"
            disabled={isUploadingDoc}
          />

          {isUploadingDoc ? (
            <>
              <Loader2 size={20} className="mx-auto mb-2 animate-spin text-[#33C5E0]" />
              <p className="text-xs text-[#92A5A8]">Uploading...</p>
            </>
          ) : selfieDoc ? (
            <>
              <CheckCircle size={20} className="mx-auto mb-2 text-[#48BB78]" />
              <p className="text-xs text-[#48BB78] font-medium">{selfieDoc.name}</p>
            </>
          ) : (
            <>
              <Upload size={20} className="mx-auto mb-2 text-[#92A5A8]" />
              <p className="text-xs text-slate-200">Recent selfie with ID document</p>
            </>
          )}
        </div>
      </div>

      <div className="pt-2 bg-black/20 rounded-lg p-3">
        <p className="text-xs text-[#92A5A8]">
          <span className="font-semibold text-[#33C5E0]">Pro tip:</span> Clear, well-lit
          photos will help speed up verification
        </p>
      </div>
    </motion.div>
  );
}
