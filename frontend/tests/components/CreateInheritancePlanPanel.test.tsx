import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CreateInheritancePlanPanel } from "@/components/plans/CreateInheritancePlanPanel";

vi.mock("@/context/WalletContext", () => ({
  useWallet: vi.fn().mockReturnValue({
    isConnected: true,
    address: "GMOCK1234567890",
    openModal: vi.fn(),
    kit: { signTransaction: vi.fn().mockResolvedValue({ signedTxXdr: "signed" }) },
  }),
}));

vi.mock("@/context/CrossChainWalletContext", () => ({
  useCrossChainWallet: vi.fn().mockReturnValue({
    evmAddress: null,
    solanaAddress: null,
    isEvmConnecting: false,
    isSolanaConnecting: false,
    connectEvm: vi.fn(),
    connectSolana: vi.fn(),
  }),
}));

vi.mock("@/hooks/useBridgeDeposit", () => ({
  useBridgeDeposit: vi.fn().mockReturnValue({
    quote: {
      sourceChain: "ETH",
      sourceToken: { symbol: "USDC", name: "USD Coin", address: "0x", decimals: 6, chainId: "ETH" },
      amount: "100",
      routeId: "test-route",
      relayerFeeUsd: 2.5,
      gasFeeUsd: 8,
      destinationFeeUsd: 0.5,
      totalFeeUsd: 11,
      estimatedReceiveAmount: "89.00",
      estimatedReceiveSymbol: "USDC",
      estimatedDurationMinutes: 15,
    },
    isQuoteLoading: false,
    quoteError: null,
    steps: [],
    isBridging: false,
    bridgeError: null,
    transferId: null,
    walletType: "evm",
    sourceWalletAddress: null,
    isSourceWalletConnected: false,
    isStellarConnected: true,
    isSourceWalletConnecting: false,
    canStartBridge: false,
    connectSourceWallet: vi.fn(),
    startBridge: vi.fn(),
    resetBridge: vi.fn(),
  }),
}));

vi.mock("framer-motion", () => ({
  motion: {
    div: ({ children, className, ...rest }: any) => (
      <div className={className} {...rest}>
        {children}
      </div>
    ),
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe("CreateInheritancePlanPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders plan form and cross-chain deposit controls", () => {
    render(<CreateInheritancePlanPanel />);

    expect(screen.getByRole("heading", { name: "Create Plan" })).toBeInTheDocument();
    expect(screen.getByLabelText("Origin chain")).toBeInTheDocument();
    expect(screen.getByLabelText("Source token")).toBeInTheDocument();
    expect(screen.getByText("Bridging Fee Breakdown")).toBeInTheDocument();
  });

  it("shows chain options for EVM and Solana", async () => {
    render(<CreateInheritancePlanPanel />);
    const user = userEvent.setup();

    await user.click(screen.getByLabelText("Origin chain"));
    expect(screen.getByRole("option", { name: "Ethereum" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Solana" })).toBeInTheDocument();
  });
});
