import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";
import { InactivityTimerCard } from "@/components/plans/InactivityTimerCard";
import { plansAPI } from "@/app/lib/api/plans";
import * as WalletContext from "@/context/WalletContext";

// Mock dependencies
vi.mock("@/app/lib/api/plans");
vi.mock("@/context/WalletContext");

describe("InactivityTimerCard", () => {
  const mockPlanId = "plan-123";

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders timer component with initial state", async () => {
    const mockInactivityStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(
      mockInactivityStatus
    );

    vi.mocked(WalletContext.useWallet).mockReturnValue({
      kit: null,
      selectedWalletId: null,
      address: null,
      isConnected: false,
      isConnecting: false,
      connect: vi.fn(),
      disconnect: vi.fn(),
      openModal: vi.fn(),
      closeModal: vi.fn(),
      isModalOpen: false,
      supportedWallets: [],
    });

    render(<InactivityTimerCard planId={mockPlanId} />);

    await waitFor(() => {
      expect(screen.getByText(/Inactivity Timer/i)).toBeInTheDocument();
      expect(screen.getByText(/ACTIVE/i)).toBeInTheDocument();
    });
  });

  it("displays warning badge when timer is low (< 24 hours)", async () => {
    const now = Date.now();
    const twelveHoursAgo = now - 12 * 60 * 60 * 1000;

    const mockInactivityStatus = {
      last_ping_timestamp: twelveHoursAgo,
      inactivity_period_days: 1,
      days_until_claimable: 0,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(
      mockInactivityStatus
    );

    vi.mocked(WalletContext.useWallet).mockReturnValue({
      kit: null,
      selectedWalletId: null,
      address: null,
      isConnected: false,
      isConnecting: false,
      connect: vi.fn(),
      disconnect: vi.fn(),
      openModal: vi.fn(),
      closeModal: vi.fn(),
      isModalOpen: false,
      supportedWallets: [],
    });

    render(<InactivityTimerCard planId={mockPlanId} />);

    await waitFor(() => {
      expect(screen.getByText(/LOW TIME/i)).toBeInTheDocument();
    });
  });

  it("displays claimable badge when timer reaches zero", async () => {
    const now = Date.now();
    const agoTime = now - 200 * 24 * 60 * 60 * 1000; // 200 days ago

    const mockInactivityStatus = {
      last_ping_timestamp: agoTime,
      inactivity_period_days: 180,
      days_until_claimable: 0,
      is_claimable: true,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(
      mockInactivityStatus
    );

    vi.mocked(WalletContext.useWallet).mockReturnValue({
      kit: null,
      selectedWalletId: null,
      address: null,
      isConnected: false,
      isConnecting: false,
      connect: vi.fn(),
      disconnect: vi.fn(),
      openModal: vi.fn(),
      closeModal: vi.fn(),
      isModalOpen: false,
      supportedWallets: [],
    });

    render(<InactivityTimerCard planId={mockPlanId} />);

    await waitFor(() => {
      expect(screen.getByText(/CLAIMABLE/i)).toBeInTheDocument();
      expect(screen.getByText(/Plan is claimable/i)).toBeInTheDocument();
    });
  });

  it("ping button is disabled when wallet is not connected", async () => {
    const mockInactivityStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(
      mockInactivityStatus
    );

    vi.mocked(WalletContext.useWallet).mockReturnValue({
      kit: null,
      selectedWalletId: null,
      address: null,
      isConnected: false,
      isConnecting: false,
      connect: vi.fn(),
      disconnect: vi.fn(),
      openModal: vi.fn(),
      closeModal: vi.fn(),
      isModalOpen: false,
      supportedWallets: [],
    });

    render(<InactivityTimerCard planId={mockPlanId} />);

    await waitFor(() => {
      const button = screen.getByRole("button", {
        name: /Ping \(Verify Alive\)/i,
      });
      expect(button).toBeDisabled();
    });
  });

  it("calls ping API when button is clicked with wallet connected", async () => {
    const mockInactivityStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(
      mockInactivityStatus
    );
    vi.mocked(plansAPI.pingKeepAlive).mockResolvedValue({
      id: mockPlanId,
      user_id: "user-123",
      title: "Test Plan",
      fee: 100,
      net_amount: 1000,
      status: "active",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    });

    const mockKit = {
      signTransaction: vi
        .fn()
        .mockResolvedValue({ signedTxXdr: "signed-xdr-123" }),
    };

    vi.mocked(WalletContext.useWallet).mockReturnValue({
      kit: mockKit as any,
      selectedWalletId: "freighter",
      address: "G123",
      isConnected: true,
      isConnecting: false,
      connect: vi.fn(),
      disconnect: vi.fn(),
      openModal: vi.fn(),
      closeModal: vi.fn(),
      isModalOpen: false,
      supportedWallets: [],
    });

    render(<InactivityTimerCard planId={mockPlanId} />);

    await waitFor(() => {
      const button = screen.getByRole("button", {
        name: /Ping \(Verify Alive\)/i,
      });
      expect(button).not.toBeDisabled();
    });

    const button = screen.getByRole("button", {
      name: /Ping \(Verify Alive\)/i,
    });
    fireEvent.click(button);

    await waitFor(() => {
      expect(vi.mocked(plansAPI.pingKeepAlive)).toHaveBeenCalledWith(
        mockPlanId,
        "signed-xdr-123"
      );
    });
  });

  it("calls onPingSuccess callback after successful ping", async () => {
    const mockInactivityStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(
      mockInactivityStatus
    );
    vi.mocked(plansAPI.pingKeepAlive).mockResolvedValue({
      id: mockPlanId,
      user_id: "user-123",
      title: "Test Plan",
      fee: 100,
      net_amount: 1000,
      status: "active",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    });

    const mockKit = {
      signTransaction: vi
        .fn()
        .mockResolvedValue({ signedTxXdr: "signed-xdr-123" }),
    };

    const onPingSuccess = vi.fn();

    vi.mocked(WalletContext.useWallet).mockReturnValue({
      kit: mockKit as any,
      selectedWalletId: "freighter",
      address: "G123",
      isConnected: true,
      isConnecting: false,
      connect: vi.fn(),
      disconnect: vi.fn(),
      openModal: vi.fn(),
      closeModal: vi.fn(),
      isModalOpen: false,
      supportedWallets: [],
    });

    render(
      <InactivityTimerCard
        planId={mockPlanId}
        onPingSuccess={onPingSuccess}
      />
    );

    await waitFor(() => {
      const button = screen.getByRole("button", {
        name: /Ping \(Verify Alive\)/i,
      });
      expect(button).not.toBeDisabled();
    });

    const button = screen.getByRole("button", {
      name: /Ping \(Verify Alive\)/i,
    });
    fireEvent.click(button);

    await waitFor(() => {
      expect(onPingSuccess).toHaveBeenCalled();
    });
  });

  it("calls onPingError callback on ping failure", async () => {
    const mockInactivityStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(
      mockInactivityStatus
    );

    const pingError = new Error("Ping failed");
    vi.mocked(plansAPI.pingKeepAlive).mockRejectedValue(pingError);

    const mockKit = {
      signTransaction: vi
        .fn()
        .mockResolvedValue({ signedTxXdr: "signed-xdr-123" }),
    };

    const onPingError = vi.fn();

    vi.mocked(WalletContext.useWallet).mockReturnValue({
      kit: mockKit as any,
      selectedWalletId: "freighter",
      address: "G123",
      isConnected: true,
      isConnecting: false,
      connect: vi.fn(),
      disconnect: vi.fn(),
      openModal: vi.fn(),
      closeModal: vi.fn(),
      isModalOpen: false,
      supportedWallets: [],
    });

    render(
      <InactivityTimerCard planId={mockPlanId} onPingError={onPingError} />
    );

    await waitFor(() => {
      const button = screen.getByRole("button", {
        name: /Ping \(Verify Alive\)/i,
      });
      expect(button).not.toBeDisabled();
    });

    const button = screen.getByRole("button", {
      name: /Ping \(Verify Alive\)/i,
    });
    fireEvent.click(button);

    await waitFor(() => {
      expect(onPingError).toHaveBeenCalledWith(pingError);
    });
  });
});
