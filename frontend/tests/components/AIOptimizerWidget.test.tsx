import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

const api = vi.hoisted(() => ({
  getPlanOptimization: vi.fn(),
  applyOptimization: vi.fn(),
}));
vi.mock("@/lib/api/aiOptimization", () => api);

import { AIOptimizerWidget } from "@/components/AIOptimizerWidget";

const rec = (current: number) => ({
  id: "rec_1",
  planId: 1,
  recommendedAllocations: [
    {
      assetSymbol: "XLM",
      chain: "Stellar",
      currentPercentage: current,
      recommendedPercentage: 30,
      adjustmentReason: "Reduce concentration risk",
      expectedImpact: "Lower volatility",
    },
  ],
  confidenceScore: 87,
  expectedReturn: 14.3,
  riskScore: 42,
  reasoning: "Server reasoning",
  generatedAt: "2026-01-01T00:00:00Z",
  projectedOutcomes: {},
});

beforeEach(() => {
  vi.resetAllMocks();
});

describe("AIOptimizerWidget", () => {
  it("shows a loading state first", () => {
    api.getPlanOptimization.mockReturnValue(new Promise(() => {}));
    render(<AIOptimizerWidget planId="1" />);
    expect(screen.getByRole("status", { name: /loading/i })).toBeInTheDocument();
  });

  it("renders live recommendation data", async () => {
    api.getPlanOptimization.mockResolvedValue(rec(45));
    render(<AIOptimizerWidget planId="1" />);
    expect(await screen.findByText("Server reasoning")).toBeInTheDocument();
    expect(screen.getByText("45% → 30%")).toBeInTheDocument();
  });

  it("shows an empty state when there are no recommendations", async () => {
    api.getPlanOptimization.mockResolvedValue({ ...rec(45), recommendedAllocations: [] });
    render(<AIOptimizerWidget planId="1" />);
    expect(await screen.findByText("No Recommendations Available")).toBeInTheDocument();
  });

  it("surfaces a live load failure with the server message", async () => {
    api.getPlanOptimization.mockRejectedValue(new Error("Plan not found"));
    render(<AIOptimizerWidget planId="1" />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Plan not found");
    expect(screen.queryByText(/Apply 1-Click/)).not.toBeInTheDocument();
  });

  it("surfaces an apply failure and does not claim success", async () => {
    api.getPlanOptimization.mockResolvedValue(rec(45));
    api.applyOptimization.mockRejectedValue(new Error("Recommendation expired"));
    render(<AIOptimizerWidget planId="1" />);
    await userEvent.click(await screen.findByRole("button", { name: /apply 1-click/i }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Recommendation expired");
    expect(screen.queryByText(/^Applied$/)).not.toBeInTheDocument();
    expect(screen.getByText("Server reasoning")).toBeInTheDocument();
  });

  it("shows server-confirmed state after acceptance by reloading from the server", async () => {
    api.getPlanOptimization
      .mockResolvedValueOnce(rec(45))
      .mockResolvedValueOnce(rec(30));
    api.applyOptimization.mockResolvedValue({
      status: "accepted",
      appliedAt: "2026-01-01T00:00:00Z",
    });
    render(<AIOptimizerWidget planId="1" />);
    await userEvent.click(await screen.findByRole("button", { name: /apply 1-click/i }));
    await waitFor(() => expect(screen.getByText("30% → 30%")).toBeInTheDocument());
    expect(api.applyOptimization).toHaveBeenCalledWith("rec_1");
    expect(screen.getByRole("status")).toHaveTextContent(/accepted by the server/i);
    expect(screen.getByRole("button", { name: /applied/i })).toBeDisabled();
  });

  it("surfaces a failed post-accept reload", async () => {
    api.getPlanOptimization
      .mockResolvedValueOnce(rec(45))
      .mockRejectedValueOnce(new Error("Network down"));
    api.applyOptimization.mockResolvedValue({
      status: "accepted",
      appliedAt: "2026-01-01T00:00:00Z",
    });
    render(<AIOptimizerWidget planId="1" />);
    await userEvent.click(await screen.findByRole("button", { name: /apply 1-click/i }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Network down");
  });
});
