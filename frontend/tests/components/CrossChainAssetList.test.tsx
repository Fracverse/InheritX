import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CrossChainAssetList, CrossChainAsset, AssetsByChain } from "@/components/CrossChainAssetList";

// ─── Mocks ─────────────────────────────────────────────────────────────────

vi.mock("@/components/ui/Skeleton", () => ({
  Skeleton: ({ className }: { className?: string }) => (
    <div data-testid="skeleton" className={className} />
  ),
}));

const mockAssets: AssetsByChain = {
  stellar: [
    {
      chain: "stellar",
      symbol: "USDC",
      name: "USD Coin",
      balance: "1000.50",
      decimals: 7,
      usdValue: 1000.5,
    },
    {
      chain: "stellar",
      symbol: "XLM",
      name: "Stellar Lumens",
      balance: "5000",
      decimals: 7,
      usdValue: 600,
    },
  ],
  ethereum: [
    {
      chain: "ethereum",
      contractAddress: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      symbol: "ETH",
      name: "Ethereum",
      balance: "0.5",
      decimals: 18,
      usdValue: 1750,
    },
  ],
};

function setupFetch(data: AssetsByChain | null, status = 200) {
  global.fetch = vi.fn().mockResolvedValue({
    ok: status >= 200 && status < 300,
    status,
    json: async () => data,
  });
}

// ─── Tests ─────────────────────────────────────────────────────────────────

