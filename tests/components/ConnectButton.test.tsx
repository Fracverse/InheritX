import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConnectButton } from "@/components/ConnectButton";
import { useWallet } from "@/context/WalletContext";
import { useRouter } from "next/navigation";

// Mock dependencies
vi.mock("@/context/WalletContext");
vi.mock("next/navigation");
vi.mock("@/util/address", () => ({
  formatAddress: (address: string) => `${address.slice(0, 4)}...${address.slice(-4)}`,
}));

// Mock framer-motion to avoid animation issues in tests
vi.mock("framer-motion", () => ({
  motion: {
    button: ({ children, onClick, className, whileHover, whileTap, ...props }: any) => (
      <button onClick={onClick} className={className} {...props}>
        {children}
      </button>
    ),
    div: ({ children, className, initial, animate, exit, transition, ...props }: any) => (
      <div className={className} {...props}>
        {children}
      </div>
    ),
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe("ConnectButton", () => {
  const mockOpenModal = vi.fn();
  const mockDisconnect = vi.fn();
  const mockPush = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useRouter as any).mockReturnValue({
      push: mockPush,
    });
  });

  describe("when wallet is not connected", () => {
    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isConnected: false,
        address: null,
        openModal: mockOpenModal,
        disconnect: mockDisconnect,
      });
    });

    it("renders connect wallet button", () => {
      render(<ConnectButton />);
      expect(screen.getByText("Connect Wallet")).toBeInTheDocument();
    });

    it("calls openModal when clicked", async () => {
      const user = userEvent.setup();
      render(<ConnectButton />);

      const button = screen.getByText("Connect Wallet");
      await user.click(button);

      expect(mockOpenModal).toHaveBeenCalledTimes(1);
    });

    it("renders with correct styling classes", () => {
      render(<ConnectButton />);
      const button = screen.getByText("Connect Wallet");
      expect(button.closest("button")).toHaveClass("flex", "items-center", "gap-4");
    });
  });

  describe("when wallet is connected", () => {
    const mockAddress = "GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCD";

    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isConnected: true,
        address: mockAddress,
        openModal: mockOpenModal,
        disconnect: mockDisconnect,
      });
    });

    it("renders formatted address", () => {
      render(<ConnectButton />);
      expect(screen.getByText("GABC...ABCD")).toBeInTheDocument();
    });

    it("shows dropdown when clicked", async () => {
      const user = userEvent.setup();
      render(<ConnectButton />);

      const addressButton = screen.getByText("GABC...ABCD");
      await user.click(addressButton);

      expect(screen.getByText("Disconnect")).toBeInTheDocument();
    });

    it("calls disconnect and redirects when disconnect is clicked", async () => {
      const user = userEvent.setup();
      mockDisconnect.mockResolvedValue(undefined);

      render(<ConnectButton />);

      // Open dropdown
      const addressButton = screen.getByText("GABC...ABCD");
      await user.click(addressButton);

      // Click disconnect
      const disconnectButton = screen.getByText("Disconnect");
      await user.click(disconnectButton);

      await waitFor(() => {
        expect(mockDisconnect).toHaveBeenCalledTimes(1);
        expect(mockPush).toHaveBeenCalledWith("/");
      });
    });

    it("closes dropdown when clicking outside", async () => {
      render(
        <div>
          <ConnectButton />
          <div data-testid="outside">Outside</div>
        </div>
      );

      // Open dropdown
      const addressButton = screen.getByText("GABC...ABCD");
      fireEvent.click(addressButton);

      expect(screen.getByText("Disconnect")).toBeInTheDocument();

      // Click outside
      const outside = screen.getByTestId("outside");
      fireEvent.mouseDown(outside);

      await waitFor(() => {
        expect(screen.queryByText("Disconnect")).not.toBeInTheDocument();
      });
    });

    it("renders dashboard UI variant when targetUI is dashboard", () => {
      render(<ConnectButton targetUI="dashboard" />);

      const button = screen.getByText("GABC...ABCD").closest("button");
      expect(button).toHaveClass("rounded-l-full", "bg-[#1C252A]");
    });

    it("renders default UI variant when targetUI is not specified", () => {
      render(<ConnectButton />);

      const button = screen.getByText("GABC...ABCD").closest("button");
      expect(button).toHaveClass("rounded-s-2xl", "bg-[#0F1621]");
    });
  });

  describe("dropdown behavior", () => {
    const mockAddress = "GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCD";

    beforeEach(() => {
      (useWallet as any).mockReturnValue({
        isConnected: true,
        address: mockAddress,
        openModal: mockOpenModal,
        disconnect: mockDisconnect,
      });
    });

    it("toggles dropdown on multiple clicks", async () => {
      const user = userEvent.setup();
      render(<ConnectButton />);

      const addressButton = screen.getByText("GABC...ABCD");

      // Open
      await user.click(addressButton);
      expect(screen.getByText("Disconnect")).toBeInTheDocument();

      // Close
      await user.click(addressButton);
      await waitFor(() => {
        expect(screen.queryByText("Disconnect")).not.toBeInTheDocument();
      });

      // Open again
      await user.click(addressButton);
      expect(screen.getByText("Disconnect")).toBeInTheDocument();
    });

    it("displays LogOut icon in disconnect button", async () => {
      const user = userEvent.setup();
      render(<ConnectButton />);

      const addressButton = screen.getByText("GABC...ABCD");
      await user.click(addressButton);

      const disconnectButton = screen.getByText("Disconnect").closest("button");
      expect(disconnectButton).toBeInTheDocument();
      expect(disconnectButton).toHaveClass("text-red-400");
    });
  });

  describe("accessibility", () => {
    it("has proper button semantics when not connected", () => {
      (useWallet as any).mockReturnValue({
        isConnected: false,
        address: null,
        openModal: mockOpenModal,
        disconnect: mockDisconnect,
      });

      render(<ConnectButton />);
      const button = screen.getByRole("button", { name: /connect wallet/i });
      expect(button).toBeInTheDocument();
    });

    it("has proper button semantics when connected", () => {
      (useWallet as any).mockReturnValue({
        isConnected: true,
        address: "GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCD",
        openModal: mockOpenModal,
        disconnect: mockDisconnect,
      });

      render(<ConnectButton />);
      const buttons = screen.getAllByRole("button");
      expect(buttons.length).toBeGreaterThan(0);
    });
  });
});
