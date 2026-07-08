export type OriginChainId = "ETH" | "BSC" | "POL" | "SOL" | "AVA" | "ARB" | "OPT";

export type SourceWalletType = "evm" | "solana" | "stellar";

export type BridgeStepId =
  | "approval"
  | "source_execution"
  | "relayer"
  | "stellar_lock";

export type BridgeStepStatus = "pending" | "active" | "completed" | "error";

export interface BridgeStep {
  id: BridgeStepId;
  label: string;
  status: BridgeStepStatus;
  detail?: string;
}

export interface OriginChain {
  id: OriginChainId;
  name: string;
  walletType: SourceWalletType;
}

export interface SourceToken {
  symbol: string;
  name: string;
  address: string;
  decimals: number;
  chainId: OriginChainId;
}

export interface BridgeFeeBreakdown {
  relayerFeeUsd: number;
  gasFeeUsd: number;
  destinationFeeUsd: number;
  totalFeeUsd: number;
  estimatedReceiveAmount: string;
  estimatedReceiveSymbol: string;
  estimatedDurationMinutes: number;
}

export interface BridgeQuoteRequest {
  sourceChain: OriginChainId;
  sourceToken: SourceToken;
  amount: string;
  destinationStellarAddress: string;
}

export interface BridgeQuote extends BridgeFeeBreakdown {
  sourceChain: OriginChainId;
  sourceToken: SourceToken;
  amount: string;
  routeId: string;
}

export interface BridgeExecutionResult {
  sourceTxId: string;
  transferId: string;
}

export interface BridgeStatusUpdate {
  step: BridgeStepId;
  status: BridgeStepStatus;
  detail?: string;
  sourceTxId?: string;
  transferId?: string;
}

export interface EvmProvider {
  request: (args: { method: string; params?: unknown[] }) => Promise<unknown>;
}

export interface SolanaProvider {
  connect: () => Promise<{ publicKey: { toString: () => string } }>;
  signTransaction: (tx: unknown) => Promise<unknown>;
  publicKey?: { toString: () => string };
  isConnected?: boolean;
}

declare global {
  interface Window {
    ethereum?: EvmProvider;
    phantom?: { solana?: SolanaProvider };
    solana?: SolanaProvider;
  }
}
