"use client";

import { motion } from "framer-motion";
import { useKYC } from "@/context/KYCContext";

export function AddressStep() {
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
        Residential Address
      </h3>

      <div className="space-y-4">
        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Street Address *</label>
          <input
            type="text"
            value={formData.streetAddress}
            onChange={(e) => updateFormData({ streetAddress: e.target.value })}
            placeholder="123 Main Street"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="flex flex-col gap-1">
            <label className="text-xs text-[#92A5A8]">City *</label>
            <input
              type="text"
              value={formData.city}
              onChange={(e) => updateFormData({ city: e.target.value })}
              placeholder="New York"
              className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs text-[#92A5A8]">Postal Code *</label>
            <input
              type="text"
              value={formData.postalCode}
              onChange={(e) => updateFormData({ postalCode: e.target.value })}
              placeholder="10001"
              className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
            />
          </div>
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs text-[#92A5A8]">Country *</label>
          <select
            value={formData.country}
            onChange={(e) => updateFormData({ country: e.target.value })}
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
        Please provide your current residential address
      </p>
    </motion.div>
  );
}
