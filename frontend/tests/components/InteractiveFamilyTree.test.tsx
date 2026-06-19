import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import InteractiveFamilyTree, {
  FamilyMemberNode,
  GeneticConnectionEdge,
  InheritanceFlowEdge,
} from "@/components/InteractiveFamilyTree";

const MEMBERS: FamilyMemberNode[] = [
  {
    id: "owner",
    name: "Plan Owner",
    relationshipLabel: "Self",
    relationshipDegree: 0,
    geneticSimilarity: 100,
    verificationStatus: "verified",
    confidenceLevel: 96,
    healthRiskScore: 64,
    healthConditions: ["Cardiovascular risk"],
  },
  {
    id: "child",
    name: "Child",
    relationshipLabel: "Child",
    relationshipDegree: 1,
    geneticSimilarity: 50,
    verificationStatus: "verified",
    confidenceLevel: 88,
    healthRiskScore: 40,
    healthConditions: [],
  },
  {
    id: "spouse",
    name: "Spouse",
    relationshipLabel: "Partner",
    relationshipDegree: 1,
    geneticSimilarity: 4,
    verificationStatus: "verified",
    confidenceLevel: 90,
    healthRiskScore: 20,
    healthConditions: [],
  },
];

const CONNECTIONS: GeneticConnectionEdge[] = [
  {
    id: "g1",
    sourceId: "owner",
    targetId: "child",
    relationshipType: "parent",
    relationshipDegree: 1,
    similarityStrength: 50,
    confidenceLevel: 90,
    verified: true,
  },
  {
    id: "g2",
    sourceId: "owner",
    targetId: "spouse",
    relationshipType: "spouse",
    relationshipDegree: 1,
    similarityStrength: 5,
    confidenceLevel: 95,
    verified: true,
  },
];

const FLOWS: InheritanceFlowEdge[] = [
  {
    id: "flow-1",
    sourceId: "owner",
    targetId: "child",
    assetType: "Trust",
    amount: 50000,
    currency: "USD",
    status: "planned",
  },
];

describe("InteractiveFamilyTree", () => {
  it("renders core controls and details panel", () => {
    render(
      <InteractiveFamilyTree
        members={MEMBERS}
        geneticConnections={CONNECTIONS}
        inheritanceFlows={FLOWS}
      />,
    );

    expect(screen.getByText("Interactive Family Tree")).toBeInTheDocument();
    expect(screen.getByLabelText("Layout algorithm")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Zoom in" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Reset view" })).toBeInTheDocument();
    expect(screen.getByText("Member Details")).toBeInTheDocument();
  });

  it("updates details when selecting a different member", () => {
    render(
      <InteractiveFamilyTree
        members={MEMBERS}
        geneticConnections={CONNECTIONS}
        inheritanceFlows={FLOWS}
      />,
    );

    const childNode = screen.getByTestId("member-node-child");
    fireEvent.click(childNode);

    expect(screen.getAllByText("Child").length).toBeGreaterThan(0);
    expect(screen.getByText(/Risk score:/)).toBeInTheDocument();
  });

  it("switches layout algorithm", () => {
    render(
      <InteractiveFamilyTree
        members={MEMBERS}
        geneticConnections={CONNECTIONS}
        inheritanceFlows={FLOWS}
      />,
    );

    const layoutSelect = screen.getByLabelText("Layout algorithm");
    fireEvent.change(layoutSelect, { target: { value: "radial" } });
    expect((layoutSelect as HTMLSelectElement).value).toBe("radial");
  });

  it("toggles genetic and inheritance visibility controls", () => {
    render(
      <InteractiveFamilyTree
        members={MEMBERS}
        geneticConnections={CONNECTIONS}
        inheritanceFlows={FLOWS}
      />,
    );

    const geneticsToggle = screen.getByLabelText("Toggle genetic links");
    const inheritanceToggle = screen.getByLabelText("Toggle inheritance flows");

    expect(geneticsToggle).toBeChecked();
    expect(inheritanceToggle).toBeChecked();

    fireEvent.click(geneticsToggle);
    fireEvent.click(inheritanceToggle);

    expect(geneticsToggle).not.toBeChecked();
    expect(inheritanceToggle).not.toBeChecked();
  });

  it("renders larger trees without failing", () => {
    const largeMembers: FamilyMemberNode[] = Array.from({ length: 140 }, (_, i) => ({
      id: `m-${i}`,
      name: `Member ${i}`,
      relationshipLabel: i === 0 ? "Root" : "Relative",
      relationshipDegree: i === 0 ? 0 : 1,
      geneticSimilarity: i === 0 ? 100 : 45,
      verificationStatus: "verified",
      confidenceLevel: 80,
      healthRiskScore: (i * 3) % 100,
      healthConditions: [],
    }));
    const largeConnections: GeneticConnectionEdge[] = Array.from(
      { length: 139 },
      (_, i) => ({
        id: `c-${i}`,
        sourceId: `m-${Math.floor(i / 2)}`,
        targetId: `m-${i + 1}`,
        relationshipType: "parent",
        relationshipDegree: 1,
        similarityStrength: 50,
        confidenceLevel: 80,
        verified: true,
      }),
    );

    render(
      <InteractiveFamilyTree
        members={largeMembers}
        geneticConnections={largeConnections}
        inheritanceFlows={[]}
      />,
    );

    expect(screen.getByLabelText("Family tree graph")).toBeInTheDocument();
    expect(screen.getByTestId("member-node-m-0")).toBeInTheDocument();
  });
});
