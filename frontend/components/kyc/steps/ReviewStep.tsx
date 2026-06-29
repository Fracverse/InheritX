"use client";

import { motion } from "framer-motion";
import { CheckCircle, AlertCircle } from "lucide-react";
import { useKYC } from "@/context/KYCContext";

export function ReviewStep() {
  const { formData, uploadedDocuments } = useKYC();

  const checklistItems = [
    {
      label: "Personal information",
      completed: formData.fullName && formData.email && formData.dateOfBirth,
    },
    {
      label: "Identity document",
      completed:
        formData.idType && formData.idNumber && formData.expiryDate && uploadedDocuments.some((d) => d.type === "id"),
    },
    {
      label: "Residential address",
      completed:
        formData.streetAddress &&
        formData.city &&
        formData.country &&
        formData.postalCode,
    },
    {
      label: "Document upload",
      completed: uploadedDocuments.length > 0,
    },
  ];

  const allCompleted = checklistItems.every((item) => item.completed);

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      className="space-y-4"
    >
      <h3 className="text-sm font-semibold text-[#33C5E0] uppercase tracking-wider">
        Review & Submit
      </h3>

      <div className="space-y-3">
        {checklistItems.map((item, idx) => (
          <motion.div
            key={idx}
            initial={{ opacity: 0, x: -8 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: idx * 0.05 }}
            className="flex items-center gap-3 p-3 rounded-lg bg-black/20 border border-[#2A3338]"
          >
            {item.completed ? (
              <CheckCircle size={16} className="text-[#48BB78] flex-shrink-0" />
            ) : (
              <AlertCircle size={16} className="text-[#F56565] flex-shrink-0" />
            )}
            <span className={`text-sm ${item.completed ? "text-[#48BB78]" : "text-[#F56565]"}`}>
              {item.label}
            </span>
          </motion.div>
        ))}
      </div>

      {allCompleted && (
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          className="p-4 rounded-lg bg-[#48BB7814] border border-[#48BB7840] space-y-2"
        >
          <div className="flex items-center gap-2">
            <CheckCircle size={18} className="text-[#48BB78]" />
            <p className="text-sm font-semibold text-[#48BB78]">
              Ready to submit
            </p>
          </div>
          <p className="text-xs text-[#92A5A8]">
            All required information has been provided. Your KYC application will be
            reviewed and you'll be notified of the status via email.
          </p>
        </motion.div>
      )}

      {!allCompleted && (
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          className="p-4 rounded-lg bg-[#ED893614] border border-[#ED893640] space-y-2"
        >
          <div className="flex items-center gap-2">
            <AlertCircle size={18} className="text-[#ED8936]" />
            <p className="text-sm font-semibold text-[#ED8936]">
              Incomplete information
            </p>
          </div>
          <p className="text-xs text-[#92A5A8]">
            Please complete all required fields before submitting.
          </p>
        </motion.div>
      )}

      {/* Summary */}
      <div className="pt-4 space-y-2 text-xs text-[#92A5A8]">
        <p>
          <span className="font-semibold text-slate-200">Name:</span> {formData.fullName || "—"}
        </p>
        <p>
          <span className="font-semibold text-slate-200">Email:</span> {formData.email || "—"}
        </p>
        <p>
          <span className="font-semibold text-slate-200">Documents uploaded:</span>{" "}
          {uploadedDocuments.length}
        </p>
      </div>
    </motion.div>
  );
}
