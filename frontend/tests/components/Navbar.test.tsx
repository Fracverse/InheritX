import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Navbar from "@/app/components/Navbar";
import { useWallet } from "@/context/WalletContext";
import { useAccountBalance } from "@/hooks/useAccountBalance";
import { STELLAR_FAUCET_URL } from "@/app/lib/stellar/network";

// Mock dependencies
vi.mock("@/context/WalletContext");
vi.mock("@/hooks/useAccountBalance");
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn() }),
}));
vi.mock("next/image", () => ({
  __esModule: true,
  default: ({ src, alt, ...props }: any) => <img src={src} alt={alt} {...props} />,
}));
vi.mock("@/components/ConnectButton", () => ({
  ConnectButton: () => <button data-testid="connect-btn">Connect Button</button>,
}));

describe("Navbar Component with Asset Balance & Gas Warning", () => {
  const mockRefresh = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("when wallet is disconnected", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isConnected: false,
        address: null,
      });
      (useAccountBalance as any).mockReturnValue({
        xlmBalance: null,
        formattedBalance: "-- XLM",
        isLoading: false,
        isRefreshing: false,
        isLowGas: false,
        isUnfunded: false,
        error: null,
        refresh: mockRefresh,
      });
    });

    it("renders navigation links and connect button without balance badge", () => {
      render(<Navbar />);

      expect(screen.getByRole("link", { name: /home/i })).toBeInTheDocument();
      expect(screen.getByRole("link", { name: /how it works/i })).toBeInTheDocument();
      expect(screen.getByTestId("connect-btn")).toBeInTheDocument();

      // No balance badge or warning should be displayed when disconnected
      expect(screen.queryByTestId("account-balance-badge")).not.toBeInTheDocument();
      expect(screen.queryByTestId("low-gas-warning")).not.toBeInTheDocument();
    });
  });

  describe("when wallet is connected with sufficient gas (>= 5 XLM)", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isConnected: true,
        address: "GBZV2IQM4N5K6L7J8H9G0F1E2D3C4B5A6Z7Y8X9W",
      });
      (useAccountBalance as any).mockReturnValue({
        xlmBalance: 25.5,
        formattedBalance: "25.50 XLM",
        isLoading: false,
        isRefreshing: false,
        isLowGas: false,
        isUnfunded: false,
        error: null,
        refresh: mockRefresh,
      });
    });

    it("renders the standard account balance badge with cyan accent", () => {
      render(<Navbar />);

      const balanceBadge = screen.getByTestId("account-balance-badge");
      expect(balanceBadge).toBeInTheDocument();
      expect(balanceBadge).toHaveTextContent("25.50 XLM");

      // No low gas warning should be present
      expect(screen.queryByTestId("low-gas-warning")).not.toBeInTheDocument();
      expect(screen.queryByTestId("faucet-link")).not.toBeInTheDocument();
    });

    it("triggers manual refresh when refresh button is clicked", async () => {
      const user = userEvent.setup();
      render(<Navbar />);

      const refreshBtn = screen.getByTestId("refresh-balance-btn");
      await user.click(refreshBtn);

      expect(mockRefresh).toHaveBeenCalledTimes(1);
    });
  });

  describe("when wallet is connected with low gas (< 5 XLM)", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isConnected: true,
        address: "GBZV2IQM4N5K6L7J8H9G0F1E2D3C4B5A6Z7Y8X9W",
      });
      (useAccountBalance as any).mockReturnValue({
        xlmBalance: 2.15,
        formattedBalance: "2.15 XLM",
        isLoading: false,
        isRefreshing: false,
        isLowGas: true,
        isUnfunded: false,
        error: null,
        refresh: mockRefresh,
      });
    });

    it("renders the orange warning badge with alert role", () => {
      render(<Navbar />);

      const warningBadge = screen.getByTestId("low-gas-warning");
      expect(warningBadge).toBeInTheDocument();
      expect(warningBadge).toHaveAttribute("role", "alert");
      expect(warningBadge).toHaveTextContent("2.15 XLM");
      expect(warningBadge).toHaveTextContent(/low gas/i);

      // Normal badge should not be displayed
      expect(screen.queryByTestId("account-balance-badge")).not.toBeInTheDocument();
    });

    it("renders direct link to testnet faucet with target='_blank' and rel='noopener noreferrer'", () => {
      render(<Navbar />);

      const faucetLink = screen.getByTestId("faucet-link");
      expect(faucetLink).toBeInTheDocument();
      expect(faucetLink).toHaveAttribute("href", STELLAR_FAUCET_URL);
      expect(faucetLink).toHaveAttribute("target", "_blank");
      expect(faucetLink).toHaveAttribute("rel", "noopener noreferrer");
      expect(faucetLink).toHaveTextContent("Faucet");
    });
  });

  describe("when account is unfunded on testnet (0 XLM)", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isConnected: true,
        address: "GNEWUNFUNDEDACCOUNT1234567890",
      });
      (useAccountBalance as any).mockReturnValue({
        xlmBalance: 0,
        formattedBalance: "0.00 XLM",
        isLoading: false,
        isRefreshing: false,
        isLowGas: true,
        isUnfunded: true,
        error: null,
        refresh: mockRefresh,
      });
    });

    it("renders the low gas warning badge with 0.00 XLM and faucet link", () => {
      render(<Navbar />);

      const warningBadge = screen.getByTestId("low-gas-warning");
      expect(warningBadge).toBeInTheDocument();
      expect(warningBadge).toHaveTextContent("0.00 XLM");

      const faucetLink = screen.getByTestId("faucet-link");
      expect(faucetLink).toBeInTheDocument();
    });
  });

  describe("mobile navigation drawer", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isConnected: true,
        address: "GBZV2IQM4N5K6L7J8H9G0F1E2D3C4B5A6Z7Y8X9W",
      });
      (useAccountBalance as any).mockReturnValue({
        xlmBalance: 1.5,
        formattedBalance: "1.50 XLM",
        isLoading: false,
        isRefreshing: false,
        isLowGas: true,
        isUnfunded: false,
        error: null,
        refresh: mockRefresh,
      });
    });

    it("renders balance badge and faucet link inside mobile drawer when opened", async () => {
      const user = userEvent.setup();
      render(<Navbar />);

      const toggleBtn = screen.getByRole("button", { name: /open menu/i });
      await user.click(toggleBtn);

      // Both desktop and mobile badges should be present
      const warningBadges = screen.getAllByTestId("low-gas-warning");
      expect(warningBadges.length).toBeGreaterThanOrEqual(1);

      const faucetLinks = screen.getAllByTestId("faucet-link");
      expect(faucetLinks.length).toBeGreaterThanOrEqual(1);
    });
  });
});
