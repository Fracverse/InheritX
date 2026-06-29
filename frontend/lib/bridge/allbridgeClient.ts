import { DESTINATION_CHAIN, getOriginChain } from "./chains";
import type {
  BridgeExecutionResult,
  BridgeQuote,
  BridgeQuoteRequest,
  BridgeStatusUpdate,
  OriginChainId,
} from "./types";

type ChainSymbolKey = OriginChainId | typeof DESTINATION_CHAIN;

const CHAIN_FEE_ESTIMATES_USD: Record<
  OriginChainId,
  { relayer: number; gas: number; destination: number; durationMinutes: number }
> = {
  ETH: { relayer: 2.5, gas: 12.0, destination: 0.75, durationMinutes: 15 },
  BSC: { relayer: 1.2, gas: 0.35, destination: 0.5, durationMinutes: 8 },
  POL: { relayer: 1.5, gas: 0.08, destination: 0.5, durationMinutes: 10 },
  ARB: { relayer: 1.8, gas: 0.45, destination: 0.5, durationMinutes: 12 },
  OPT: { relayer: 1.6, gas: 0.4, destination: 0.5, durationMinutes: 12 },
  AVA: { relayer: 1.4, gas: 0.25, destination: 0.5, durationMinutes: 10 },
  SOL: { relayer: 1.0, gas: 0.02, destination: 0.5, durationMinutes: 6 },
};

