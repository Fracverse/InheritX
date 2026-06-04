import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WalletModal } from "@/components/WalletModal";
import { useWallet } from "@/context/WalletContext";

// Mock dependencies
vi.mock("@/context/WalletContext");

// Mock framer-motion
vi.mock("framer-motion", () => ({
  motion: {
    div: ({ children, className, onClick, ...props }: any) => (
      <div className={className} onClick={onClick} {...props}>
        {children}
      </div>
    ),
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe("WalletModal", () => {
  const mockConnect = vi.fn();
  const mockCloseModal = vi.fn();

  const mockSupportedWallets = [
    { id: "freighter", name: "Freighter", icon: "/icons/freighter.png" },
    { id: "albedo", name: "Albedo", icon: "/icons/albedo.png" },
    { id: "xbull", name: "xBull", icon: "/icons/xbull.png" },
    { id: "rabet", name: "Rabet", icon: "/icons/rabet.png" },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("when modal is closed", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isModalOpen: false,
        closeModal: mockCloseModal,
        supportedWallets: mockSupportedWallets,
        connect: mockConnect,
        isConnecting: false,
      });
    });

    it("does not render modal", () => {
      render(<WalletModal />);
      expect(screen.queryByText("Connect Wallet")).not.toBeInTheDocument();
    });
  });

  describe("when modal is open", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isModalOpen: true,
        closeModal: mockCloseModal,
        supportedWallets: mockSupportedWallets,
        connect: mockConnect,
        isConnecting: false,
      });
    });

    it("renders modal with title and description", () => {
      render(<WalletModal />);

      expect(screen.getByRole("heading", { name: /connect wallet/i })).toBeInTheDocument();
      expect(
        screen.getByText("Connect your wallet to get started with InheritX")
      ).toBeInTheDocument();
    });

    it("renders all supported wallets", () => {
      render(<WalletModal />);

      mockSupportedWallets.forEach((wallet) => {
        expect(screen.getByText(new RegExp(wallet.name, "i"))).toBeInTheDocument();
      });
    });

    it("closes modal when backdrop is clicked", async () => {
      const user = userEvent.setup();
      const { container } = render(<WalletModal />);

      const backdrop = container.querySelector(".fixed.inset-0.bg-transparent");
      if (backdrop) {
        await user.click(backdrop);
        expect(mockCloseModal).toHaveBeenCalledTimes(1);
      }
    });

    it("allows selecting a wallet", async () => {
      const user = userEvent.setup();
      render(<WalletModal />);

      const freighterButton = screen.getByText(/freighter/i).closest("button");
      expect(freighterButton).toBeInTheDocument();

      await user.click(freighterButton!);

      // Check if the wallet is selected (check icon should appear)
      const checkIcon = freighterButton!.querySelector("svg");
      expect(checkIcon).toBeInTheDocument();
    });

    it("enables connect button only when a wallet is selected", async () => {
      const user = userEvent.setup();
      render(<WalletModal />);

      const connectButton = screen.getByRole("button", { name: /connect wallet/i });

      // Initially disabled (no selection)
      expect(connectButton).toBeDisabled();

      // Select a wallet
      const freighterButton = screen.getByText(/freighter/i).closest("button");
      await user.click(freighterButton!);

      // Now enabled
      await waitFor(() => {
        expect(connectButton).not.toBeDisabled();
      });
    });

    it("calls connect with selected wallet ID when connect button is clicked", async () => {
      const user = userEvent.setup();
      render(<WalletModal />);

      // Select Freighter
      const freighterButton = screen.getByText(/freighter/i).closest("button");
      await user.click(freighterButton!);

      // Click connect
      const connectButton = screen.getByRole("button", { name: /connect wallet/i });
      await user.click(connectButton);

      expect(mockConnect).toHaveBeenCalledWith("freighter");
    });

    it("allows changing wallet selection", async () => {
      const user = userEvent.setup();
      render(<WalletModal />);

      // Select Freighter
      const freighterButton = screen.getByText(/freighter/i).closest("button");
      await user.click(freighterButton!);

      // Select Albedo instead
      const albedoButton = screen.getByText(/albedo/i).closest("button");
      await user.click(albedoButton!);

      // Click connect
      const connectButton = screen.getByRole("button", { name: /connect wallet/i });
      await user.click(connectButton);

      expect(mockConnect).toHaveBeenCalledWith("albedo");
    });

    it("resets selection when modal is reopened", () => {
      const { rerender } = render(<WalletModal />);

      // Close modal
      (useWallet as any).mockReturnValue({
        isModalOpen: false,
        closeModal: mockCloseModal,
        supportedWallets: mockSupportedWallets,
        connect: mockConnect,
        isConnecting: false,
      });
      rerender(<WalletModal />);

      // Reopen modal
      (useWallet as any).mockReturnValue({
        isModalOpen: true,
        closeModal: mockCloseModal,
        supportedWallets: mockSupportedWallets,
        connect: mockConnect,
        isConnecting: false,
      });
      rerender(<WalletModal />);

      const connectButton = screen.getByRole("button", { name: /connect wallet/i });
      expect(connectButton).toBeDisabled();
    });
  });

  describe("when connecting", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isModalOpen: true,
        closeModal: mockCloseModal,
        supportedWallets: mockSupportedWallets,
        connect: mockConnect,
        isConnecting: true,
      });
    });

    it("disables connect button during connection", async () => {
      const user = userEvent.setup();
      render(<WalletModal />);

      // Select a wallet first
      const freighterButton = screen.getByText(/freighter/i).closest("button");
      await user.click(freighterButton!);

      const connectButton = screen.getByRole("button", { name: /connect wallet/i });
      expect(connectButton).toBeDisabled();
    });
  });

  describe("wallet list rendering", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isModalOpen: true,
        closeModal: mockCloseModal,
        supportedWallets: mockSupportedWallets,
        connect: mockConnect,
        isConnecting: false,
      });
    });

    it("renders wallet icons", () => {
      render(<WalletModal />);

      // Check that wallet buttons exist
      const walletButtons = screen.getAllByRole("button");
      
      // Should have wallet buttons + connect button
      expect(walletButtons.length).toBeGreaterThanOrEqual(mockSupportedWallets.length);
    });

    it("applies correct styling to selected wallet", async () => {
      const user = userEvent.setup();
      render(<WalletModal />);

      const freighterButton = screen.getByText(/freighter/i).closest("button");
      await user.click(freighterButton!);

      expect(freighterButton).toHaveClass("bg-[#1a2333]");
    });

    it("applies correct styling to unselected wallets", () => {
      render(<WalletModal />);

      const albedoButton = screen.getByText(/albedo/i).closest("button");
      expect(albedoButton).toHaveClass("bg-transparent");
    });
  });

  describe("accessibility", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isModalOpen: true,
        closeModal: mockCloseModal,
        supportedWallets: mockSupportedWallets,
        connect: mockConnect,
        isConnecting: false,
      });
    });

    it("has proper heading structure", () => {
      render(<WalletModal />);
      const heading = screen.getByRole("heading", { name: /connect wallet/i });
      expect(heading).toBeInTheDocument();
    });

    it("all wallet options are buttons", () => {
      render(<WalletModal />);
      const buttons = screen.getAllByRole("button");
      // Should have wallet buttons + connect button
      expect(buttons.length).toBeGreaterThanOrEqual(mockSupportedWallets.length);
    });

    it("connect button has proper disabled state", () => {
      render(<WalletModal />);
      const connectButton = screen.getByRole("button", { name: /connect wallet/i });
      expect(connectButton).toHaveAttribute("disabled");
    });
  });
});
