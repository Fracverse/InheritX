import type { BridgeStep, BridgeStepId, BridgeStepStatus } from "./types";

export const BRIDGE_STEP_DEFINITIONS: { id: BridgeStepId; label: string }[] = [
  { id: "approval", label: "Bridging transaction approval" },
  { id: "source_execution", label: "Bridge execution on source chain" },
  { id: "relayer", label: "Relayer processing" },
  { id: "stellar_lock", label: "Final token lock on Stellar/Soroban" },
];

export function createInitialBridgeSteps(): BridgeStep[] {
  return BRIDGE_STEP_DEFINITIONS.map((step, index) => ({
    ...step,
    status: index === 0 ? "active" : "pending",
  }));
}

export function applyBridgeStatusUpdate(
  steps: BridgeStep[],
  update: { step: BridgeStepId; status: BridgeStepStatus; detail?: string }
): BridgeStep[] {
  const stepIndex = steps.findIndex((s) => s.id === update.step);
  if (stepIndex === -1) return steps;

  return steps.map((step, index) => {
    if (index < stepIndex) {
      return { ...step, status: "completed" as const };
    }
    if (index === stepIndex) {
      return {
        ...step,
        status: update.status,
        detail: update.detail ?? step.detail,
      };
    }
    if (index === stepIndex + 1 && update.status === "completed") {
      return { ...step, status: "active" as const };
    }
    return step;
  });
}
