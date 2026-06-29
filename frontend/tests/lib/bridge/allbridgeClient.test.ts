import { describe, it, expect } from "vitest";
import {
  applyBridgeStatusUpdate,
  createInitialBridgeSteps,
  buildQuoteFromEstimates,
} from "@/lib/bridge";
import { DEFAULT_TOKENS } from "@/lib/bridge/chains";

describe("bridge steps", () => {
  it("creates four ordered steps with approval active", () => {
    const steps = createInitialBridgeSteps();
    expect(steps).toHaveLength(4);
    expect(steps[0].id).toBe("approval");
    expect(steps[0].status).toBe("active");
    expect(steps[1].status).toBe("pending");
  });

  it("marks prior steps completed when a step completes", () => {
    const initial = createInitialBridgeSteps();
    const updated = applyBridgeStatusUpdate(initial, {
      step: "approval",
      status: "completed",
      detail: "Approved",
    });

    expect(updated[0].status).toBe("completed");
    expect(updated[1].status).toBe("active");
  });
});

describe("buildQuoteFromEstimates", () => {
  it("returns USD fee breakdown for supported chains", () => {
    const quote = buildQuoteFromEstimates({
      sourceChain: "ETH",
      sourceToken: DEFAULT_TOKENS.ETH[0],
      amount: "100",
      destinationStellarAddress: "GTEST123",
    });

    expect(quote.relayerFeeUsd).toBeGreaterThan(0);
    expect(quote.gasFeeUsd).toBeGreaterThan(0);
    expect(quote.destinationFeeUsd).toBeGreaterThan(0);
    expect(quote.totalFeeUsd).toBe(
      quote.relayerFeeUsd + quote.gasFeeUsd + quote.destinationFeeUsd
    );
    expect(Number.parseFloat(quote.estimatedReceiveAmount)).toBeLessThan(100);
  });
});
