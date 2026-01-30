"use client";

import React, { useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  X,
  Upload,
  Check,
  Loader2,
  ChevronDown,
  Clock,
  CheckCircle,
  XCircle,
} from "lucide-react";
import { useKYC } from "@/context/KYCContext";

const ID_TYPES = [
  { value: "international_passport", label: "International Passport" },
  { value: "national_id", label: "National ID" },
  { value: "drivers_license", label: "Driver's License" },
];

// Status content component
function KYCStatusContent({
  status,
  onClose,
}: {
  status: string;
  onClose: () => void;
}) {
  const getStatusConfig = () => {
    switch (status) {
      case "submitted":
        return {
          icon: Clock,
          iconBg: "bg-[#ECC94B]/20",
          iconColor: "text-[#ECC94B]",
          title: "Application Under Review",
          message:
            "Your KYC application has been submitted successfully. Our team is reviewing your documents. This process typically takes 1-3 business days.",
          note: "You will be notified once your verification is complete.",
        };
      case "approved":
        return {
          icon: CheckCircle,
          iconBg: "bg-[#48BB78]/20",
          iconColor: "text-[#48BB78]",
          title: "Verification Complete",
          message:
            "Congratulations! Your identity has been verified successfully. You now have full access to create and manage inheritance plans.",
          note: null,
        };
      case "rejected":
        return {
          icon: XCircle,
          iconBg: "bg-[#F56565]/20",
          iconColor: "text-[#F56565]",
          title: "Verification Failed",
          message:
            "Unfortunately, we couldn't verify your identity with the provided documents. Please contact support for assistance.",
          note: "You may need to resubmit with clearer documents.",
        };
      default:
        return null;
    }
  };

  const config = getStatusConfig();
  if (!config) return null;

  const StatusIcon = config.icon;

  return (
    <div className="p-8 text-center">
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ type: "spring", duration: 0.5 }}
        className={`w-20 h-20 rounded-full ${config.iconBg} flex items-center justify-center mx-auto mb-6`}
      >
        <StatusIcon size={40} className={config.iconColor} />
      </motion.div>

      <motion.h3
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="text-2xl font-bold text-white mb-3"
      >
        {config.title}
      </motion.h3>

      <motion.p
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
        className="text-[#92A5A8] text-sm leading-relaxed mb-4 max-w-md mx-auto"
      >
        {config.message}
      </motion.p>

      {config.note && (
        <motion.p
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="text-[#6B7C7F] text-xs mb-6"
        >
          {config.note}
        </motion.p>
      )}

      <motion.button
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
        onClick={onClose}
        className="bg-[#1C252A] text-white px-8 py-3 rounded-lg font-medium hover:bg-[#2A3338] transition-colors"
      >
        Close
      </motion.button>
    </div>
  );
}

