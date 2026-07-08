"use client";

import { Check, Circle, Loader2, X } from "lucide-react";
import type { BridgeStep, BridgeStepStatus } from "@/lib/bridge/types";

interface BridgeProgressStepperProps {
  steps: BridgeStep[];
}

function StepIcon({ status }: { status: BridgeStepStatus }) {
  if (status === "completed") {
    return (
      <span className="flex h-7 w-7 items-center justify-center rounded-full bg-[#48BB78]/20 text-[#48BB78]">
        <Check size={14} />
      </span>
    );
  }
  if (status === "active") {
    return (
      <span className="flex h-7 w-7 items-center justify-center rounded-full bg-[#33C5E0]/20 text-[#33C5E0]">
        <Loader2 size={14} className="animate-spin" />
      </span>
    );
  }
  if (status === "error") {
    return (
      <span className="flex h-7 w-7 items-center justify-center rounded-full bg-[#F56565]/20 text-[#F56565]">
        <X size={14} />
      </span>
    );
  }
  return (
    <span className="flex h-7 w-7 items-center justify-center rounded-full border border-[#2A3338] text-[#92A5A8]">
      <Circle size={10} />
    </span>
  );
}

function statusLabel(status: BridgeStepStatus): string {
  switch (status) {
    case "completed":
      return "Completed";
    case "active":
      return "In progress";
    case "error":
      return "Failed";
    default:
      return "Pending";
  }
}

export function BridgeProgressStepper({ steps }: BridgeProgressStepperProps) {
  return (
    <ol className="space-y-0" aria-label="Cross-chain bridge progress">
      {steps.map((step, index) => (
        <li key={step.id} className="relative flex gap-4 pb-6 last:pb-0">
          {index < steps.length - 1 && (
            <span
              className={`absolute left-3.5 top-7 h-[calc(100%-1.75rem)] w-px ${
                step.status === "completed" ? "bg-[#48BB78]/50" : "bg-[#2A3338]"
              }`}
              aria-hidden="true"
            />
          )}

          <div className="relative z-10 shrink-0">
            <StepIcon status={step.status} />
          </div>

          <div className="min-w-0 flex-1 pt-0.5">
            <div className="flex flex-wrap items-center gap-2">
              <p className="text-sm font-medium text-slate-100">{step.label}</p>
              <span
                className={`text-[10px] uppercase tracking-wider px-2 py-0.5 rounded-full ${
                  step.status === "completed"
                    ? "bg-[#48BB7814] text-[#48BB78]"
                    : step.status === "active"
                      ? "bg-[#33C5E014] text-[#33C5E0]"
                      : step.status === "error"
                        ? "bg-[#F5656514] text-[#F56565]"
                        : "bg-white/5 text-[#92A5A8]"
                }`}
              >
                {statusLabel(step.status)}
              </span>
            </div>
            {step.detail && (
              <p className="mt-1 text-xs text-[#92A5A8] break-words">{step.detail}</p>
            )}
          </div>
        </li>
      ))}
    </ol>
  );
}

export default BridgeProgressStepper;
