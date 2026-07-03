"use client";

import { useCallback, useEffect, useState } from "react";
import {
  applyBridgeStatusUpdate,
  createInitialBridgeSteps,
  executeBridgeTransfer,
  confirmStellarLock,
  getBridgeQuote,
  walletTypeForChain,
} from "@/lib/bridge";
import type {
  BridgeQuote,
  BridgeStep,
  OriginChainId,
  SourceToken,
} from "@/lib/bridge/types";
import { useCrossChainWallet } from "@/context/CrossChainWalletContext";
import { useWallet } from "@/context/WalletContext";

interface UseBridgeDepositOptions {
  sourceChain: OriginChainId;
  sourceToken: SourceToken;
  amount: string;
}

export function useBridgeDeposit({
  sourceChain,
  sourceToken,
  amount,
}: UseBridgeDepositOptions) {
  const { address: stellarAddress, kit, isConnected: isStellarConnected } =
    useWallet();
  const {
    evmAddress,
    solanaAddress,
    connectEvm,
    connectSolana,
    isEvmConnecting,
    isSolanaConnecting,
  } = useCrossChainWallet();

  const [quote, setQuote] = useState<BridgeQuote | null>(null);
  const [isQuoteLoading, setIsQuoteLoading] = useState(false);
  const [quoteError, setQuoteError] = useState<string | null>(null);
  const [steps, setSteps] = useState<BridgeStep[]>(createInitialBridgeSteps);
  const [isBridging, setIsBridging] = useState(false);
  const [bridgeError, setBridgeError] = useState<string | null>(null);
  const [transferId, setTransferId] = useState<string | null>(null);

  const walletType = walletTypeForChain(sourceChain);
  const sourceWalletAddress =
    walletType === "solana" ? solanaAddress : evmAddress;

  const isSourceWalletConnected = Boolean(sourceWalletAddress);
  const canStartBridge =
    isStellarConnected &&
    isSourceWalletConnected &&
    Boolean(quote) &&
    Number.parseFloat(amount) > 0 &&
    !isBridging;

  useEffect(() => {
    const parsedAmount = Number.parseFloat(amount);
    if (!parsedAmount || parsedAmount <= 0 || !stellarAddress) {
      setQuote(null);
      setQuoteError(null);
      return;
    }

    let cancelled = false;
    setIsQuoteLoading(true);
    setQuoteError(null);

    getBridgeQuote({
      sourceChain,
      sourceToken,
      amount,
      destinationStellarAddress: stellarAddress,
    })
      .then((result) => {
        if (!cancelled) setQuote(result);
      })
      .catch((error) => {
        if (!cancelled) {
          setQuote(null);
          setQuoteError(
            error instanceof Error ? error.message : "Failed to fetch bridge quote."
          );
        }
      })
      .finally(() => {
        if (!cancelled) setIsQuoteLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [sourceChain, sourceToken, amount, stellarAddress]);

  const connectSourceWallet = useCallback(async () => {
    if (walletType === "solana") {
      return connectSolana();
    }
    return connectEvm(sourceChain);
  }, [walletType, connectSolana, connectEvm, sourceChain]);

  const handleStatusUpdate = useCallback(
    (update: Parameters<typeof applyBridgeStatusUpdate>[1]) => {
      setSteps((current) => applyBridgeStatusUpdate(current, update));
    },
    []
  );

  const startBridge = useCallback(async () => {
    if (!quote || !stellarAddress || !sourceWalletAddress) return;

    setBridgeError(null);
    setIsBridging(true);
    setSteps(createInitialBridgeSteps());

    try {
      const result = await executeBridgeTransfer({
        quote,
        sourceWalletAddress,
        destinationStellarAddress: stellarAddress,
        onStatusUpdate: handleStatusUpdate,
      });

      setTransferId(result.transferId);

      if (kit) {
        await kit.signTransaction(
          `unsigned-xdr::lock_deposit::${result.transferId}::${Date.now()}`
        );
      }

      await confirmStellarLock(handleStatusUpdate, result.transferId);
    } catch (error) {
      setBridgeError(
        error instanceof Error ? error.message : "Cross-chain deposit failed."
      );
    } finally {
      setIsBridging(false);
    }
  }, [
    quote,
    stellarAddress,
    sourceWalletAddress,
    handleStatusUpdate,
    kit,
  ]);

  const resetBridge = useCallback(() => {
    setSteps(createInitialBridgeSteps());
    setBridgeError(null);
    setTransferId(null);
  }, []);

  return {
    quote,
    isQuoteLoading,
    quoteError,
    steps,
    isBridging,
    bridgeError,
    transferId,
    walletType,
    sourceWalletAddress,
    isSourceWalletConnected,
    isStellarConnected,
    isSourceWalletConnecting: isEvmConnecting || isSolanaConnecting,
    canStartBridge,
    connectSourceWallet,
    startBridge,
    resetBridge,
  };
}
