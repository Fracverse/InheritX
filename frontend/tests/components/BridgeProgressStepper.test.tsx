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

  it("renders explorer links for source and Stellar transaction hashes", () => {
    const steps = [
      {
        id: "approval",
        label: "Bridging transaction approval",
        status: "completed",
        sourceTxId: "0xabc123def456",
        sourceExplorerUrl: "https://sepolia.etherscan.io/tx/0xabc123def456",
      },
      {
        id: "source_execution",
        label: "Bridge execution on source chain",
        status: "completed",
        sourceTxId: "0xabc123def456",
        sourceExplorerUrl: "https://sepolia.etherscan.io/tx/0xabc123def456",
      },
      {
        id: "relayer",
        label: "Relayer processing",
        status: "completed",
        transferId: "transfer-42",
      },
      {
        id: "stellar_lock",
        label: "Final token lock on Stellar/Soroban",
        status: "completed",
        stellarTxHash: "abc123",
        stellarExplorerUrl: "https://stellar.expert/explorer/testnet/tx/abc123",
      },
    ];

    render(<BridgeProgressStepper steps={steps} />);

    const sourceLinks = screen.getAllByRole("link", {
      name: /view on sepolia etherscan/i,
    });

    expect(sourceLinks).toHaveLength(2);
    expect(sourceLinks[0]).toHaveAttribute(
      "href",
      "https://sepolia.etherscan.io/tx/0xabc123def456"
    );
    expect(screen.getByRole("link", { name: /view on stellar expert/i })).toHaveAttribute(
      "href",
      "https://stellar.expert/explorer/testnet/tx/abc123"
    );
  });
});
