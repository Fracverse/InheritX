import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { BridgeProgressStepper } from "@/components/plans/BridgeProgressStepper";
import { createInitialBridgeSteps } from "@/lib/bridge/steps";

describe("BridgeProgressStepper", () => {
  it("renders all bridge steps with labels", () => {
    render(<BridgeProgressStepper steps={createInitialBridgeSteps()} />);

    expect(
      screen.getByText("Bridging transaction approval")
    ).toBeInTheDocument();
    expect(
      screen.getByText("Bridge execution on source chain")
    ).toBeInTheDocument();
    expect(screen.getByText("Relayer processing")).toBeInTheDocument();
    expect(
      screen.getByText("Final token lock on Stellar/Soroban")
    ).toBeInTheDocument();
  });

  it("shows in progress status for active step", () => {
    const steps = createInitialBridgeSteps();
    render(<BridgeProgressStepper steps={steps} />);
    expect(screen.getByText("In progress")).toBeInTheDocument();
  });
});
