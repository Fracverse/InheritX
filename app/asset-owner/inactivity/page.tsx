"use client";

import { motion } from "framer-motion";
import { useState, useCallback } from "react";
import clsx from "clsx";

type InactivityStep = "setup" | "filled" | "confirmation";

interface InactivityFormData {
  duration: string;
  beneficiaryName: string;
  beneficiaryEmail: string;
  claimCode: string;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.03,
      delayChildren: 0.05,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 8 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.25 },
  },
};

const durationOptions = [
  { value: "3-months", label: "3 Months" },
  { value: "6-months", label: "6 Months" },
  { value: "1-year", label: "1 Year" },
  { value: "1-month", label: "1 Month" },
];

export default function InactivityPage() {
  const [currentStep, setCurrentStep] = useState<InactivityStep>("setup");
  const [formData, setFormData] = useState<InactivityFormData>({
    duration: "3-months",
    beneficiaryName: "",
    beneficiaryEmail: "",
    claimCode: "123456",
  });
  const [isSaving, setIsSaving] = useState(false);

  const handleInputChange = useCallback((field: keyof InactivityFormData, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  }, []);

  const handleSaveSettings = useCallback(async () => {
    setIsSaving(true);
    await new Promise((resolve) => setTimeout(resolve, 1200));
    setCurrentStep("filled");
    setIsSaving(false);
  }, []);

  const handleConfirm = useCallback(async () => {
    setIsSaving(true);
    await new Promise((resolve) => setTimeout(resolve, 1000));
    setCurrentStep("confirmation");
    setIsSaving(false);
  }, []);

  const handleReset = useCallback(() => {
    setCurrentStep("setup");
    setFormData({
      duration: "3-months",
      beneficiaryName: "",
      beneficiaryEmail: "",
      claimCode: "123456",
    });
  }, []);

  return (
    <div className="w-full">
      {/* Setup Step */}
      {currentStep === "setup" && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.3 }}
        >
          <div className="mb-8">
            <h1 className="text-2xl font-semibold text-[#FCFFFF] mb-2">Inactivity Set-up</h1>
            <p className="text-sm text-[#92A5A8]">
              Define the condition under which your inheritance plans kicks in.
            </p>
          </div>

          <motion.div
            variants={containerVariants}
            initial="hidden"
            animate="visible"
            className="space-y-6"
          >
            {/* Inactivity Duration Section */}
            <motion.div variants={itemVariants} className="space-y-3">
              <label className="text-sm font-semibold text-[#FCFFFF]">Inactivity Duration</label>
              <div className="space-y-2">
                {durationOptions.map((option) => (
                  <motion.button
                    key={option.value}
                    whileHover={{ x: 2 }}
                    whileTap={{ scale: 0.98 }}
                    onClick={() => handleInputChange("duration", option.value)}
                    className={clsx(
                      "w-full text-left px-4 py-2.5 rounded-lg text-sm transition-all",
                      formData.duration === option.value
                        ? "bg-[#33C5E0]/20 text-[#33C5E0] border border-[#33C5E0]/40"
                        : "text-[#92A5A8] hover:text-[#FCFFFF] hover:bg-[#1C252A]/50"
                    )}
                  >
                    {option.label}
                  </motion.button>
                ))}
              </div>
            </motion.div>

            {/* Beneficiary Name Section */}
            <motion.div variants={itemVariants} className="space-y-2">
              <label htmlFor="beneficiary-name" className="text-sm font-semibold text-[#FCFFFF]">
                Beneficiary Name
              </label>
              <input
                id="beneficiary-name"
                type="text"
                value={formData.beneficiaryName}
                onChange={(e) => handleInputChange("beneficiaryName", e.target.value)}
                placeholder="Juliet Johnson"
                className="w-full px-4 py-2.5 rounded-lg text-sm bg-[#1C252A] border border-[#2A3338] text-[#FCFFFF] placeholder-[#92A5A8] focus:border-[#33C5E0] focus:outline-none transition-colors"
              />
            </motion.div>

            {/* Beneficiary Email Section */}
            <motion.div variants={itemVariants} className="space-y-2">
              <label htmlFor="beneficiary-email" className="text-sm font-semibold text-[#FCFFFF]">
                Beneficiary Email
              </label>
              <input
                id="beneficiary-email"
                type="email"
                value={formData.beneficiaryEmail}
                onChange={(e) => handleInputChange("beneficiaryEmail", e.target.value)}
                placeholder="e.g. thejulietjohnson@gmail.com"
                className="w-full px-4 py-2.5 rounded-lg text-sm bg-[#1C252A] border border-[#2A3338] text-[#FCFFFF] placeholder-[#92A5A8] focus:border-[#33C5E0] focus:outline-none transition-colors"
              />
            </motion.div>

            {/* Claim Code Section */}
            <motion.div variants={itemVariants} className="space-y-2">
              <label htmlFor="claim-code" className="text-sm font-semibold text-[#FCFFFF]">
                Claim Code
              </label>
              <input
                id="claim-code"
                type="text"
                value={formData.claimCode}
                readOnly
                className="w-full px-4 py-2.5 rounded-lg text-sm bg-[#1C252A] border border-[#2A3338] text-[#92A5A8] cursor-not-allowed"
              />
            </motion.div>

            {/* Claim Code Mechanism Info */}
            <motion.div
              variants={itemVariants}
              className="p-4 rounded-lg border border-[#33C5E0]/30 bg-[#33C5E0]/5 space-y-2"
            >
              <h4 className="text-sm font-semibold text-[#33C5E0]">Claim Code Mechanism</h4>
              <p className="text-xs text-[#33C5E0]/70 leading-relaxed">
                The code would be sent to the email of your beneficiary if you are inactive for a set period of
                time. With the claim code, your beneficiary would be able to claim the assets from the
                inheritance plans you have set.
              </p>
            </motion.div>

            {/* Save Button */}
            <motion.div variants={itemVariants} className="pt-2">
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={handleSaveSettings}
                disabled={isSaving || !formData.beneficiaryName || !formData.beneficiaryEmail}
                className={clsx(
                  "flex items-center gap-2 px-6 py-2.5 rounded-lg text-sm font-semibold transition-all",
                  isSaving || !formData.beneficiaryName || !formData.beneficiaryEmail
                    ? "bg-[#33C5E0]/30 text-[#FCFFFF] cursor-not-allowed"
                    : "bg-[#33C5E0] text-[#161E22] hover:bg-[#33C5E0]/90"
                )}
              >
                {isSaving ? "SAVING..." : "SAVE SETTINGS"}
                <span className="text-base">↗</span>
              </motion.button>
            </motion.div>
          </motion.div>
        </motion.div>
      )}

      {/* Filled Step */}
      {currentStep === "filled" && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.3 }}
        >
          <div className="mb-8">
            <h1 className="text-2xl font-semibold text-[#FCFFFF] mb-2">Inactivity Set-up</h1>
            <p className="text-sm text-[#92A5A8]">
              Define the condition under which your inheritance plans kicks in.
            </p>
          </div>

          <motion.div
            variants={containerVariants}
            initial="hidden"
            animate="visible"
            className="space-y-6"
          >
            {/* Inactivity Duration Section */}
            <motion.div variants={itemVariants} className="space-y-3">
              <label className="text-sm font-semibold text-[#FCFFFF]">Inactivity Duration</label>
              <div className="space-y-2">
                {durationOptions.map((option) => (
                  <div
                    key={option.value}
                    className={clsx(
                      "px-4 py-2.5 rounded-lg text-sm transition-all",
                      formData.duration === option.value
                        ? "bg-[#33C5E0]/20 text-[#33C5E0] border border-[#33C5E0]/40"
                        : "text-[#92A5A8]"
                    )}
                  >
                    {option.label}
                  </div>
                ))}
              </div>
            </motion.div>

            {/* Beneficiary Name Section */}
            <motion.div variants={itemVariants} className="space-y-2">
              <label className="text-sm font-semibold text-[#FCFFFF]">Beneficiary Name</label>
              <div className="px-4 py-2.5 rounded-lg text-sm bg-[#1C252A] border border-[#2A3338] text-[#FCFFFF]">
                {formData.beneficiaryName}
              </div>
            </motion.div>

            {/* Beneficiary Email Section */}
            <motion.div variants={itemVariants} className="space-y-2">
              <label className="text-sm font-semibold text-[#FCFFFF]">Beneficiary Email</label>
              <div className="px-4 py-2.5 rounded-lg text-sm bg-[#1C252A] border border-[#2A3338] text-[#FCFFFF]">
                {formData.beneficiaryEmail}
              </div>
            </motion.div>

            {/* Claim Code Section */}
            <motion.div variants={itemVariants} className="space-y-2">
              <label className="text-sm font-semibold text-[#FCFFFF]">Claim Code</label>
              <div className="px-4 py-2.5 rounded-lg text-sm bg-[#1C252A] border border-[#2A3338] text-[#92A5A8]">
                {formData.claimCode}
              </div>
            </motion.div>

            {/* Claim Code Mechanism Info */}
            <motion.div
              variants={itemVariants}
              className="p-4 rounded-lg border border-[#33C5E0]/30 bg-[#33C5E0]/5 space-y-2"
            >
              <h4 className="text-sm font-semibold text-[#33C5E0]">Claim Code Mechanism</h4>
              <p className="text-xs text-[#33C5E0]/70 leading-relaxed">
                The code would be sent to the email of your beneficiary if you are inactive for a set period of
                time. With the claim code, your beneficiary would be able to claim the assets from the
                inheritance plans you have set.
              </p>
            </motion.div>

            {/* Action Buttons */}
            <motion.div variants={itemVariants} className="flex gap-3 pt-2">
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={handleReset}
                className="flex-1 px-6 py-2.5 rounded-lg text-sm font-semibold border border-[#2A3338] text-[#FCFFFF] hover:bg-[#1C252A] transition-all"
              >
                Edit
              </motion.button>
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={handleConfirm}
                disabled={isSaving}
                className={clsx(
                  "flex-1 flex items-center justify-center gap-2 px-6 py-2.5 rounded-lg text-sm font-semibold transition-all",
                  isSaving
                    ? "bg-[#33C5E0]/30 text-[#FCFFFF] cursor-not-allowed"
                    : "bg-[#33C5E0] text-[#161E22] hover:bg-[#33C5E0]/90"
                )}
              >
                {isSaving ? "CONFIRMING..." : "CONFIRM"}
                <span className="text-base">↗</span>
              </motion.button>
            </motion.div>
          </motion.div>
        </motion.div>
      )}

      {/* Confirmation Step */}
      {currentStep === "confirmation" && (
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          transition={{ duration: 0.3 }}
        >
          <div className="mb-8">
            <h1 className="text-2xl font-semibold text-[#FCFFFF] mb-2">Inactivity Set-up</h1>
            <p className="text-sm text-[#92A5A8]">
              Your inactivity settings have been saved successfully.
            </p>
          </div>

          <motion.div
            variants={containerVariants}
            initial="hidden"
            animate="visible"
            className="space-y-6"
          >
            {/* Success Message */}
            <motion.div
              variants={itemVariants}
              className="p-6 rounded-lg border border-[#33C5E0]/30 bg-[#33C5E0]/10 text-center space-y-3"
            >
              <div className="flex justify-center">
                <div className="w-12 h-12 rounded-full bg-[#33C5E0]/20 flex items-center justify-center">
                  <span className="text-2xl text-[#33C5E0]">✓</span>
                </div>
              </div>
              <h3 className="text-lg font-semibold text-[#FCFFFF]">Settings Confirmed</h3>
              <p className="text-sm text-[#92A5A8]">
                Your inactivity plan is now active. Your beneficiary will be notified at {formData.beneficiaryEmail}
              </p>
            </motion.div>

            {/* Summary */}
            <motion.div variants={itemVariants} className="space-y-4">
              <h4 className="text-sm font-semibold text-[#FCFFFF]">Summary</h4>
              <div className="space-y-3">
                <div className="flex justify-between items-center p-3 rounded-lg bg-[#1C252A] border border-[#2A3338]">
                  <span className="text-sm text-[#92A5A8]">Inactivity Duration</span>
                  <span className="text-sm font-semibold text-[#FCFFFF]">
                    {durationOptions.find((o) => o.value === formData.duration)?.label}
                  </span>
                </div>
                <div className="flex justify-between items-center p-3 rounded-lg bg-[#1C252A] border border-[#2A3338]">
                  <span className="text-sm text-[#92A5A8]">Beneficiary</span>
                  <span className="text-sm font-semibold text-[#FCFFFF]">{formData.beneficiaryName}</span>
                </div>
                <div className="flex justify-between items-center p-3 rounded-lg bg-[#1C252A] border border-[#2A3338]">
                  <span className="text-sm text-[#92A5A8]">Email</span>
                  <span className="text-sm font-semibold text-[#FCFFFF]">{formData.beneficiaryEmail}</span>
                </div>
              </div>
            </motion.div>

            {/* Action Button */}
            <motion.div variants={itemVariants} className="pt-2">
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={handleReset}
                className="w-full px-6 py-2.5 rounded-lg text-sm font-semibold bg-[#33C5E0] text-[#161E22] hover:bg-[#33C5E0]/90 transition-all"
              >
                Create Another Plan
              </motion.button>
            </motion.div>
          </motion.div>
        </motion.div>
      )}
    </div>
  );
}