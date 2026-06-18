"use client";

import {
  Clock,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  RotateCcw,
  type LucideIcon,
} from "lucide-react";
import { DNAVerificationStatus } from "@/app/lib/api/geneticVerification";

interface Props {
  status: DNAVerificationStatus;
  dnaHash: string;
}

const STATUS_CONFIG: Record<
  DNAVerificationStatus,
  { label: string; description: string; icon: LucideIcon; color: string }
> = {
  pending: {
    label: "Pending",
    description: "Your submission is queued for verification.",
    icon: Clock,
    color: "#ECC94B",
  },
  verified: {
    label: "Verified",
    description: "Your genetic verification has been confirmed.",
    icon: CheckCircle2,
    color: "#48BB78",
  },
  rejected: {
    label: "Rejected",
    description: "Verification could not be completed with this submission.",
    icon: XCircle,
    color: "#F56565",
  },
  partial_match: {
    label: "Partial Match",
    description: "Some markers matched; additional review is required.",
    icon: AlertTriangle,
    color: "#ED8936",
  },
  requires_retest: {
    label: "Requires Retest",
    description: "Please resubmit a new sample for verification.",
    icon: RotateCcw,
    color: "#33C5E0",
  },
};

export default function VerificationStatusCard({ status, dnaHash }: Props) {
  const config = STATUS_CONFIG[status];
  const Icon = config.icon;

  return (
    <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
      <div className="flex items-center gap-4">
        <div
          className="flex h-12 w-12 items-center justify-center rounded-full"
          style={{ backgroundColor: `${config.color}33` }}
        >
          <Icon size={24} style={{ color: config.color }} />
        </div>
        <div>
          <h3 className="text-lg font-semibold text-white">{config.label}</h3>
          <p className="text-sm text-[#92A5A8]">{config.description}</p>
        </div>
      </div>
      <div className="mt-4 flex items-center justify-between rounded-lg border border-[#1C252A] bg-[#0A0F11] px-4 py-3">
        <span className="text-xs text-[#6B7C7F]">DNA fingerprint</span>
        <code className="text-xs text-[#92A5A8]">
          {dnaHash.slice(0, 12)}…{dnaHash.slice(-8)}
        </code>
      </div>
    </div>
  );
}