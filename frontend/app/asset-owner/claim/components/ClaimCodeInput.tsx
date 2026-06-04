"use client";

import React from "react";

interface ClaimCodeInputProps {
  value: string[];
  onChange: (code: string[]) => void;
}

export default function ClaimCodeInput({ value, onChange }: ClaimCodeInputProps) {
  const handleChange = (index: number, newValue: string) => {
    if (newValue.length > 1) return;
    const newCode = [...value];
    newCode[index] = newValue.toUpperCase();
    onChange(newCode);

    // Auto-focus next input
    if (newValue && index < 5) {
      const nextInput = document.getElementById(`claim-code-${index + 1}`);
      nextInput?.focus();
    }
  };

  const handleKeyDown = (index: number, e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Backspace" && !value[index] && index > 0) {
      const prevInput = document.getElementById(`claim-code-${index - 1}`);
      prevInput?.focus();
    }
  };

  const handlePaste = (e: React.ClipboardEvent<HTMLInputElement>) => {
    e.preventDefault();
    const pastedData = e.clipboardData.getData("text").slice(0, 6).toUpperCase();
    const newCode = [...value];
    pastedData.split("").forEach((char, i) => {
      if (i < 6) newCode[i] = char;
    });
    onChange(newCode);
    const nextEmptyIndex = newCode.findIndex((c) => !c);
    if (nextEmptyIndex !== -1) {
      const nextInput = document.getElementById(`claim-code-${nextEmptyIndex}`);
      nextInput?.focus();
    }
  };

  return (
    <div className="flex gap-2 min-[380px]:gap-3 md:gap-4 justify-center flex-wrap">
      {value.map((code, index) => {
        const isFilled = code !== "";
        return (
          <input
            key={index}
            id={`claim-code-${index}`}
            type="password"
            inputMode="numeric"
            autoComplete="off"
            data-lpignore="true"
            maxLength={1}
            value={code}
            onChange={(e) => handleChange(index, e.target.value)}
            onKeyDown={(e) => handleKeyDown(index, e)}
            onPaste={handlePaste}
            className={`w-10 h-10 min-[380px]:w-12 min-[380px]:h-12 md:w-16 md:h-16 rounded-xl border-2 text-center text-lg md:text-2xl font-bold focus:outline-none transition-all ${
              isFilled
                ? "bg-[#33C5E014] border border-[#33C5E03D] text-[#FCFFFF] placeholder:text-[#FCFFFF]"
                : "bg-[#161E22] border-[#161E22] shadow-[4px_4px_10px_0px_#131A1DE5_inset,-4px_-4px_8px_0px_#192227E5_inset,4px_-4px_8px_0px_#131A1D33_inset,-4px_4px_8px_0px_#131A1D33_inset]  text-[#FCFFFF] placeholder:text-[#5A6C6F]"
            } placeholder:text-4xl md:placeholder:text-5xl`}
            placeholder="•"
          />
        );
      })}
    </div>
  );
}