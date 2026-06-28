"use client";

import { motion } from "framer-motion";
import { useKYC } from "@/context/KYCContext";

export function PersonalInfoStep() {
  const { formData, updateFormData } = useKYC();

  const countries = [
    "United States",
    "Canada",
    "United Kingdom",
    "Australia",
    "Germany",
    "France",
    "Japan",
    "Other",
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      className="space-y-4"
    >
      <h3 className="text-sm font-semibold text-[#33C5E0] uppercase tracking-wider">
        Personal Information
      </h3>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Full Name *</label>
          <input
            type="text"
            value={formData.fullName}
            onChange={(e) => updateFormData({ fullName: e.target.value })}
            placeholder="John Doe"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Email *</label>
          <input
            type="email"
            value={formData.email}
            onChange={(e) => updateFormData({ email: e.target.value })}
            placeholder="john@example.com"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Date of Birth *</label>
          <input
            type="date"
            value={formData.dateOfBirth}
            onChange={(e) => updateFormData({ dateOfBirth: e.target.value })}
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Nationality *</label>
          <select
            value={formData.nationality}
            onChange={(e) => updateFormData({ nationality: e.target.value })}
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
          >
            <option value="">Select country...</option>
            {countries.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
        </div>
      </div>

      <p className="text-xs text-[#92A5A8] italic">
        * Required fields must be completed
      </p>
    </motion.div>
  );
}
