import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { StrKey } from "@stellar/stellar-sdk";

const getLedgerEntries = vi.fn();

vi.mock("@stellar/stellar-sdk", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@stellar/stellar-sdk")>();
  return {
    ...actual,
    rpc: {
      ...actual.rpc,
      Server: vi.fn().mockImplementation(() => ({ getLedgerEntries })),
    },
  };
});

import NetworkBanner from "@/components/ui/NetworkBanner";

const INHERITANCE_ID = StrKey.encodeContract(Buffer.alloc(32, 1));
const LENDING_ID = StrKey.encodeContract(Buffer.alloc(32, 2));

beforeEach(() => {
  getLedgerEntries.mockReset();
  getLedgerEntries.mockResolvedValue({ entries: [{}] });
  vi.stubEnv("NEXT_PUBLIC_INHERITANCE_CONTRACT_ID", INHERITANCE_ID);
  vi.stubEnv("NEXT_PUBLIC_LENDING_CONTRACT_ID", LENDING_ID);
});

afterEach(() => {
  vi.unstubAllEnvs();
});

describe("NetworkBanner", () => {
  it("shows contract IDs with Stellar Expert links and no warning when deployed", async () => {
    render(<NetworkBanner />);

    expect(screen.getByTitle(INHERITANCE_ID)).toBeInTheDocument();
    expect(screen.getByTitle(LENDING_ID)).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "View Inheritance contract on Stellar Expert" })
    ).toHaveAttribute(
      "href",
      `https://stellar.expert/explorer/testnet/contract/${INHERITANCE_ID}`
    );
    await waitFor(() => expect(getLedgerEntries).toHaveBeenCalledTimes(2));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("copies a contract ID to the clipboard", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    render(<NetworkBanner />);
    fireEvent.click(screen.getByRole("button", { name: "Copy Lending contract ID" }));

    await waitFor(() => expect(writeText).toHaveBeenCalledWith(LENDING_ID));
  });

  it("warns when an env var is missing", () => {
    vi.stubEnv("NEXT_PUBLIC_LENDING_CONTRACT_ID", "");

    render(<NetworkBanner />);

    expect(screen.getByRole("alert")).toHaveTextContent(
      "NEXT_PUBLIC_LENDING_CONTRACT_ID is not set."
    );
    expect(screen.getByText(/Lending: not configured/)).toBeInTheDocument();
  });

  it("warns when an env var is not a valid contract ID", () => {
    vi.stubEnv("NEXT_PUBLIC_INHERITANCE_CONTRACT_ID", "not-a-contract");

    render(<NetworkBanner />);

    expect(screen.getByRole("alert")).toHaveTextContent(
      "NEXT_PUBLIC_INHERITANCE_CONTRACT_ID is not a valid Soroban contract ID."
    );
  });

  it("warns when the contract is not deployed on the network", async () => {
    getLedgerEntries.mockResolvedValue({ entries: [] });

    render(<NetworkBanner />);

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent(
        "NEXT_PUBLIC_INHERITANCE_CONTRACT_ID does not match a contract deployed on this network."
      )
    );
  });

  it("does not flag a mismatch when the RPC cannot be reached", async () => {
    getLedgerEntries.mockRejectedValue(new Error("network down"));

    render(<NetworkBanner />);

    await waitFor(() => expect(getLedgerEntries).toHaveBeenCalledTimes(2));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
