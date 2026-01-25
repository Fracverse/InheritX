"use client";

import React, { useState } from "react";
import { ArrowLeft, ChevronDown } from "lucide-react";
import DetailRow from "./DetailRow";
import CollapsibleSection from "./CollapsibleSection";

interface PlanSummaryData {
  planName: string;
  description: string;
  beneficiary: string;
  beneficiaryEmail: string;
  walletAddress: string;
  executeOn: string;
  assets: string[];
}

interface PlanSummaryProps {
  data: PlanSummaryData;
  onBack: () => void;
  onWithdraw?: () => void;
}

interface AssetItem {
  name: string;
  subtitle?: string;
  amount: string;
  usd: string;
  percentage: string;
}

export default function PlanSummary({ data, onBack, onWithdraw }: PlanSummaryProps) {
  const [expandedSections, setExpandedSections] = useState<string[]>([]);

  const toggleSection = (section: string) => {
    setExpandedSections((prev) =>
      prev.includes(section)
        ? prev.filter((s) => s !== section)
        : [...prev, section]
    );
  };

  // Mock data for expanded sections
  const assetsData = [
    {
      type: "Tokens",
      items: [
        { name: "ETH", amount: "2", usd: "$5,482", percentage: "10%" },
      ] as AssetItem[],
    },
    {
      type: "NFTs",
      items: [
        { name: "Monkey Art", amount: "3", usd: "$6,011", percentage: "30%" },
      ] as AssetItem[],
    },
    {
      type: "RWA",
      items: [
        { name: "Real World Asset", subtitle: "Mercedes-Benz S-Class", amount: "1", usd: "$17,750", percentage: "60%" },
      ] as AssetItem[],
    },
  ];

  const rulesData = {
    claimCode: "102667",
    distribution: "Yearly Release of Funds (disbursement)",
  };

  const legalSettingsData = {
    documents: [
      { name: "ID - Front.PNG", image: "/placeholder-doc1.png" },
      { name: "ID - Back.PNG", image: "/placeholder-doc2.png" },
      { name: "Selfie", image: "/placeholder-doc3.png" },
    ],
    trustee: {
      name: "John Doe",
      phone: "+234 812 3456 678",
      email: "Johndoe@gmail.com",
    },
  };

  const notesData = "Release funds monthly for upkeep of the property.";

  return (
    <div className="animate-fade-in w-full">
      {/* Header - Mobile Stacked, Desktop Side by Side */}
      <div className="mb-4 md:mb-6 pb-4 border-b border-[#33C5E014]">
        <div className="flex items-center gap-3 md:gap-4">
          {/* Back button */}
          <button
            type="button"
            onClick={onBack}
            className="w-8 h-8 md:w-10 md:h-10 flex items-center justify-center text-[#92A5A8] hover:text-[#FCFFFF] transition-colors shrink-0"
            aria-label="Go back"
          >
            <ArrowLeft size={20} />
          </button>
          
          {/* Title and description */}
          <div className="flex-1 min-w-0">
            <div className="flex flex-col md:flex-row md:items-center">
              <h1 className="text-[18px] md:text-[24px] font-medium text-[#92A5A8] whitespace-nowrap">
                Claim Plan
              </h1>
              
              {/* Single separator between h1 and p - only on desktop */}
              <span className="hidden md:inline border-2 border-[#1C252A] h-5 mx-4"></span>
              
              <p className="text-xs md:text-sm text-[#92A5A8] mt-1 md:mt-0 md:line-clamp-1">
                To transfer inheritance to your wallet, click on the &apos;Withdraw&apos; button.
              </p>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-4xl mx-auto animate-fade-in w-full">
        <div className="mb-6 md:mb-8 flex items-start md:items-center justify-between flex-col md:flex-row gap-4">
          <div className="flex items-center gap-3 md:gap-4">
            <h1 className="text-[16px] md:text-[18px] font-semibold text-[#FCFFFF]">
              Plan Summary        
            </h1>
          </div>
          <button
            type="button"
            onClick={onWithdraw}
            className="w-full md:w-auto px-6 py-3 border border-[#33C5E03D] bg-[#33C5E014] text-[#33C5E0] rounded-3xl hover:bg-[#33C5E0]/10 transition-colors font-medium text-sm md:text-base"
          >
            WITHDRAW ALL
          </button>
        </div>

        {/* Plan Details */}
        <div className="bg-[#182024] rounded-2xl p-4 md:p-8">
          <div className="space-y-3 md:space-y-4">
            <DetailRow label="PLAN NAME" value={data.planName} />
            <DetailRow label="DESCRIPTION" value={data.description} />
            
            <DetailRow 
              label="BENEFICIARY" 
              value={
                <div className="flex items-center gap-2">
                  <div className="w-6 h-6 rounded-full bg-[#33C5E0] flex items-center justify-center text-xs text-[#161E22] font-semibold shrink-0">
                    {data.beneficiary.charAt(0).toUpperCase()}
                  </div>
                  <a
                    href="#"
                    className="text-sm md:text-base text-[#33C5E0] underline font-medium break-all"
                  >
                    {data.beneficiary}
                  </a>
                </div>
              } 
            />

            <DetailRow label="ASSETS" value={data.assets.join(", ")} />
            
            <DetailRow 
              label="EMAIL" 
              value={
                <a
                  href={`mailto:${data.beneficiaryEmail}`}
                  className="text-sm md:text-base text-[#92A5A8] underline font-medium break-all"
                >
                  {data.beneficiaryEmail}
                </a>
              } 
            />

            <DetailRow 
              label="WALLET ADDRESS" 
              value={<span className="font-mono">{data.walletAddress}</span>} 
            />
            
            <DetailRow label="EXECUTE ON" value={data.executeOn} last />
          </div>

          <div className="mt-6 md:mt-8 space-y-0">
            {/* Assets Section */}
            <CollapsibleSection 
              title="Assets" 
              isOpen={expandedSections.includes("Assets")} 
              onToggle={() => toggleSection("Assets")}
            >
              <div className="space-y-6">
                {assetsData.map((category, idx) => (
                  <div key={idx}>
                    <h3 className="text-xs md:text-sm text-[#92A5A8] uppercase mb-4">{category.type}</h3>
                    {category.items.map((item, itemIdx) => (
                      <div key={itemIdx} className="bg-[#1C252A] rounded-xl p-4 md:p-5 mb-3">
                        <div className="flex items-center justify-between mb-4">
                          <div className="flex items-center gap-3">
                            <div className="w-8 h-8 md:w-10 md:h-10 rounded-full bg-[#33C5E0] flex items-center justify-center">
                              <span className="text-sm md:text-base font-bold text-[#0A0F11]">
                                {item.name.charAt(0)}
                              </span>
                            </div>
                            <div>
                              <div className="text-sm md:text-base text-[#FCFFFF] font-medium">{item.name}</div>
                              {item.subtitle && (
                                <div className="text-xs text-[#92A5A8]">{item.subtitle}</div>
                              )}
                            </div>
                          </div>
                          <ChevronDown className="text-[#92A5A8] w-5 h-5" />
                        </div>
                        <div className="grid grid-cols-3 gap-4">
                          <div>
                            <div className="text-xs text-[#92A5A8] mb-1">Amount</div>
                            <div className="text-sm md:text-base text-[#FCFFFF] font-medium">{item.amount}</div>
                            <div className="text-xs text-[#92A5A8]">{item.usd}</div>
                          </div>
                          <div>
                            <div className="text-xs text-[#92A5A8] mb-1">%</div>
                            <div className="text-sm md:text-base text-[#FCFFFF] font-medium">{item.percentage}</div>
                          </div>
                          <div className="flex items-end justify-end">
                            <button type="button" className="px-4 md:px-6 py-2 bg-[#33C5E0] text-[#0A0F11] rounded-full text-xs md:text-sm font-semibold hover:bg-[#2AB4CF] transition-colors">
                              WITHDRAW
                            </button>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                ))}
              </div>
            </CollapsibleSection>

            {/* Rules & Conditions Section */}
            <CollapsibleSection 
              title="Rules & Conditions" 
              isOpen={expandedSections.includes("Rules & Conditions")} 
              onToggle={() => toggleSection("Rules & Conditions")}
            >
              <div className="space-y-4">
                <DetailRow label="CLAIM CODE" value={rulesData.claimCode} className="border-0 pb-0" />
                <DetailRow label="DISTRIBUTION" value={rulesData.distribution} className="border-0 pb-0" />
              </div>
            </CollapsibleSection>

            {/* Legal Settings Section */}
            <CollapsibleSection 
              title="Legal Settings" 
              isOpen={expandedSections.includes("Legal Settings")} 
              onToggle={() => toggleSection("Legal Settings")}
            >
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm md:text-base text-[#FCFFFF] font-medium mb-4">Attached Documents</h3>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-3 md:gap-4">
                    {legalSettingsData.documents.map((doc, idx) => (
                      <div key={idx} className="text-center">
                        <div className="bg-[#2A3338] rounded-lg aspect-3/4 mb-2 flex items-center justify-center">
                          <span className="text-xs text-[#92A5A8]">Document Preview</span>
                        </div>
                        <p className="text-xs text-[#92A5A8]">{doc.name}</p>
                      </div>
                    ))}
                  </div>
                </div>

                <div>
                  <h3 className="text-sm md:text-base text-[#FCFFFF] font-medium mb-4">Assigned Trustee</h3>
                  <div className="space-y-3">
                    <DetailRow label="NAME" value={legalSettingsData.trustee.name} className="border-0 pb-0" />
                    <DetailRow label="PHONE NO" value={legalSettingsData.trustee.phone} className="border-0 pb-0" />
                    <DetailRow label="EMAIL" value={legalSettingsData.trustee.email} className="border-0 pb-0" />
                  </div>
                </div>
              </div>
            </CollapsibleSection>

            {/* Notes Section */}
            <CollapsibleSection 
              title="Notes" 
              isOpen={expandedSections.includes("Notes")} 
              onToggle={() => toggleSection("Notes")}
              className="border-b-0"
            >
              <p className="text-sm md:text-base text-[#FCFFFF]">{notesData}</p>
            </CollapsibleSection>
          </div>
        </div>
      </div>
    </div>
  );
}