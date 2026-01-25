import React, { useState } from "react";
import { ArrowUpRight } from "lucide-react";
import ClaimCodeInput from "./ClaimCodeInput";
import FormInput from "./FormInput";

interface ClaimFormProps {
  onSubmit: (data: { beneficiaryName: string; beneficiaryEmail: string; claimCode: string[] }) => void;
}

export default function ClaimForm({ onSubmit }: ClaimFormProps) {
  const [formData, setFormData] = useState({
    beneficiaryName: "",
    beneficiaryEmail: "",
    claimCode: ["", "", "", "", "", ""],
  });

  // Basic email validation regex
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

  const isFormValid =
    formData.beneficiaryName &&
    emailRegex.test(formData.beneficiaryEmail) &&
    formData.claimCode.every((code) => code);

  const handleSubmit = () => {
    if (isFormValid) {
      onSubmit(formData);
    }
  };

  return (
    // Overlay - absolute positioning instead of fixed
    <div className="absolute inset-0 bg-[#161E22CC]/50 flex items-center justify-center p-4 z-50">
      <div className="max-w-134 mx-auto w-full">
        <div className="border border-[#2A3338] bg-[#161E22] rounded-3xl p-4 md:p-8">
          <div className="mb-6 md:mb-8">
            <h1 className="text-xl md:text-3xl font-semibold text-[#FCFFFF] mb-2">
              Claim Plan
            </h1>
            <p className="text-xs md:text-base text-[#92A5A8]">
              Input your details to claim your inheritance
            </p>
          </div>

          <div className="space-y-4 md:space-y-6">
            <FormInput
              label="Beneficiary Name"
              type="text"
              value={formData.beneficiaryName}
              onChange={(e) =>
                setFormData({ ...formData, beneficiaryName: e.target.value })
              }
              placeholder="Enter the name of your beneficiary"
            />

            <FormInput
              label="Beneficiary Email"
              type="email"
              value={formData.beneficiaryEmail}
              onChange={(e) =>
                setFormData({ ...formData, beneficiaryEmail: e.target.value })
              }
              placeholder="Enter the email of your beneficiary"
            />

            {/* Claim Code */}
            <div>
              <label className="block text-xs md:text-sm font-medium text-[#FCFFFF] mb-3 md:mb-2">
                Input Claim Code
              </label>
              <ClaimCodeInput 
                value={formData.claimCode} 
                onChange={(code) => setFormData({ ...formData, claimCode: code })} 
              />
            </div>

            {/* Submit Button */}
            <button
              type="button"
              onClick={handleSubmit}
              disabled={!isFormValid}
              className={`w-full md:w-60.75 md:mx-auto h-12 flex flex-row items-center justify-center gap-4 transition-all mt-6 md:mt-8 font-sans font-medium text-sm uppercase tracking-normal px-12 whitespace-nowrap ${
                isFormValid
                  ? "bg-[#33C5E0] text-[#0A0F11] rounded-t-lg rounded-b-3xl hover:bg-[#2AB4CF]"
                  : "bg-transparent border-2 border-[#2A3338] text-[#33C5E0] rounded-t-lg rounded-b-3xl cursor-not-allowed opacity-60"
              }`}
              style={{
                borderTopLeftRadius: '8px',
                borderTopRightRadius: '8px',
                borderBottomRightRadius: '24px',
                borderBottomLeftRadius: '24px'
              }}
            >
              CLAIM INHERITANCE
              <ArrowUpRight size={18} className="shrink-0" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}