"use client";

import { useState } from "react";
import { useParams } from "next/navigation";
import GeneticUploadComponent from "./components/GeneticUploadComponent";
import PrivacyControls from "./components/PrivacyControls";
import GeneticDashboard from "./components/GeneticDashboard";
import FamilyInviteComponent from "./components/FamilyInviteComponent";
import {
  DEFAULT_PRIVACY_SETTINGS,
  FamilyInvitation,
  GeneticInheritance,
  GeneticUploadResult,
  PrivacySettings,
} from "@/app/lib/api/geneticVerification";

type Tab = "upload" | "privacy" | "dashboard" | "family";

const TABS: { id: Tab; label: string }[] = [
  { id: "upload", label: "Upload" },
  { id: "privacy", label: "Privacy" },
  { id: "dashboard", label: "Dashboard" },
  { id: "family", label: "Family" },
];

export default function GeneticVerificationPage() {
  const params = useParams();
  const planId = (params?.id as string) ?? "";

  const [activeTab, setActiveTab] = useState<Tab>("upload");
  const [geneticInheritance, setGeneticInheritance] =
    useState<GeneticInheritance | null>(null);
  const [privacySettings, setPrivacySettings] = useState<PrivacySettings>(
    DEFAULT_PRIVACY_SETTINGS,
  );
  const [invitations, setInvitations] = useState<FamilyInvitation[]>([]);

  // Until the backend exposes GET /api/genetic/:planId, we derive a minimal
  // local view straight from the upload response so the Dashboard tab has
  // something to show. Replace with a getVerificationStatus() refetch once
  // that route exists.
  const handleUploadComplete = (result: GeneticUploadResult) => {
    setGeneticInheritance((prev) => ({
      dnaHash: result.dnaHash,
      verifiedLineage: prev?.verifiedLineage ?? false,
      geneticTriggers: prev?.geneticTriggers ?? [],
      familyTreeId: prev?.familyTreeId ?? 0,
      verificationTimestamp: Math.floor(Date.now() / 1000),
      verifyingAuthority: prev?.verifyingAuthority ?? "",
    }));
    setActiveTab("dashboard");
  };

  return (
    <div className="min-h-screen bg-[#0A0F11] px-4 py-8 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-5xl">
        <header className="mb-8">
          <h1 className="text-3xl font-bold text-white">
            Genetic Verification
          </h1>
          <p className="mt-2 text-sm text-[#92A5A8]">
            Link a genetic verification to this inheritance plan and control
            who can see it.
          </p>
        </header>

        <div className="mb-8 inline-flex space-x-1 rounded-lg border border-[#1C252A] bg-[#0D1417] p-1">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`rounded-md px-5 py-2.5 text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? "bg-[#33C5E0] text-[#0D1417]"
                  : "text-[#92A5A8] hover:bg-[#1C252A] hover:text-white"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {activeTab === "upload" && (
          <GeneticUploadComponent
            planId={planId}
            onUploadComplete={handleUploadComplete}
          />
        )}
        {activeTab === "privacy" && (
          <PrivacyControls
            planId={planId}
            settings={privacySettings}
            onSettingsChange={setPrivacySettings}
          />
        )}
        {activeTab === "dashboard" && (
          <GeneticDashboard geneticInheritance={geneticInheritance} />
        )}
        {activeTab === "family" && (
          <FamilyInviteComponent
            planId={planId}
            invitations={invitations}
            onInviteSent={(invite) =>
              setInvitations((prev) => [...prev, invite])
            }
          />
        )}
      </div>
    </div>
  );
}