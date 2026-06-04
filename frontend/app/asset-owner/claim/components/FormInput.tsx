import React, { InputHTMLAttributes } from "react";

interface FormInputProps extends InputHTMLAttributes<HTMLInputElement> {
  label: string;
}

export default function FormInput({ label, className = "", ...props }: FormInputProps) {
  return (
    <div>
      <label className="block text-xs md:text-sm font-medium text-[#FCFFFF] mb-2">
        {label}
      </label>
      <input
        className={`w-full px-4 py-3 bg-[#161E22] border border-[#2A3338] rounded-lg text-sm md:text-base text-[#FCFFFF] placeholder:text-[#92A5A8] focus:outline-none focus:border-[#33C5E0] transition-colors ${className}`}
        {...props}
      />
    </div>
  );
}
