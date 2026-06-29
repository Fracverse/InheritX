"use client";

import React, {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
} from "react";
import {
  connectEvmWallet,
  connectSolanaWallet,
  switchEvmChain,
} from "@/lib/bridge/wallets";
import type { OriginChainId } from "@/lib/bridge/types";

interface CrossChainWalletContextType {
  evmAddress: string | null;
  solanaAddress: string | null;
  isEvmConnecting: boolean;
  isSolanaConnecting: boolean;
  evmError: string | null;
  solanaError: string | null;
  connectEvm: (chainId?: OriginChainId) => Promise<string>;
  connectSolana: () => Promise<string>;
  disconnectEvm: () => void;
  disconnectSolana: () => void;
}

const CrossChainWalletContext = createContext<
  CrossChainWalletContextType | undefined
>(undefined);

export function CrossChainWalletProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const [evmAddress, setEvmAddress] = useState<string | null>(null);
  const [solanaAddress, setSolanaAddress] = useState<string | null>(null);
  const [isEvmConnecting, setIsEvmConnecting] = useState(false);
  const [isSolanaConnecting, setIsSolanaConnecting] = useState(false);
  const [evmError, setEvmError] = useState<string | null>(null);
  const [solanaError, setSolanaError] = useState<string | null>(null);

  const connectEvm = useCallback(async (chainId?: OriginChainId) => {
    setIsEvmConnecting(true);
    setEvmError(null);
    try {
      if (chainId) {
        await switchEvmChain(chainId);
      }
      const address = await connectEvmWallet();
      setEvmAddress(address);
      return address;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to connect EVM wallet.";
      setEvmError(message);
      throw error;
    } finally {
      setIsEvmConnecting(false);
    }
  }, []);

  const connectSolana = useCallback(async () => {
    setIsSolanaConnecting(true);
    setSolanaError(null);
    try {
      const address = await connectSolanaWallet();
      setSolanaAddress(address);
      return address;
    } catch (error) {
      const message =
        error instanceof Error
          ? error.message
          : "Failed to connect Solana wallet.";
      setSolanaError(message);
      throw error;
    } finally {
      setIsSolanaConnecting(false);
    }
  }, []);

  const disconnectEvm = useCallback(() => {
    setEvmAddress(null);
    setEvmError(null);
  }, []);

  const disconnectSolana = useCallback(() => {
    setSolanaAddress(null);
    setSolanaError(null);
  }, []);

  const value = useMemo(
    () => ({
      evmAddress,
      solanaAddress,
      isEvmConnecting,
      isSolanaConnecting,
      evmError,
      solanaError,
      connectEvm,
      connectSolana,
      disconnectEvm,
      disconnectSolana,
    }),
    [
      evmAddress,
      solanaAddress,
      isEvmConnecting,
      isSolanaConnecting,
      evmError,
      solanaError,
      connectEvm,
      connectSolana,
      disconnectEvm,
      disconnectSolana,
    ]
  );

  return (
    <CrossChainWalletContext.Provider value={value}>
      {children}
    </CrossChainWalletContext.Provider>
  );
}

export function useCrossChainWallet() {
  const context = useContext(CrossChainWalletContext);
  if (!context) {
    throw new Error(
      "useCrossChainWallet must be used within CrossChainWalletProvider"
    );
  }
  return context;
}