describe("CrossChainAssetList", () => {
  const userAddress = "GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCD";

  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ── Rendering ──────────────────────────────────────────────────────────

  describe("loading state", () => {
    it("renders skeleton loaders while fetching", () => {
      // Never resolves so we stay in loading state
      global.fetch = vi.fn().mockReturnValue(new Promise(() => {}));
      render(<CrossChainAssetList userAddress={userAddress} />);
      expect(screen.getAllByTestId("skeleton").length).toBeGreaterThan(0);
      expect(screen.getByLabelText("Loading assets")).toBeInTheDocument();
    });
  });

  describe("successful data fetch", () => {
    beforeEach(() => setupFetch(mockAssets));

    it("renders chain group headers after loading", async () => {
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("Stellar")).toBeInTheDocument());
      expect(screen.getByText("Ethereum")).toBeInTheDocument();
    });

    it("displays asset symbol and name", async () => {
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("USDC")).toBeInTheDocument());
      expect(screen.getByText("USD Coin")).toBeInTheDocument();
    });

    it("shows asset balances", async () => {
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("1,000.5")).toBeInTheDocument());
    });

    it("shows USD values", async () => {
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("$1,000.50")).toBeInTheDocument());
    });

    it("groups assets by chain", async () => {
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("Stellar")).toBeInTheDocument());
      // Stellar group shows 2 assets, Ethereum shows 1
      expect(screen.getByText("(2)")).toBeInTheDocument();
      expect(screen.getByText("(1)")).toBeInTheDocument();
    });

    it("fetches from the correct API endpoint", async () => {
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(global.fetch).toHaveBeenCalledWith(
        `/api/cross-chain/assets/${userAddress}`
      ));
    });
  });

  // ── Error state ────────────────────────────────────────────────────────

  describe("error state", () => {
    it("shows error message on failed fetch", async () => {
      setupFetch(null, 500);
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() =>
        expect(screen.getByRole("alert")).toBeInTheDocument()
      );
      expect(screen.getByText("Failed to load assets")).toBeInTheDocument();
    });

    it("shows retry button on error", async () => {
      setupFetch(null, 500);
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() =>
        expect(screen.getByRole("button", { name: /retry/i })).toBeInTheDocument()
      );
    });

    it("retries fetch when retry button is clicked", async () => {
      const user = userEvent.setup();
      setupFetch(null, 500);
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() =>
        expect(screen.getByRole("button", { name: /retry/i })).toBeInTheDocument()
      );

      // Switch to success for the retry
      setupFetch(mockAssets);
      await user.click(screen.getByRole("button", { name: /retry/i }));
      await waitFor(() => expect(screen.getByText("Stellar")).toBeInTheDocument());
    });
  });

  // ── Empty state ────────────────────────────────────────────────────────

  describe("empty state", () => {
    it("shows friendly empty message when no assets found", async () => {
      setupFetch({});
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() =>
        expect(screen.getByText("No assets found")).toBeInTheDocument()
      );
    });
  });

  // ── Search filter ──────────────────────────────────────────────────────

  describe("search filter", () => {
    it("filters assets by symbol", async () => {
      setupFetch(mockAssets);
      const user = userEvent.setup();
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("USDC")).toBeInTheDocument());

      const search = screen.getByPlaceholderText("Search assets…");
      await user.type(search, "ETH");

      await waitFor(() => expect(screen.queryByText("USDC")).not.toBeInTheDocument());
      expect(screen.getByText("ETH")).toBeInTheDocument();
    });

    it("filters assets by name", async () => {
      setupFetch(mockAssets);
      const user = userEvent.setup();
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("USDC")).toBeInTheDocument());

      const search = screen.getByPlaceholderText("Search assets…");
      await user.type(search, "Lumens");

      await waitFor(() => expect(screen.getByText("Stellar Lumens")).toBeInTheDocument());
      expect(screen.queryByText("USD Coin")).not.toBeInTheDocument();
    });

    it("shows empty state message when search returns no results", async () => {
      setupFetch(mockAssets);
      const user = userEvent.setup();
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("USDC")).toBeInTheDocument());

      await user.type(screen.getByPlaceholderText("Search assets…"), "ZZZNOMATCH");

      await waitFor(() =>
        expect(screen.getByText("No assets found")).toBeInTheDocument()
      );
      expect(screen.getByText(/adjusting your search/i)).toBeInTheDocument();
    });
  });

  // ── Chain filter ───────────────────────────────────────────────────────

  describe("chain filter", () => {
    it("filters to selected chain when chain pill is clicked", async () => {
      setupFetch(mockAssets);
      const user = userEvent.setup();
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("Stellar")).toBeInTheDocument());

      // Click Stellar pill to filter to only Stellar
      const stellarPill = screen.getAllByRole("button", { name: /stellar/i })[0];
      await user.click(stellarPill);

      await waitFor(() => expect(screen.queryByText("Ethereum")).not.toBeInTheDocument());
      expect(screen.getByText("USDC")).toBeInTheDocument();
    });

    it("toggles chain filter off on second click", async () => {
      setupFetch(mockAssets);
      const user = userEvent.setup();
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("Ethereum")).toBeInTheDocument());

      const stellarPill = screen.getAllByRole("button", { name: /stellar/i })[0];
      await user.click(stellarPill); // filter on
      await user.click(stellarPill); // filter off

      await waitFor(() => expect(screen.getByText("Ethereum")).toBeInTheDocument());
    });
  });

  // ── Sort ───────────────────────────────────────────────────────────────

  describe("sort options", () => {
    it("renders sort select with correct options", async () => {
      setupFetch(mockAssets);
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => screen.getByLabelText("Sort assets"));

      const select = screen.getByLabelText("Sort assets") as HTMLSelectElement;
      expect(select).toBeInTheDocument();
      expect(select.options.length).toBe(3);
    });

    it("changes sort value on selection", async () => {
      setupFetch(mockAssets);
      const user = userEvent.setup();
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => screen.getByLabelText("Sort assets"));

      const select = screen.getByLabelText("Sort assets");
      await user.selectOptions(select, "alpha");
      expect((select as HTMLSelectElement).value).toBe("alpha");
    });
  });

  // ── Asset selection ────────────────────────────────────────────────────

  describe("asset selection", () => {
    it("does not show checkboxes when showSelection is false (default)", async () => {
      setupFetch(mockAssets);
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("USDC")).toBeInTheDocument());
      expect(screen.queryAllByRole("checkbox")).toHaveLength(0);
    });

    it("shows checkboxes when showSelection is true", async () => {
      setupFetch(mockAssets);
      render(<CrossChainAssetList userAddress={userAddress} showSelection />);
      await waitFor(() =>
        expect(screen.getAllByRole("checkbox").length).toBeGreaterThan(0)
      );
    });

    it("calls onAssetSelect with correct asset when an asset card is clicked", async () => {
      setupFetch(mockAssets);
      const onSelect = vi.fn();
      const user = userEvent.setup();

      render(
        <CrossChainAssetList
          userAddress={userAddress}
          showSelection
          onAssetSelect={onSelect}
        />
      );
      await waitFor(() => expect(screen.getByText("USDC")).toBeInTheDocument());

      const usdcCard = screen.getByText("USDC").closest("[role='checkbox']")!;
      await user.click(usdcCard);

      expect(onSelect).toHaveBeenCalledTimes(1);
      expect(onSelect).toHaveBeenCalledWith(
        expect.objectContaining({ symbol: "USDC", chain: "stellar" })
      );
    });

    it("renders pre-selected assets as checked", async () => {
      setupFetch(mockAssets);
      const selected: CrossChainAsset[] = [mockAssets.stellar[0]];

      render(
        <CrossChainAssetList
          userAddress={userAddress}
          showSelection
          selectedAssets={selected}
        />
      );
      await waitFor(() => expect(screen.getByText("USDC")).toBeInTheDocument());

      const usdcCard = screen.getByText("USDC").closest("[role='checkbox']")!;
      expect(usdcCard).toHaveAttribute("aria-checked", "true");
    });
  });

  // ── Chain collapse ─────────────────────────────────────────────────────

  describe("chain group collapse", () => {
    it("collapses a chain group when its header is clicked", async () => {
      setupFetch(mockAssets);
      const user = userEvent.setup();
      render(<CrossChainAssetList userAddress={userAddress} />);
      await waitFor(() => expect(screen.getByText("USDC")).toBeInTheDocument());

      // Stellar group toggle button
      const stellarToggle = screen.getByRole("button", { name: /stellar/i });
      await user.click(stellarToggle);

      await waitFor(() => expect(screen.queryByText("USDC")).not.toBeInTheDocument());
    });
  });
});
