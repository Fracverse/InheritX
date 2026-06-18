import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  AIOptimizationDashboard,
  OptimizationScenario,
} from "@/components/AIOptimizationDashboard";
import type {
  AssetAllocation,
  OptimizationRecommendation,
} from "@/app/lib/api/aiOptimization";

// ── Mocks ──────────────────────────────────────────────────────────────────────

vi.mock("framer-motion", () => ({
  motion: {
    div: ({ children, className, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
      <div className={className} {...props}>{children}</div>
    ),
  },
  AnimatePresence: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

// ── Fixtures ───────────────────────────────────────────────────────────────────

const MOCK_ALLOCATIONS: AssetAllocation[] = [
  {
    assetSymbol: "XLM",
    chain: "Stellar",
    currentPercentage: 45,
    recommendedPercentage: 30,
    adjustmentReason: "Reduce concentration risk",
    expectedImpact: "Lower volatility exposure",
  },
  {
    assetSymbol: "USDC",
    chain: "Stellar",
    currentPercentage: 25,
    recommendedPercentage: 35,
    adjustmentReason: "Increase stable allocation",
    expectedImpact: "Improved capital preservation",
  },
  {
    assetSymbol: "BTC",
    chain: "Bitcoin",
    currentPercentage: 20,
    recommendedPercentage: 22,
    adjustmentReason: "Long-term store of value",
    expectedImpact: "Enhanced 10-year value projection",
  },
  {
    assetSymbol: "ETH",
    chain: "Ethereum",
    currentPercentage: 10,
    recommendedPercentage: 13,
    adjustmentReason: "DeFi yield-generating assets",
    expectedImpact: "Additional yield ~4.2% APY",
  },
];

const MOCK_RECOMMENDATION: OptimizationRecommendation = {
  id: "rec_001",
  planId: 42,
  recommendedAllocations: MOCK_ALLOCATIONS,
  confidenceScore: 87,
  expectedReturn: 14.3,
  riskScore: 42,
  reasoning:
    "AI analysis reveals an overweight position in XLM relative to risk tolerance.",
  generatedAt: "2025-06-18T10:00:00.000Z",
  projectedOutcomes: {
    estimatedValue1Year: 114_300,
    estimatedValue5Year: 197_600,
    estimatedValue10Year: 389_200,
    riskMetrics: {
      volatility: 18.4,
      sharpeRatio: 1.34,
      maxDrawdown: 28.7,
      valueAtRisk: 8.2,
    },
  },
};

const DEFAULT_PROPS = {
  inheritancePlanId: 42,
  currentAllocations: MOCK_ALLOCATIONS,
  optimizationRecommendations: MOCK_RECOMMENDATION,
  onAcceptRecommendation: vi.fn(),
  onRejectRecommendation: vi.fn(),
  onCustomizeRecommendation: vi.fn(),
};

// ── Helper ────────────────────────────────────────────────────────────────────

function renderDashboard(overrides = {}) {
  return render(<AIOptimizationDashboard {...DEFAULT_PROPS} {...overrides} />);
}

/** Find a tab button by exact label text (avoids matching action buttons with similar names) */
function getTab(label: string) {
  return screen.getAllByRole("button").find((b) => b.textContent?.trim() === label)!;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("AIOptimizationDashboard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ── Rendering ──

  describe("initial render", () => {
    it("renders the dashboard heading", () => {
      renderDashboard();
      expect(screen.getByText("AI Optimization")).toBeInTheDocument();
    });

    it("shows the plan ID", () => {
      renderDashboard();
      expect(screen.getByText(/Plan #42/)).toBeInTheDocument();
    });

    it("renders all four tabs", () => {
      renderDashboard();
      // Use exact tab labels to avoid matching action panel buttons
      const tabs = screen.getAllByRole("button").filter((b) =>
        ["Overview", "Allocation", "Projections", "Customize"].includes(b.textContent?.trim() ?? "")
      );
      expect(tabs.length).toBe(4);
    });

    it("shows confidence score in summary cards", () => {
      renderDashboard();
      expect(screen.getByText("87")).toBeInTheDocument();
    });

    it("shows expected annual return", () => {
      renderDashboard();
      expect(screen.getByText("+14.3%")).toBeInTheDocument();
    });

    it("shows risk score", () => {
      renderDashboard();
      expect(screen.getByText("42")).toBeInTheDocument();
    });

    it("shows projected 10-year value", () => {
      renderDashboard();
      expect(screen.getByText("$389K")).toBeInTheDocument();
    });

    it("renders action buttons", () => {
      renderDashboard();
      expect(screen.getByRole("button", { name: /Accept All Recommendations/i })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /Customize Allocations/i })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /Reject/i })).toBeInTheDocument();
    });
  });

  // ── Overview Tab ──

  describe("overview tab", () => {
    it("shows AI Reasoning section", () => {
      renderDashboard();
      expect(screen.getByText("AI Reasoning")).toBeInTheDocument();
    });

    it("expands reasoning panel on click", async () => {
      const user = userEvent.setup();
      renderDashboard();
      const reasoningBtn = screen.getByText("AI Reasoning").closest("button")!;
      await user.click(reasoningBtn);
      expect(screen.getByText(/AI analysis reveals an overweight position/)).toBeInTheDocument();
    });

    it("shows risk metrics after expanding reasoning", async () => {
      const user = userEvent.setup();
      renderDashboard();
      const reasoningBtn = screen.getByText("AI Reasoning").closest("button")!;
      await user.click(reasoningBtn);
      expect(screen.getByText("Sharpe Ratio")).toBeInTheDocument();
      expect(screen.getByText("1.34")).toBeInTheDocument();
    });

    it("shows key changes section", () => {
      renderDashboard();
      expect(screen.getByText("Key Changes")).toBeInTheDocument();
    });

    it("shows best practices section", () => {
      renderDashboard();
      expect(screen.getByText("Best Practices")).toBeInTheDocument();
    });
  });

  // ── Allocation Tab ──

  describe("allocation tab", () => {
    it("switches to allocation tab", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Allocation"));
      expect(screen.getByText("Portfolio Allocation Comparison")).toBeInTheDocument();
    });

    it("shows asset-by-asset comparison chart", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Allocation"));
      expect(screen.getByText("Asset-by-Asset Comparison")).toBeInTheDocument();
    });

    it("shows detailed allocation table", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Allocation"));
      expect(screen.getByText("Detailed Allocation Table")).toBeInTheDocument();
    });

    it("shows asset symbols in the table", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Allocation"));
      const table = screen.getByRole("table");
      expect(within(table).getAllByText("XLM").length).toBeGreaterThan(0);
      expect(within(table).getAllByText("BTC").length).toBeGreaterThan(0);
    });

    it("shows current and recommended percentages in table", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Allocation"));
      expect(screen.getAllByText("45.0%").length).toBeGreaterThan(0);
      expect(screen.getAllByText("30.0%").length).toBeGreaterThan(0);
    });
  });

  // ── Projections Tab ──

  describe("projections tab", () => {
    it("switches to projections tab", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Projections"));
      expect(screen.getByText("Scenario Analysis")).toBeInTheDocument();
    });

    it("shows scenario selector with all options", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Projections"));
      // Use getAllByText since scenario labels also appear in SVG chart legend
      expect(screen.getAllByText("Conservative").length).toBeGreaterThan(0);
      expect(screen.getAllByText("Moderate").length).toBeGreaterThan(0);
      expect(screen.getAllByText("Aggressive").length).toBeGreaterThan(0);
      expect(screen.getAllByText("Custom").length).toBeGreaterThan(0);
    });

    it("shows projection milestone cards", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Projections"));
      expect(screen.getByText("1 Year Projection")).toBeInTheDocument();
      expect(screen.getByText("5 Years Projection")).toBeInTheDocument();
      expect(screen.getByText("10 Years Projection")).toBeInTheDocument();
    });

    it("shows risk metrics section", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Projections"));
      expect(screen.getByText("Risk Metrics")).toBeInTheDocument();
      expect(screen.getAllByText("Volatility").length).toBeGreaterThan(0);
      expect(screen.getAllByText("Max Drawdown").length).toBeGreaterThan(0);
    });

    it("switches scenario on click", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Projections"));
      // Click the Aggressive scenario button (the `<p>` label is inside a button)
      const aggLabels = screen.getAllByText("Aggressive");
      const aggBtn = aggLabels.find((el) => el.closest("button"))!.closest("button")!;
      await user.click(aggBtn);
      // After switching, the milestone cards should show the scenario adjustment
      expect(screen.getAllByText(/\+40% scenario adj\./).length).toBeGreaterThan(0);
    });
  });

  // ── Customize Tab ──

  describe("customize tab", () => {
    it("switches to customize tab", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Customize"));
      expect(screen.getByText("Custom Allocation")).toBeInTheDocument();
    });

    it("shows sliders for each asset", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Customize"));
      const sliders = screen.getAllByRole("slider");
      expect(sliders.length).toBe(MOCK_ALLOCATIONS.length);
    });

    it("shows what-if analysis section", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Customize"));
      expect(screen.getByText("What-If Analysis")).toBeInTheDocument();
    });

    it("shows apply custom allocations button", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Customize"));
      expect(screen.getByRole("button", { name: /Apply Custom Allocations/i })).toBeInTheDocument();
    });

    it("calls onCustomizeRecommendation when apply is clicked", async () => {
      const onCustomize = vi.fn();
      const user = userEvent.setup();
      render(<AIOptimizationDashboard {...DEFAULT_PROPS} onCustomizeRecommendation={onCustomize} />);
      await user.click(getTab("Customize"));
      await user.click(screen.getByRole("button", { name: /Apply Custom Allocations/i }));
      expect(onCustomize).toHaveBeenCalledTimes(1);
      expect(onCustomize).toHaveBeenCalledWith(expect.arrayContaining([
        expect.objectContaining({ assetSymbol: "XLM" }),
      ]));
    });
  });

  // ── Accept flow ──

  describe("accept recommendation flow", () => {
    it("shows accept confirmation modal on click", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /Accept All Recommendations/i }));
      expect(screen.getByText("Accept AI Recommendation")).toBeInTheDocument();
    });

    it("shows recommendation summary in accept modal", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /Accept All Recommendations/i }));
      // "+14.3%" appears in both the summary card and the modal — confirm it's present at all
      expect(screen.getAllByText("+14.3%").length).toBeGreaterThan(0);
      // The modal-specific content is unique
      expect(screen.getByText("Expected Annual Return")).toBeInTheDocument();
    });

    it("calls onAcceptRecommendation when apply is confirmed", async () => {
      const onAccept = vi.fn();
      const user = userEvent.setup();
      render(<AIOptimizationDashboard {...DEFAULT_PROPS} onAcceptRecommendation={onAccept} />);
      await user.click(screen.getByRole("button", { name: /Accept All Recommendations/i }));
      await user.click(screen.getByRole("button", { name: /Apply Recommendations/i }));
      expect(onAccept).toHaveBeenCalledTimes(1);
      expect(onAccept).toHaveBeenCalledWith(MOCK_RECOMMENDATION);
    });

    it("cancels accept modal on cancel click", async () => {
      const onAccept = vi.fn();
      const user = userEvent.setup();
      render(<AIOptimizationDashboard {...DEFAULT_PROPS} onAcceptRecommendation={onAccept} />);
      await user.click(screen.getByRole("button", { name: /Accept All Recommendations/i }));
      await user.click(screen.getByRole("button", { name: /^Cancel$/i }));
      expect(onAccept).not.toHaveBeenCalled();
      expect(screen.queryByText("Accept AI Recommendation")).not.toBeInTheDocument();
    });

    it("closes accept modal when backdrop is clicked", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /Accept All Recommendations/i }));
      expect(screen.getByText("Accept AI Recommendation")).toBeInTheDocument();
      const backdrop = document.querySelector(".fixed.inset-0");
      if (backdrop) await user.click(backdrop);
      expect(screen.queryByText("Accept AI Recommendation")).not.toBeInTheDocument();
    });
  });

  // ── Reject flow ──

  describe("reject recommendation flow", () => {
    it("shows reject modal on reject button click", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /^Reject$/i }));
      expect(screen.getByText("Reject Recommendation")).toBeInTheDocument();
    });

    it("shows preset rejection reasons", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /^Reject$/i }));
      expect(screen.getByText("I prefer higher risk tolerance")).toBeInTheDocument();
      expect(screen.getByText("I want to maintain current allocation")).toBeInTheDocument();
    });

    it("disables confirm button when no reason selected", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /^Reject$/i }));
      const confirmBtn = screen.getByRole("button", { name: /Confirm Rejection/i });
      expect(confirmBtn).toBeDisabled();
    });

    it("enables confirm after selecting a preset reason", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /^Reject$/i }));
      await user.click(screen.getByText("I prefer higher risk tolerance"));
      const confirmBtn = screen.getByRole("button", { name: /Confirm Rejection/i });
      expect(confirmBtn).not.toBeDisabled();
    });

    it("calls onRejectRecommendation with selected reason", async () => {
      const onReject = vi.fn();
      const user = userEvent.setup();
      render(<AIOptimizationDashboard {...DEFAULT_PROPS} onRejectRecommendation={onReject} />);
      await user.click(screen.getByRole("button", { name: /^Reject$/i }));
      await user.click(screen.getByText("I prefer higher risk tolerance"));
      await user.click(screen.getByRole("button", { name: /Confirm Rejection/i }));
      expect(onReject).toHaveBeenCalledWith("I prefer higher risk tolerance");
    });

    it("accepts custom text reason", async () => {
      const onReject = vi.fn();
      const user = userEvent.setup();
      render(<AIOptimizationDashboard {...DEFAULT_PROPS} onRejectRecommendation={onReject} />);
      await user.click(screen.getByRole("button", { name: /^Reject$/i }));
      const textarea = screen.getByPlaceholderText(/describe your specific reason/i);
      await user.clear(textarea);
      await user.type(textarea, "I have a different strategy");
      await user.click(screen.getByRole("button", { name: /Confirm Rejection/i }));
      expect(onReject).toHaveBeenCalledWith("I have a different strategy");
    });

    it("closes reject modal on cancel", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /^Reject$/i }));
      await user.click(screen.getByRole("button", { name: /^Cancel$/i }));
      expect(screen.queryByText("Reject Recommendation")).not.toBeInTheDocument();
    });
  });

  // ── Tooltip ──

  describe("tooltips", () => {
    it("shows tooltip on hover over info icon", async () => {
      const user = userEvent.setup();
      renderDashboard();
      const infoIcons = document.querySelectorAll('[class*="cursor-help"]');
      if (infoIcons.length > 0) {
        await user.hover(infoIcons[0] as HTMLElement);
        expect(document.querySelector('[class*="absolute"]')).toBeInTheDocument();
      }
    });
  });

  // ── Navigation via action buttons ──

  describe("action button navigation", () => {
    it("navigates to customize tab when Customize Allocations is clicked", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(screen.getByRole("button", { name: /Customize Allocations/i }));
      expect(screen.getByText("Custom Allocation")).toBeInTheDocument();
    });
  });

  // ── Accessibility ──

  describe("accessibility", () => {
    it("has proper heading structure", () => {
      renderDashboard();
      const heading = screen.getByRole("heading", { level: 1 });
      expect(heading).toBeInTheDocument();
    });

    it("sliders have aria-labels", async () => {
      const user = userEvent.setup();
      renderDashboard();
      await user.click(getTab("Customize"));
      const sliders = screen.getAllByRole("slider");
      sliders.forEach((slider) => {
        expect(slider).toHaveAttribute("aria-label");
      });
    });

    it("refresh button has aria-label", () => {
      renderDashboard();
      expect(screen.getByRole("button", { name: /Refresh recommendations/i })).toBeInTheDocument();
    });
  });

  // ── Responsive summary cards ──

  describe("summary cards", () => {
    it("shows 1-year projected value", () => {
      renderDashboard();
      expect(screen.getByText("$114K")).toBeInTheDocument();
    });

    it("shows 5-year projected value", () => {
      renderDashboard();
      expect(screen.getByText("$198K")).toBeInTheDocument();
    });

    it("shows Medium Risk badge for risk score 42", () => {
      renderDashboard();
      expect(screen.getByText("Medium Risk")).toBeInTheDocument();
    });
  });

  // ── Different prop variations ──

  describe("prop variations", () => {
    it("shows High Risk badge for risk score 75", () => {
      render(
        <AIOptimizationDashboard
          {...DEFAULT_PROPS}
          optimizationRecommendations={{ ...MOCK_RECOMMENDATION, riskScore: 75 }}
        />,
      );
      expect(screen.getByText("High Risk")).toBeInTheDocument();
    });

    it("shows Low Risk badge for risk score 20", () => {
      render(
        <AIOptimizationDashboard
          {...DEFAULT_PROPS}
          optimizationRecommendations={{ ...MOCK_RECOMMENDATION, riskScore: 20 }}
        />,
      );
      expect(screen.getByText("Low Risk")).toBeInTheDocument();
    });

    it("renders with different plan ID", () => {
      render(<AIOptimizationDashboard {...DEFAULT_PROPS} inheritancePlanId={99} />);
      expect(screen.getByText(/Plan #99/)).toBeInTheDocument();
    });
  });
});
