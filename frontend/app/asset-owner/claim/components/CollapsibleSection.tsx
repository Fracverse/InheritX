import React, { ReactNode } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";

interface CollapsibleSectionProps {
  title: string;
  isOpen: boolean;
  onToggle: () => void;
  children: ReactNode;
  className?: string;
}

export default function CollapsibleSection({ 
  title, 
  isOpen, 
  onToggle, 
  children,
  className = "" 
}: CollapsibleSectionProps) {
  return (
    <div className={`border-b border-[#1C252A] ${className}`}>
      <button
        type="button"
        onClick={onToggle}
        className="w-full flex items-center justify-between py-4 md:py-5 hover:bg-[#1C252A]/30 transition-colors"
      >
        <span className="text-sm md:text-base text-[#33C5E0] font-medium">{title}</span>
        {isOpen ? (
          <ChevronUp className="text-[#92A5A8] w-5 h-5" />
        ) : (
          <ChevronDown className="text-[#92A5A8] w-5 h-5" />
        )}
      </button>
      
      {isOpen && (
        <div className="pb-6">
          {children}
        </div>
      )}
    </div>
  );
}
