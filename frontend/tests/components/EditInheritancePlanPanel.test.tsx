/**
 * Tests for EditInheritancePlanPanel - Issue #1265
 * Verifies real Soroban update_plan transaction flow with wallet signing.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Plan } from "@/app/lib/api/plans";

// Mock all dependencies before imports
vi.mock("framer-motion", () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

vi.mock("@/components/plans/AllocationFlowChart", () => ({
  AllocationFlowChart: () => <div data-testid="allocation-flow-chart">Flow Chart</div>,
}));

vi.mock("@/components/plans/BeneficiaryAllocationRow", () => ({
  BeneficiaryAllocationRow: () => <div data-testid="beneficiary-row">Beneficiary Row</div>,
  DEFAULT_BENEFICIARY_DRAFT: {
    address: "",
    name: "",
    allocationBps: 0,
    isFiat: false,
    fiatBank: "",
    fiatAccount: "",
    fiatCurrency: "USD",
    fiatDailyLimit: "",
    email: "",
    claimCode: "",
  },
  beneficiaryDraftToRequest: (draft: any) => ({
    address: draft.address,
    name: draft.name,
    allocation_bps: draft.allocationBps,
    fiat_anchor_info: draft.isFiat ? JSON.stringify({
      bank: draft.fiatBank,
      account: draft.fiatAccount,
      currency: draft.fiatCurrency,
      daily_limit: draft.fiatDailyLimit,
    }) : "",
  }),
  bpsToPercentageLabel: (bps: number) => `${(bps / 100).toFixed(1)}%`,
  totalAllocationBps: (beneficiaries: any[]) =>
    beneficiaries.reduce((sum, b) => sum + (b.allocationBps || 0), 0),
  validateBeneficiaryDrafts: (beneficiaries: any[]) => ({
    rowErrors: {},
    totalError: undefined,
  }),
}));

vi.mock("@/app/lib/api/geneticVerification", () => ({
  createGeneticVerificationAPI: vi.fn(),
  DEFAULT_PRIVACY_SETTINGS: {
    allowFamilyMatching: false,
    zeroKnowledgeHashing: true,
  },
}));

vi.mock("@/context/WalletContext");
vi.mock("@/store");
vi.mock("@/app/services/inheritanceContractService");
vi.mock("@/app/lib/api/plans", () => ({
  plansAPI: {
    updatePlan: vi.fn(),
  },
}));

// Now import after mocks are set up
import { EditInheritancePlanPanel } from "@/components/plans/EditInheritancePlanPanel";
import * as inheritanceContractService from "@/app/services/inheritanceContractService";
import * as plansAPIModule from "@/app/lib/api/plans";
import { useWallet } from "@/context/WalletContext";
import { useAppStore } from "@/store";

const mockPlan: Plan = {
  id: "123",
  user_id: "user-123",
  title: "Test Plan",
  description: "Test description",
  amount: 1000,
  fee: 20,
  net_amount: 980,
  status: "Active",
  is_active: true,
  owner_address: "GDEMOWALLETADDRESS12345VALIDSOROBANACCOUNT",
  token_address: "GUSDC...",
  grace_period_seconds: 2592000,
  yield_rate_bps: 500,
  earn_yield: false,
  last_ping: Math.floor(Date.now() / 1000),
  accrued_yield: 0,
  beneficiaries: [
    {
      name: "Alice",
      email: "alice@example.com",
      wallet_address: "GALICE...",
      allocation_bps: 5000,
      claim_code: "123456",
    },
    {
      name: "Bob",
      email: "bob@example.com",
      wallet_address: "GBOB...",
      allocation_bps: 5000,
      claim_code: "654321",
    },
  ],
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

const mockKit = {
  signTransaction: vi.fn(),
};

const mockWalletContext = {
  kit: mockKit,
  selectedWalletId: "freighter",
  address: "GDEMOWALLETADDRESS12345VALIDSOROBANACCOUNT",
  isConnected: true,
  connect: vi.fn(),
  disconnect: vi.fn(),
};

const mockAppStore = {
  userId: "user-123",
  setUserId: vi.fn(),
};

const mockPlanResponse: any = {
  id: "123",
  owner_address: "GDEMOWALLETADDRESS12345VALIDSOROBANACCOUNT",
  token_address: "GUSDC...",
  amount: "1000",
  grace_period: 2592000,
  grace_period_seconds: 2592000,
  earn_yield: false,
  last_ping: Math.floor(Date.now() / 1000),
  is_active: true,
  status: "Active",
  yield_rate_bps: 500,
  accrued_yield: 0,
  created_at: "2026-01-01T00:00:00Z",
  beneficiaries: [],
};

describe("EditInheritancePlanPanel - Issue #1265", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useWallet).mockReturnValue(mockWalletContext as any);
    vi.mocked(useAppStore).mockReturnValue(mockAppStore as any);
  });

  it("should reject mock wallet addresses", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    // Mock wallet with MOCK_ADDRESS
    vi.mocked(useWallet).mockReturnValue({
      ...mockWalletContext,
      address: "MOCK_ADDRESS",
    } as any);

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    // Wait a bit for validation to run
    await new Promise(resolve => setTimeout(resolve, 100));

    // Should not invoke blockchain
    expect(inheritanceContractService.invokeUpdateInheritancePlan).not.toHaveBeenCalled();
    expect(plansAPIModule.plansAPI.updatePlan).not.toHaveBeenCalled();
  });

  it("should reject GTEST wallet addresses", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    // Mock wallet with GTEST address
    vi.mocked(useWallet).mockReturnValue({
      ...mockWalletContext,
      address: "GTESTADDRESS123",
    } as any);

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    // Wait a bit for validation to run
    await new Promise(resolve => setTimeout(resolve, 100));

    // Should not invoke blockchain
    expect(inheritanceContractService.invokeUpdateInheritancePlan).not.toHaveBeenCalled();
    expect(plansAPIModule.plansAPI.updatePlan).not.toHaveBeenCalled();
  });

  it("should reject E2E mock wallet address", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    // Mock wallet with E2E test address
    vi.mocked(useWallet).mockReturnValue({
      ...mockWalletContext,
      address: "GDE2KZQ4QGJZ5Z5QW2Y4B7Y6Q5D3P9V8N7M6L5K4J3H2G1FTEST",
    } as any);

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    // Wait a bit for validation to run
    await new Promise(resolve => setTimeout(resolve, 100));

    // Should not invoke blockchain
    expect(inheritanceContractService.invokeUpdateInheritancePlan).not.toHaveBeenCalled();
    expect(plansAPIModule.plansAPI.updatePlan).not.toHaveBeenCalled();
  });

  it("should handle wallet rejection gracefully", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    // Mock wallet rejection
    vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mockRejectedValue(
      new Error("User rejected the transaction")
    );

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    // Wait for the error to be processed
    await waitFor(() => {
      expect(inheritanceContractService.invokeUpdateInheritancePlan).toHaveBeenCalled();
    }, { timeout: 3000 });

    // Verify backend sync was NOT called
    expect(plansAPIModule.plansAPI.updatePlan).not.toHaveBeenCalled();
    expect(onSaved).not.toHaveBeenCalled();
  });

  it("should complete successful update flow with blockchain and backend sync", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    // Mock successful blockchain submission
    vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mockResolvedValue({
      submission: { hash: "0xABC123", successful: true, raw: {} },
      unsignedTransactionXdr: "",
      signedTransactionXdr: "",
      explorerUrl: "",
    });

    vi.mocked(plansAPIModule.plansAPI.updatePlan).mockResolvedValue(mockPlanResponse);

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    // Wait for backend sync to be called
    await waitFor(() => {
      expect(plansAPIModule.plansAPI.updatePlan).toHaveBeenCalled();
    }, { timeout: 3000 });

    // Verify blockchain invocation was called with correct parameters
    expect(inheritanceContractService.invokeUpdateInheritancePlan).toHaveBeenCalledWith(
      expect.objectContaining({
        contractInput: expect.objectContaining({
          owner: "GDEMOWALLETADDRESS12345VALIDSOROBANACCOUNT",
          planId: "123",
          gracePeriodSeconds: 2592000,
          earnYield: false,
        }),
      })
    );

    // Verify backend sync was called AFTER blockchain success
    expect(plansAPIModule.plansAPI.updatePlan).toHaveBeenCalledWith(
      "123",
      expect.objectContaining({
        grace_period: 2592000,
        earn_yield: false,
      })
    );
  });

  it("should handle simulation failure before wallet prompt", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    // Mock simulation error
    vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mockRejectedValue(
      new inheritanceContractService.ContractSimulationError(
        "Beneficiary allocations must add up to exactly 100%."
      )
    );

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    await waitFor(() => {
      expect(inheritanceContractService.invokeUpdateInheritancePlan).toHaveBeenCalled();
    }, { timeout: 3000 });

    expect(mockKit.signTransaction).not.toHaveBeenCalled();
    expect(plansAPIModule.plansAPI.updatePlan).not.toHaveBeenCalled();
  });

  it("should handle backend sync failure after blockchain success", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mockResolvedValue({
      submission: { hash: "0xDEF456", successful: true, raw: {} },
      unsignedTransactionXdr: "",
      signedTransactionXdr: "",
      explorerUrl: "",
    });

    vi.mocked(plansAPIModule.plansAPI.updatePlan).mockRejectedValue(
      new Error("Backend API error")
    );

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    await waitFor(() => {
      expect(plansAPIModule.plansAPI.updatePlan).toHaveBeenCalled();
    }, { timeout: 3000 });

    expect(inheritanceContractService.invokeUpdateInheritancePlan).toHaveBeenCalled();
    expect(onSaved).not.toHaveBeenCalled();
  });

  it("should prevent duplicate submissions while signing", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    let resolveSign: (value: any) => void;
    const slowSignPromise = new Promise((resolve) => {
      resolveSign = resolve;
    });

    vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mockReturnValue(
      slowSignPromise as any
    );

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });

    // First click
    await user.click(saveButton);

    // Try clicking again immediately
    await user.click(saveButton);

    // Verify only one invocation
    expect(inheritanceContractService.invokeUpdateInheritancePlan).toHaveBeenCalledTimes(1);

    // Resolve the promise
    resolveSign!({
      submission: { hash: "0xABC", successful: true, raw: {} },
      unsignedTransactionXdr: "",
      signedTransactionXdr: "",
      explorerUrl: "",
    });

    vi.mocked(plansAPIModule.plansAPI.updatePlan).mockResolvedValue(mockPlanResponse);

    await waitFor(() => {
      expect(plansAPIModule.plansAPI.updatePlan).toHaveBeenCalled();
    }, { timeout: 3000 });
  });

  it("should verify submission.successful before backend sync", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    // Service throws when submission.successful is false
    vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mockRejectedValue(
      new inheritanceContractService.TransactionSubmissionError(
        "The update transaction was submitted but did not succeed on the blockchain."
      )
    );

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    await waitFor(() => {
      expect(inheritanceContractService.invokeUpdateInheritancePlan).toHaveBeenCalled();
    }, { timeout: 3000 });

    expect(plansAPIModule.plansAPI.updatePlan).not.toHaveBeenCalled();
  });

  it("should verify Horizon response successful field means ledger inclusion", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    // Mock Horizon response with successful: false (transaction not applied to ledger)
    vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mockRejectedValue(
      new inheritanceContractService.TransactionSubmissionError(
        "The update transaction was submitted but did not succeed on the blockchain.",
        { hash: "0xFAIL", successful: false }
      )
    );

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    // Wait for the contract invocation to be called and fail
    await waitFor(() => {
      expect(inheritanceContractService.invokeUpdateInheritancePlan).toHaveBeenCalled();
    }, { timeout: 3000 });

    // Give a moment for any async operations to settle
    await new Promise(resolve => setTimeout(resolve, 100));

    // Backend sync should NOT be called because transaction failed on ledger
    expect(plansAPIModule.plansAPI.updatePlan).not.toHaveBeenCalled();
    expect(onSaved).not.toHaveBeenCalled();
  });

  it("should encode contract parameters with correct 7-field BeneficiaryData tuple", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onSaved = vi.fn();

    vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mockResolvedValue({
      submission: { hash: "0xABC", successful: true, raw: {} },
      unsignedTransactionXdr: "",
      signedTransactionXdr: "",
      explorerUrl: "",
    });

    vi.mocked(plansAPIModule.plansAPI.updatePlan).mockResolvedValue(mockPlanResponse);

    render(
      <EditInheritancePlanPanel plan={mockPlan} onClose={onClose} onSaved={onSaved} />
    );

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveButton);

    await waitFor(() => {
      expect(inheritanceContractService.invokeUpdateInheritancePlan).toHaveBeenCalled();
    }, { timeout: 3000 });

    const callArgs = vi.mocked(inheritanceContractService.invokeUpdateInheritancePlan).mock
      .calls[0][0];
    const beneficiaries = callArgs.contractInput.beneficiaries;

    // Verify 7-field tuple structure
    expect(beneficiaries[0]).toMatchObject({
      fullName: "Alice",
      email: "alice@example.com",
      claimCode: 123456,
      bankAccount: "GALICE...",
      allocationBp: 5000,
      priority: 1,
      walletAddress: "GALICE...", // 7th field
    });

    expect(beneficiaries[1]).toMatchObject({
      fullName: "Bob",
      email: "bob@example.com",
      claimCode: 654321,
      bankAccount: "GBOB...",
      allocationBp: 5000,
      priority: 2,
      walletAddress: "GBOB...", // 7th field
    });
  });
});
