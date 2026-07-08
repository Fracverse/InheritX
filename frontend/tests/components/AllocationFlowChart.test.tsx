import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { AllocationFlowChart } from "@/components/plans/AllocationFlowChart";
import type { BeneficiaryFlow } from "@/components/plans/AllocationFlowChart";

const mockBeneficiaries: BeneficiaryFlow[] = [
  { name: "Alice", allocation_percentage: 50, isFiat: false },
  { name: "Bob", allocation_percentage: 30, isFiat: true },
  { name: "Carol", allocation_percentage: 20, isFiat: false },
];

describe("AllocationFlowChart", () => {
  it("renders the fund node with the total amount", () => {
    render(
      <AllocationFlowChart
        totalAmount={50000}
        beneficiaries={mockBeneficiaries}
      />
    );

    expect(screen.getByText("Inheritance Fund")).toBeInTheDocument();
    expect(screen.getByText("$50,000.00")).toBeInTheDocument();
  });

  it("renders all beneficiary names", () => {
    render(
      <AllocationFlowChart
        totalAmount={50000}
        beneficiaries={mockBeneficiaries}
      />
    );

    expect(screen.getByText("Alice")).toBeInTheDocument();
    expect(screen.getByText("Bob")).toBeInTheDocument();
    expect(screen.getByText("Carol")).toBeInTheDocument();
  });

  it("displays allocation percentages for each beneficiary", () => {
    render(
      <AllocationFlowChart
        totalAmount={50000}
        beneficiaries={mockBeneficiaries}
      />
    );

    expect(screen.getByText("50%")).toBeInTheDocument();
    expect(screen.getByText("30%")).toBeInTheDocument();
    expect(screen.getByText("20%")).toBeInTheDocument();
  });

  it("shows estimated payout amounts for each beneficiary", () => {
    render(
      <AllocationFlowChart
        totalAmount={50000}
        beneficiaries={mockBeneficiaries}
      />
    );

    // Alice: 50% of $50,000 = $25,000.00 -> $25.00K
    // Bob: 30% of $50,000 = $15,000.00 -> $15.00K
    // Carol: 20% of $50,000 = $10,000.00 -> $10.00K
    expect(screen.getByText("$25.00K")).toBeInTheDocument();
    expect(screen.getByText("$15.00K")).toBeInTheDocument();
    expect(screen.getByText("$10.00K")).toBeInTheDocument();
  });

  it("indicates direct token payout for non-fiat beneficiaries", () => {
    render(
      <AllocationFlowChart
        totalAmount={10000}
        beneficiaries={mockBeneficiaries}
      />
    );

    const directBadges = screen.getAllByText("DIRECT TOKEN");
    expect(directBadges.length).toBe(2);
  });

  it("indicates fiat off-ramp for fiat beneficiaries", () => {
    render(
      <AllocationFlowChart
        totalAmount={10000}
        beneficiaries={mockBeneficiaries}
      />
    );

    const fiatBadges = screen.getAllByText("FIAT OFF-RAMP");
    expect(fiatBadges.length).toBe(1);
  });

  it("renders nothing harmful with empty beneficiaries", () => {
    const { container } = render(
      <AllocationFlowChart totalAmount={0} beneficiaries={[]} />
    );

    const svg = container.querySelector("svg");
    expect(svg).toBeInTheDocument();
    expect(screen.getByText("Inheritance Fund")).toBeInTheDocument();
  });

  it("renders a single beneficiary centered", () => {
    render(
      <AllocationFlowChart
        totalAmount={5000}
        beneficiaries={[
          { name: "Sole Heir", allocation_percentage: 100, isFiat: false },
        ]}
      />
    );

    expect(screen.getByText("Sole Heir")).toBeInTheDocument();
    expect(screen.getByText("100%")).toBeInTheDocument();
    expect(screen.getByText("$5.00K")).toBeInTheDocument();
  });

  it("handles large amounts with millions format", () => {
    render(
      <AllocationFlowChart
        totalAmount={2_500_000}
        beneficiaries={[
          { name: "Heir", allocation_percentage: 100, isFiat: false },
        ]}
      />
    );

    expect(screen.getByText("$2.50M")).toBeInTheDocument();
  });

  it("handles all beneficiaries as fiat off-ramp", () => {
    const allFiat: BeneficiaryFlow[] = [
      { name: "A", allocation_percentage: 60, isFiat: true },
      { name: "B", allocation_percentage: 40, isFiat: true },
    ];

    render(
      <AllocationFlowChart totalAmount={10000} beneficiaries={allFiat} />
    );

    const fiatBadges = screen.getAllByText("FIAT OFF-RAMP");
    expect(fiatBadges.length).toBe(2);
    expect(screen.queryByText("DIRECT TOKEN")).not.toBeInTheDocument();
  });

  it("has correct aria-label on SVG", () => {
    render(
      <AllocationFlowChart
        totalAmount={50000}
        beneficiaries={mockBeneficiaries}
      />
    );

    const svg = document.querySelector("svg");
    expect(svg).toHaveAttribute(
      "aria-label",
      "Allocation flow chart: $50,000 distributed among 3 beneficiaries"
    );
  });

  it("updates when beneficiaries change", () => {
    const { rerender } = render(
      <AllocationFlowChart
        totalAmount={50000}
        beneficiaries={[
          { name: "Alice", allocation_percentage: 100, isFiat: false },
        ]}
      />
    );

    expect(screen.getByText("Alice")).toBeInTheDocument();
    expect(screen.getByText("100%")).toBeInTheDocument();

    rerender(
      <AllocationFlowChart
        totalAmount={50000}
        beneficiaries={[
          { name: "Alice", allocation_percentage: 70, isFiat: false },
          { name: "Bob", allocation_percentage: 30, isFiat: true },
        ]}
      />
    );

    expect(screen.getByText("Alice")).toBeInTheDocument();
    expect(screen.getByText("Bob")).toBeInTheDocument();
    expect(screen.getByText("70%")).toBeInTheDocument();
    expect(screen.getByText("30%")).toBeInTheDocument();
    expect(screen.getByText("$35.00K")).toBeInTheDocument();
    expect(screen.getByText("$15.00K")).toBeInTheDocument();
  });
});