export function KYCVerificationModal() {
  const {
    isKYCModalOpen,
    closeKYCModal,
    formData,
    updateFormData,
    submitKYC,
    isSubmitting,
    kycStatus,
  } = useKYC();

  const fileInputRef = useRef<HTMLInputElement>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.fullName.trim()) newErrors.fullName = "Full name is required";
    if (!formData.email.trim()) newErrors.email = "Email is required";
    else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
      newErrors.email = "Invalid email format";
    }
    if (!formData.dateOfBirth)
      newErrors.dateOfBirth = "Date of birth is required";
    if (!formData.nationality.trim())
      newErrors.nationality = "Nationality is required";
    if (!formData.idNumber.trim()) newErrors.idNumber = "ID number is required";
    if (!formData.expiryDate) newErrors.expiryDate = "Expiry date is required";
    if (!formData.idDocument) newErrors.idDocument = "ID document is required";
    if (!formData.streetAddress.trim())
      newErrors.streetAddress = "Street address is required";
    if (!formData.city.trim()) newErrors.city = "City is required";
    if (!formData.country.trim()) newErrors.country = "Country is required";
    if (!formData.postalCode.trim())
      newErrors.postalCode = "Postal code is required";

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validateForm()) return;

    try {
      await submitKYC();
    } catch (error) {
      console.error("Submission error:", error);
    }
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      handleFileSelect(file);
    }
  };

  const handleFileSelect = (file: File) => {
    const validTypes = [
      "image/jpeg",
      "image/jpg",
      "image/png",
      "application/pdf",
    ];
    const maxSize = 5 * 1024 * 1024; // 5MB

    if (!validTypes.includes(file.type)) {
      setErrors((prev) => ({
        ...prev,
        idDocument: "Please upload JPG, JPEG, PNG, or PDF file",
      }));
      return;
    }

    if (file.size > maxSize) {
      setErrors((prev) => ({
        ...prev,
        idDocument: "File size must be less than 5MB",
      }));
      return;
    }

    updateFormData({ idDocument: file });
    setErrors((prev) => {
      const newErrors = { ...prev };
      delete newErrors.idDocument;
      return newErrors;
    });
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const file = e.dataTransfer.files?.[0];
    if (file) {
      handleFileSelect(file);
    }
  };

  const inputClasses =
    "w-full bg-transparent border border-[#2A3338] rounded-lg px-4 py-3 text-[#FCFFFF] placeholder-[#6B7C7F] focus:outline-none focus:border-[#33C5E0] transition-colors";

  const labelClasses = "block text-sm text-[#92A5A8] mb-2";

  if (!isKYCModalOpen) return null;

  // Show status view if KYC is not pending
  const showStatusView = kycStatus !== "pending";

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm"
        onClick={closeKYCModal}
      >
        <motion.div
          initial={{ scale: 0.95, opacity: 0, y: 20 }}
          animate={{ scale: 1, opacity: 1, y: 0 }}
          exit={{ scale: 0.95, opacity: 0, y: 20 }}
          transition={{ duration: 0.2, ease: "easeOut" }}
          className={`bg-[#0D1417] w-full ${showStatusView ? "max-w-md" : "max-w-2xl"} max-h-[90vh] overflow-hidden rounded-2xl`}
          onClick={(e) => e.stopPropagation()}
        >
          {showStatusView ? (
            // Status View
            <>
              <div className="flex justify-end p-4">
                <button
                  onClick={closeKYCModal}
                  className="p-2 text-[#6B7C7F] hover:text-white hover:bg-white/10 rounded-full transition-colors"
                >
                  <X size={20} />
                </button>
              </div>
              <KYCStatusContent status={kycStatus} onClose={closeKYCModal} />
            </>
          ) : (
            // Form View
            <>
              {/* Header */}
              <div className="p-6 pb-4">
                <div className="flex items-start justify-between">
                  <div>
                    <h2 className="text-2xl font-bold text-white">
                      KYC Verification
                    </h2>
                    <p className="text-[#6B7C7F] text-sm mt-1">
                      Complete your identity verification to create inheritance
                      plans.
                    </p>
                  </div>
                  <button
                    onClick={closeKYCModal}
                    className="p-2 text-[#6B7C7F] hover:text-white hover:bg-white/10 rounded-full transition-colors"
                  >
                    <X size={20} />
                  </button>
                </div>
              </div>

              {/* Form Content */}
              <form
                onSubmit={handleSubmit}
                className="overflow-y-auto max-h-[calc(90vh-180px)] px-6 no-scrollbar"
              >
                {/* Personal Information Section */}
                <section className="mb-6">
                  <div className="border border-[#1C252A] rounded-xl p-5">
                    <h3 className="text-lg font-semibold text-white mb-5">
                      Personal Information
                    </h3>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div>
                        <label className={labelClasses}>
                          Full Name <span className="text-[#33C5E0]">*</span>
                        </label>
                        <input
                          type="text"
                          value={formData.fullName}
                          onChange={(e) =>
                            updateFormData({ fullName: e.target.value })
                          }
                          className={`${inputClasses} ${errors.fullName ? "border-red-500" : ""}`}
                          placeholder="As shown on ID"
                        />
                        {errors.fullName && (
                          <p className="text-red-500 text-xs mt-1">
                            {errors.fullName}
                          </p>
                        )}
                      </div>

                      <div>
                        <label className={labelClasses}>
                          Email Address{" "}
                          <span className="text-[#33C5E0]">*</span>
                        </label>
                        <input
                          type="email"
                          value={formData.email}
                          onChange={(e) =>
                            updateFormData({ email: e.target.value })
                          }
                          className={`${inputClasses} ${errors.email ? "border-red-500" : ""}`}
                          placeholder="your@email.com"
                        />
                        {errors.email && (
                          <p className="text-red-500 text-xs mt-1">
                            {errors.email}
                          </p>
                        )}
                      </div>

                      <div>
                        <label className={labelClasses}>
                          Date of Birth{" "}
                          <span className="text-[#33C5E0]">*</span>
                        </label>
                        <input
                          type="date"
                          value={formData.dateOfBirth}
                          onChange={(e) =>
                            updateFormData({ dateOfBirth: e.target.value })
                          }
                          className={`${inputClasses} ${errors.dateOfBirth ? "border-red-500" : ""}`}
                        />
                        {errors.dateOfBirth && (
                          <p className="text-red-500 text-xs mt-1">
                            {errors.dateOfBirth}
                          </p>
                        )}
                      </div>

                      <div>
                        <label className={labelClasses}>
                          Nationality <span className="text-[#33C5E0]">*</span>
                        </label>
                        <input
                          type="text"
                          value={formData.nationality}
                          onChange={(e) =>
                            updateFormData({ nationality: e.target.value })
                          }
                          className={`${inputClasses} ${errors.nationality ? "border-red-500" : ""}`}
                          placeholder="Country of citizenship"
                        />
                        {errors.nationality && (
                          <p className="text-red-500 text-xs mt-1">
                            {errors.nationality}
                          </p>
                        )}
                      </div>
                    </div>
                  </div>
                </section>

                {/* Identity Document Section */}
                <section className="mb-6">
                  <div className="border border-[#1C252A] rounded-xl p-5">
                    <h3 className="text-lg font-semibold text-white mb-5">
                      Identity Document
                    </h3>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                      <div>
                        <label className={labelClasses}>
                          ID Type <span className="text-[#33C5E0]">*</span>
                        </label>
                        <div className="relative">
                          <select
                            value={formData.idType}
                            onChange={(e) =>
                              updateFormData({ idType: e.target.value })
                            }
                            className={`${inputClasses} appearance-none cursor-pointer`}
                          >
                            {ID_TYPES.map((type) => (
                              <option
                                key={type.value}
                                value={type.value}
                                className="bg-[#1C252A]"
                              >
                                {type.label}
                              </option>
                            ))}
                          </select>
                          <ChevronDown
                            className="absolute right-3 top-1/2 -translate-y-1/2 text-[#6B7C7F] pointer-events-none"
                            size={18}
                          />
                        </div>
                      </div>

                      <div>
                        <label className={labelClasses}>
                          ID Number <span className="text-[#33C5E0]">*</span>
                        </label>
                        <input
                          type="text"
                          value={formData.idNumber}
                          onChange={(e) =>
                            updateFormData({ idNumber: e.target.value })
                          }
                          className={`${inputClasses} ${errors.idNumber ? "border-red-500" : ""}`}
                          placeholder="Document number"
                        />
                        {errors.idNumber && (
                          <p className="text-red-500 text-xs mt-1">
                            {errors.idNumber}
                          </p>
                        )}
                      </div>
                    </div>

                    <div className="mb-4">
                      <label className={labelClasses}>
                        Expiry Date <span className="text-[#33C5E0]">*</span>
                      </label>
                      <input
                        type="date"
                        value={formData.expiryDate}
                        onChange={(e) =>
                          updateFormData({ expiryDate: e.target.value })
                        }
                        className={`${inputClasses} ${errors.expiryDate ? "border-red-500" : ""}`}
                      />
                      {errors.expiryDate && (
                        <p className="text-red-500 text-xs mt-1">
                          {errors.expiryDate}
                        </p>
                      )}
                    </div>

                    {/* File Upload */}
                    <div>
                      <label className={labelClasses}>
                        Upload ID Document{" "}
                        <span className="text-[#33C5E0]">*</span>
                      </label>
                      <div
                        onClick={() => fileInputRef.current?.click()}
                        onDragOver={handleDragOver}
                        onDragLeave={handleDragLeave}
                        onDrop={handleDrop}
                        className={`border-2 border-dashed rounded-xl p-8 text-center cursor-pointer transition-all ${
                          isDragOver
                            ? "border-[#33C5E0] bg-[#33C5E0]/10"
                            : errors.idDocument
                              ? "border-red-500"
                              : "border-[#2A3338] hover:border-[#33C5E0]/50"
                        }`}
                      >
                        <input
                          ref={fileInputRef}
                          type="file"
                          accept=".jpg,.jpeg,.png,.pdf"
                          onChange={handleFileChange}
                          className="hidden"
                        />
                        {formData.idDocument ? (
                          <div className="flex items-center justify-center gap-2 text-[#33C5E0]">
                            <Check size={20} />
                            <span className="font-medium">
                              {formData.idDocument.name}
                            </span>
                          </div>
                        ) : (
                          <>
                            <Upload
                              className="mx-auto text-[#33C5E0] mb-2"
                              size={24}
                            />
                            <p className="text-white text-sm">
                              Click to upload
                            </p>
                            <p className="text-[#6B7C7F] text-xs mt-1">
                              JPG,JPEG,PNG,PDF (max 5MB)
                            </p>
                          </>
                        )}
                      </div>
                      {errors.idDocument && (
                        <p className="text-red-500 text-xs mt-1">
                          {errors.idDocument}
                        </p>
                      )}
                    </div>
                  </div>
                </section>

                {/* Address Section */}
                <section className="mb-6">
                  <div className="border border-[#1C252A] rounded-xl p-5">
                    <h3 className="text-lg font-semibold text-white mb-5">
                      Address
                    </h3>

                    <div className="space-y-4">
                      <div>
                        <label className={labelClasses}>
                          Street Address{" "}
                          <span className="text-[#33C5E0]">*</span>
                        </label>
                        <input
                          type="text"
                          value={formData.streetAddress}
                          onChange={(e) =>
                            updateFormData({ streetAddress: e.target.value })
                          }
                          className={`${inputClasses} ${errors.streetAddress ? "border-red-500" : ""}`}
                          placeholder="Street address"
                        />
                        {errors.streetAddress && (
                          <p className="text-red-500 text-xs mt-1">
                            {errors.streetAddress}
                          </p>
                        )}
                      </div>

                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                          <label className={labelClasses}>
                            City <span className="text-[#33C5E0]">*</span>
                          </label>
                          <input
                            type="text"
                            value={formData.city}
                            onChange={(e) =>
                              updateFormData({ city: e.target.value })
                            }
                            className={`${inputClasses} ${errors.city ? "border-red-500" : ""}`}
                            placeholder="City"
                          />
                          {errors.city && (
                            <p className="text-red-500 text-xs mt-1">
                              {errors.city}
                            </p>
                          )}
                        </div>

                        <div>
                          <label className={labelClasses}>
                            Country <span className="text-[#33C5E0]">*</span>
                          </label>
                          <input
                            type="text"
                            value={formData.country}
                            onChange={(e) =>
                              updateFormData({ country: e.target.value })
                            }
                            className={`${inputClasses} ${errors.country ? "border-red-500" : ""}`}
                            placeholder="Country"
                          />
                          {errors.country && (
                            <p className="text-red-500 text-xs mt-1">
                              {errors.country}
                            </p>
                          )}
                        </div>
                      </div>

                      <div className="md:w-1/2">
                        <label className={labelClasses}>
                          Postal Code <span className="text-[#33C5E0]">*</span>
                        </label>
                        <input
                          type="text"
                          value={formData.postalCode}
                          onChange={(e) =>
                            updateFormData({ postalCode: e.target.value })
                          }
                          className={`${inputClasses} ${errors.postalCode ? "border-red-500" : ""}`}
                          placeholder="Postal code"
                        />
                        {errors.postalCode && (
                          <p className="text-red-500 text-xs mt-1">
                            {errors.postalCode}
                          </p>
                        )}
                      </div>
                    </div>
                  </div>
                </section>
              </form>

              {/* Footer with Submit Button */}
              <div className="p-6 pt-4 border-t border-[#1C252A]">
                <button
                  onClick={handleSubmit}
                  disabled={isSubmitting}
                  className="ml-auto flex items-center gap-2 bg-[#33C5E0] text-[#0D1417] px-6 py-3 rounded-lg font-semibold hover:bg-[#2AB8D3] transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isSubmitting ? (
                    <>
                      <Loader2 className="animate-spin" size={18} />
                      Submitting...
                    </>
                  ) : (
                    <>
                      <Upload size={18} />
                      Submit KYC
                    </>
                  )}
                </button>
              </div>
            </>
          )}
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
