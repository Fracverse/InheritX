"use client";

import { GeneticInheritance } from "@/app/lib/api/geneticVerification";

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
    </div>
  );
}