"use client";

import { GeneticInheritance } from "@/app/lib/api/geneticVerification";
import InteractiveFamilyTree, {
  FamilyMemberNode,
  GeneticConnectionEdge,
  InheritanceFlowEdge,
} from "@/components/InteractiveFamilyTree";

interface Props {
  geneticInheritance: GeneticInheritance | null;
}

function conditionLabel(
  condition: GeneticInheritance["geneticTriggers"][number],
): string {
  switch (condition.kind) {
    case "hereditary_disease":
      return `Hereditary disease: ${condition.name}`;
    case "life_expectancy_marker":
      return "Life expectancy marker detected";
    case "carrier_status":
      return `Carrier status: ${condition.name}`;
    case "health_risk_factor":
      return `Health risk factor: ${condition.value}/100`;
    case "age_related_condition":
      return `Age-related condition: triggers at age ${condition.age}`;
  }
}

export default function GeneticDashboard({ geneticInheritance }: Props) {
  if (!geneticInheritance) {
    return (
      <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6 text-center">
        <p className="text-sm text-[#92A5A8]">
          No genetic verification on file yet. Upload your data to get
          started.
        </p>
      </div>
    );
  }

  const { dnaHash, verifiedLineage, geneticTriggers, verificationTimestamp } =
    geneticInheritance;
  const trackedConditions = geneticTriggers
    .map((condition) => conditionLabel(condition))
    .slice(0, 2);

  const familyMembers: FamilyMemberNode[] = [
    {
      id: "owner",
      name: "Plan Owner",
      relationshipLabel: "Self",
      relationshipDegree: 0,
      geneticSimilarity: 100,
      verificationStatus: verifiedLineage ? "verified" : "pending",
      confidenceLevel: verifiedLineage ? 95 : 62,
      healthRiskScore: geneticTriggers.length > 0 ? 66 : 34,
      healthConditions: trackedConditions,
    },
    {
      id: "spouse",
      name: "Spouse",
      relationshipLabel: "Partner",
      relationshipDegree: 1,
      geneticSimilarity: 2,
      verificationStatus: "verified",
      confidenceLevel: 91,
      healthRiskScore: 31,
      healthConditions: [],
    },
    {
      id: "child-1",
      name: "Child A",
      relationshipLabel: "First child",
      relationshipDegree: 1,
      geneticSimilarity: 50,
      verificationStatus: "verified",
      confidenceLevel: 88,
      healthRiskScore: trackedConditions.length > 0 ? 52 : 27,
      healthConditions: trackedConditions.slice(0, 1),
    },
    {
      id: "child-2",
      name: "Child B",
      relationshipLabel: "Second child",
      relationshipDegree: 1,
      geneticSimilarity: 49,
      verificationStatus: "partial_match",
      confidenceLevel: 76,
      healthRiskScore: 41,
      healthConditions: [],
    },
    {
      id: "parent",
      name: "Parent",
      relationshipLabel: "Parental lineage",
      relationshipDegree: 1,
      geneticSimilarity: 50,
      verificationStatus: "verified",
      confidenceLevel: 92,
      healthRiskScore: trackedConditions.length > 0 ? 58 : 36,
      healthConditions: trackedConditions.slice(0, 1),
    },
  ];

  const geneticConnections: GeneticConnectionEdge[] = [
    {
      id: "g-1",
      sourceId: "parent",
      targetId: "owner",
      relationshipType: "parent",
      relationshipDegree: 1,
      similarityStrength: 50,
      confidenceLevel: 92,
      verified: true,
    },
    {
      id: "g-2",
      sourceId: "owner",
      targetId: "child-1",
      relationshipType: "parent",
      relationshipDegree: 1,
      similarityStrength: 50,
      confidenceLevel: 89,
      verified: true,
    },
    {
      id: "g-3",
      sourceId: "owner",
      targetId: "child-2",
      relationshipType: "parent",
      relationshipDegree: 1,
      similarityStrength: 49,
      confidenceLevel: 77,
      verified: false,
    },
    {
      id: "g-4",
      sourceId: "child-1",
      targetId: "child-2",
      relationshipType: "sibling",
      relationshipDegree: 2,
      similarityStrength: 72,
      confidenceLevel: 80,
      verified: true,
    },
    {
      id: "g-5",
      sourceId: "owner",
      targetId: "spouse",
      relationshipType: "spouse",
      relationshipDegree: 1,
      similarityStrength: 5,
      confidenceLevel: 95,
      verified: true,
    },
  ];

  const inheritanceFlows: InheritanceFlowEdge[] = [
    {
      id: "f-1",
      sourceId: "owner",
      targetId: "spouse",
      assetType: "Emergency Liquidity",
      amount: 25000,
      currency: "USD",
      status: "planned",
    },
    {
      id: "f-2",
      sourceId: "owner",
      targetId: "child-1",
      assetType: "Education Trust",
      amount: 60000,
      currency: "USD",
      status: "active",
    },
    {
      id: "f-3",
      sourceId: "owner",
      targetId: "child-2",
      assetType: "Token Portfolio",
      amount: 42000,
      currency: "USD",
      status: "planned",
    },
    {
      id: "f-4",
      sourceId: "owner",
      targetId: "parent",
      assetType: "Care Fund",
      amount: 18000,
      currency: "USD",
      status: "distributed",
    },
  ];

  return (
    <div className="space-y-6">
      <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
        <h2 className="mb-4 text-xl font-semibold text-white">
          Verification Summary
        </h2>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <div className="rounded-lg border border-[#1C252A] bg-[#0A0F11] p-4">
            <p className="text-xs text-[#6B7C7F]">Lineage</p>
            <p
              className={`mt-1 font-semibold ${
                verifiedLineage ? "text-[#48BB78]" : "text-[#92A5A8]"
              }`}
            >
              {verifiedLineage ? "Verified" : "Not yet verified"}
            </p>
          </div>
          <div className="rounded-lg border border-[#1C252A] bg-[#0A0F11] p-4">
            <p className="text-xs text-[#6B7C7F]">Conditions tracked</p>
            <p className="mt-1 font-semibold text-white">
              {geneticTriggers.length}
            </p>
          </div>
          <div className="rounded-lg border border-[#1C252A] bg-[#0A0F11] p-4">
            <p className="text-xs text-[#6B7C7F]">Last verified</p>
            <p className="mt-1 font-semibold text-white">
              {verificationTimestamp
                ? new Date(verificationTimestamp * 1000).toLocaleDateString()
                : "—"}
            </p>
          </div>
        </div>
        <div className="mt-4 flex items-center justify-between rounded-lg border border-[#1C252A] bg-[#0A0F11] px-4 py-3">
          <span className="text-xs text-[#6B7C7F]">DNA fingerprint</span>
          <code className="text-xs text-[#92A5A8]">
            {dnaHash.slice(0, 12)}…{dnaHash.slice(-8)}
          </code>
        </div>
      </div>

      <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
        <h2 className="mb-4 text-xl font-semibold text-white">
          Detected Conditions
        </h2>
        {geneticTriggers.length === 0 ? (
          <p className="text-sm text-[#92A5A8]">
            No genetic conditions have been recorded.
          </p>
        ) : (
          <ul className="space-y-3">
            {geneticTriggers.map((condition, i) => (
              <li
                key={i}
                className="rounded-lg border border-[#1C252A] bg-[#0A0F11] px-4 py-3 text-sm text-white"
              >
                {conditionLabel(condition)}
              </li>
            ))}
          </ul>
        )}
      </div>

      <InteractiveFamilyTree
        members={familyMembers}
        geneticConnections={geneticConnections}
        inheritanceFlows={inheritanceFlows}
        defaultCenterMemberId="owner"
      />
    </div>
  );
}