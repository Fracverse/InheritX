"use client";

import { motion } from "framer-motion";
import { Upload, CheckCircle, FileText, Loader2 } from "lucide-react";
import { useKYC } from "@/context/KYCContext";
import { useRef } from "react";

interface IdentityStepProps {
  onFileUpload: (file: File, type: "id") => void;
  isUploadingDoc: boolean;
}

export function IdentityStep({ onFileUpload, isUploadingDoc }: IdentityStepProps) {
  const { formData, updateFormData, uploadedDocuments } = useKYC();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const idTypes = [
    "international_passport",
    "national_id",
    "drivers_license",
    "visa",
  ];

  const idDoc = uploadedDocuments.find((d) => d.type === "id");

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      onFileUpload(file, "id");
    }
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      className="space-y-4"
    >
      <h3 className="text-sm font-semibold text-[#33C5E0] uppercase tracking-wider">
        Identity Document
      </h3>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Document Type *</label>
          <select
            value={formData.idType}
            onChange={(e) => updateFormData({ idType: e.target.value })}
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
          >
            {idTypes.map((t) => (
              <option key={t} value={t}>
                {t.replace(/_/g, " ").charAt(0).toUpperCase() +
                  t.replace(/_/g, " ").slice(1)}
              </option>
            ))}
          </select>
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Document Number *</label>
          <input
            type="text"
            value={formData.idNumber}
            onChange={(e) => updateFormData({ idNumber: e.target.value })}
            placeholder="ABC123456"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors font-mono"
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Expiry Date *</label>
          <input
            type="date"
            value={formData.expiryDate}
            onChange={(e) => updateFormData({ expiryDate: e.target.value })}
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>
      </div>

      {/* Document Upload Zone */}
      <div className="pt-2">
        <label className="text-xs text-[#92A5A8] block mb-2">
          Upload ID Document *
        </label>
        <div
          onClick={() => fileInputRef.current?.click()}
          className="relative border-2 border-dashed border-[#2A3338] rounded-lg p-6 text-center cursor-pointer hover:border-[#33C5E0] transition-colors bg-[#0A0F1114]"
        >
          <input
            ref={fileInputRef}
            type="file"
            onChange={handleFileSelect}
            accept="image/jpeg,image/png,application/pdf"
            className="hidden"
            disabled={isUploadingDoc}
          />

          {isUploadingDoc ? (
            <>
              <Loader2 size={24} className="mx-auto mb-2 animate-spin text-[#33C5E0]" />
              <p className="text-xs text-[#92A5A8]">Uploading...</p>
            </>
          ) : idDoc ? (
            <>
              <CheckCircle size={24} className="mx-auto mb-2 text-[#48BB78]" />
              <p className="text-xs text-[#48BB78] font-medium">{idDoc.name}</p>
              <p className="text-[10px] text-[#92A5A8] mt-1">
                Uploaded successfully
              </p>
            </>
          ) : (
            <>
              <Upload size={24} className="mx-auto mb-2 text-[#92A5A8]" />
              <p className="text-xs text-slate-200 font-medium">
                Click to upload or drag and drop
              </p>
              <p className="text-[10px] text-[#92A5A8] mt-1">
                JPG, PNG, or PDF (max 5MB)
              </p>
            </>
          )}
        </div>
      </div>
    </motion.div>
  );
}