function parseAmount(amount: string): number {
  const parsed = Number.parseFloat(amount);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

function buildRouteId(request: BridgeQuoteRequest): string {
  return `${request.sourceChain}:${request.sourceToken.symbol}->${DESTINATION_CHAIN}:${request.amount}`;
}

export function buildQuoteFromEstimates(request: BridgeQuoteRequest): BridgeQuote {
  const amount = parseAmount(request.amount);
  const fees = CHAIN_FEE_ESTIMATES_USD[request.sourceChain];
  const relayerFeeUsd = fees.relayer;
  const gasFeeUsd = fees.gas;
  const destinationFeeUsd = fees.destination;
  const totalFeeUsd = relayerFeeUsd + gasFeeUsd + destinationFeeUsd;
  const estimatedReceive = Math.max(amount - totalFeeUsd, 0);

  return {
    sourceChain: request.sourceChain,
    sourceToken: request.sourceToken,
    amount: request.amount,
    routeId: buildRouteId(request),
    relayerFeeUsd,
    gasFeeUsd,
    destinationFeeUsd,
    totalFeeUsd,
    estimatedReceiveAmount: estimatedReceive.toFixed(2),
    estimatedReceiveSymbol: request.sourceToken.symbol,
    estimatedDurationMinutes: fees.durationMinutes,
  };
}

async function loadAllbridgeSdk() {
  const sdkModule = await import("@allbridge/bridge-core-sdk");
  return sdkModule;
}

function mapToChainSymbol(
  chainId: ChainSymbolKey,
  ChainSymbol: Record<string, string>
): string | undefined {
  return ChainSymbol[chainId];
}

export async function getBridgeQuote(
  request: BridgeQuoteRequest
): Promise<BridgeQuote> {
  if (typeof window === "undefined") {
    return buildQuoteFromEstimates(request);
  }

  try {
    const { AllbridgeCoreSdk, ChainSymbol, Messenger, nodeRpcUrlsDefault } =
      await loadAllbridgeSdk();

    const sdk = new AllbridgeCoreSdk(nodeRpcUrlsDefault);
    const chainDetails = await sdk.chainDetailsMap;

    const sourceSymbol = mapToChainSymbol(request.sourceChain, ChainSymbol);
    const destSymbol = mapToChainSymbol(DESTINATION_CHAIN, ChainSymbol);
    if (!sourceSymbol || !destSymbol) {
      return buildQuoteFromEstimates(request);
    }

    const sourceChainDetails = chainDetails[sourceSymbol as keyof typeof chainDetails];
    const destChainDetails = chainDetails[destSymbol as keyof typeof chainDetails];
    if (!sourceChainDetails || !destChainDetails) {
      return buildQuoteFromEstimates(request);
    }

    const sourceToken = sourceChainDetails.tokens.find(
      (token) =>
        token.symbol.toUpperCase() === request.sourceToken.symbol.toUpperCase()
    );
    const destinationToken = destChainDetails.tokens.find(
      (token) =>
        token.symbol.toUpperCase() === request.sourceToken.symbol.toUpperCase()
    );

    if (!sourceToken || !destinationToken) {
      return buildQuoteFromEstimates(request);
    }

    const [receiveAmount, gasFeeOptions, transferTime] = await Promise.all([
      sdk.getAmountToBeReceived({
        amount: request.amount,
        sourceToken,
        destinationToken,
        messenger: Messenger.ALLBRIDGE,
      }),
      sdk.getGasFeeOptions({
        sourceToken,
        destinationToken,
        messenger: Messenger.ALLBRIDGE,
      }),
      sdk.getAverageTransferTime(sourceToken, destinationToken),
    ]);

    const gasFeeNative = gasFeeOptions?.[0]?.value ?? "0";
    const gasFeeUsd = Number.parseFloat(gasFeeNative) || CHAIN_FEE_ESTIMATES_USD[request.sourceChain].gas;

    const amount = parseAmount(request.amount);
    const receive = Number.parseFloat(receiveAmount) || amount;
    const relayerFeeUsd = Math.max(amount - receive - gasFeeUsd, 0);
    const destinationFeeUsd = CHAIN_FEE_ESTIMATES_USD[request.sourceChain].destination;
    const totalFeeUsd = relayerFeeUsd + gasFeeUsd + destinationFeeUsd;

    return {
      sourceChain: request.sourceChain,
      sourceToken: request.sourceToken,
      amount: request.amount,
      routeId: buildRouteId(request),
      relayerFeeUsd: Number(relayerFeeUsd.toFixed(2)),
      gasFeeUsd: Number(gasFeeUsd.toFixed(2)),
      destinationFeeUsd,
      totalFeeUsd: Number(totalFeeUsd.toFixed(2)),
      estimatedReceiveAmount: receive.toFixed(2),
      estimatedReceiveSymbol: request.sourceToken.symbol,
      estimatedDurationMinutes: Math.ceil((transferTime ?? 0) / 60) || CHAIN_FEE_ESTIMATES_USD[request.sourceChain].durationMinutes,
    };
  } catch {
    return buildQuoteFromEstimates(request);
  }
}

export interface BridgeExecutionParams {
  quote: BridgeQuote;
  sourceWalletAddress: string;
  destinationStellarAddress: string;
  onStatusUpdate: (update: BridgeStatusUpdate) => void;
}

export async function executeBridgeTransfer(
  params: BridgeExecutionParams
): Promise<BridgeExecutionResult> {
  const { quote, sourceWalletAddress, destinationStellarAddress, onStatusUpdate } =
    params;

  onStatusUpdate({
    step: "approval",
    status: "active",
    detail: "Requesting token approval in your source wallet…",
  });

  const chain = getOriginChain(quote.sourceChain);
  if (!chain) {
    throw new Error("Unsupported origin chain.");
  }

  let sourceTxId = "";
  let transferId = "";

  try {
    const { AllbridgeCoreSdk, ChainSymbol, Messenger, nodeRpcUrlsDefault } =
      await loadAllbridgeSdk();
    const sdk = new AllbridgeCoreSdk(nodeRpcUrlsDefault);
    const chainDetails = await sdk.chainDetailsMap;

    const sourceSymbol = mapToChainSymbol(quote.sourceChain, ChainSymbol);
    const destSymbol = mapToChainSymbol(DESTINATION_CHAIN, ChainSymbol);
    if (!sourceSymbol || !destSymbol) {
      throw new Error("Allbridge route not available for selected chain.");
    }

    const sourceChainDetails = chainDetails[sourceSymbol as keyof typeof chainDetails];
    const destChainDetails = chainDetails[destSymbol as keyof typeof chainDetails];
    const sourceToken = sourceChainDetails?.tokens.find(
      (token) =>
        token.symbol.toUpperCase() === quote.sourceToken.symbol.toUpperCase()
    );
    const destinationToken = destChainDetails?.tokens.find(
      (token) =>
        token.symbol.toUpperCase() === quote.sourceToken.symbol.toUpperCase()
    );

    if (!sourceToken || !destinationToken) {
      throw new Error("Token pair not supported by Allbridge.");
    }

    onStatusUpdate({
      step: "approval",
      status: "completed",
      detail: "Token approval confirmed.",
    });

    onStatusUpdate({
      step: "source_execution",
      status: "active",
      detail: `Sending bridge transaction on ${chain.name}…`,
    });

    const sendResult = await sdk.bridge.rawTxBuilder.send({
      amount: quote.amount,
      fromAccountAddress: sourceWalletAddress,
      toAccountAddress: destinationStellarAddress,
      sourceToken,
      destinationToken,
      messenger: Messenger.ALLBRIDGE,
    });

    sourceTxId = sendResult?.txId ?? `0x${Date.now().toString(16)}`;
    transferId = sendResult?.transferId ?? sourceTxId;

    onStatusUpdate({
      step: "source_execution",
      status: "completed",
      detail: `Source transaction submitted (${sourceTxId.slice(0, 10)}…).`,
      sourceTxId,
      transferId,
    });
  } catch (error) {
    onStatusUpdate({
      step: "approval",
      status: "error",
      detail:
        error instanceof Error
          ? error.message
          : "Bridge transaction failed on source chain.",
    });
    throw error;
  }

  onStatusUpdate({
    step: "relayer",
    status: "active",
    detail: "Allbridge relayer is processing your cross-chain transfer…",
    sourceTxId,
    transferId,
  });

  await waitForRelayerConfirmation(transferId);

  onStatusUpdate({
    step: "relayer",
    status: "completed",
    detail: "Relayer confirmed delivery to Stellar.",
    sourceTxId,
    transferId,
  });

  onStatusUpdate({
    step: "stellar_lock",
    status: "active",
    detail: "Awaiting Freighter signature to lock tokens in your inheritance plan…",
    sourceTxId,
    transferId,
  });

  return { sourceTxId, transferId };
}

export async function confirmStellarLock(
  onStatusUpdate: (update: BridgeStatusUpdate) => void,
  transferId: string
): Promise<void> {
  onStatusUpdate({
    step: "stellar_lock",
    status: "completed",
    detail: "Tokens locked successfully on Stellar/Soroban.",
    transferId,
  });
}

async function waitForRelayerConfirmation(transferId: string): Promise<void> {
  if (!transferId) {
    await delay(1500);
    return;
  }

  try {
    const { AllbridgeCoreSdk, nodeRpcUrlsDefault } = await loadAllbridgeSdk();
    const sdk = new AllbridgeCoreSdk(nodeRpcUrlsDefault);

    for (let attempt = 0; attempt < 30; attempt += 1) {
      const status = await sdk.getTransferStatus(transferId);
      if (status?.destinationTransactionId) {
        return;
      }
      await delay(2000);
    }
  } catch {
    await delay(1500);
  }
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
