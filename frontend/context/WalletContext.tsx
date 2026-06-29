"use client";

import React, { createContext, useContext, useEffect, useState, useCallback } from "react";
import {
  StellarWalletsKit,
  WalletNetwork,
  allowAllModules,
} from "@creit.tech/stellar-wallets-kit";
import { useRouter } from "next/navigation";

interface WalletContextType {
  connect: (moduleId: string) => Promise<void>;
  disconnect: () => Promise<void>;
  signTransaction: (xdr: string) => Promise<{ signedTxXdr: string }>;
  address: string | null;
  isConnected: boolean;
  isConnecting: boolean;
  selectedWalletId: string | null;
  kit: StellarWalletsKit | null;
  openModal: () => void;
  closeModal: () => void;
  isModalOpen: boolean;
  supportedWallets: { id: string; name: string; icon: string }[];
}

const WalletContext = createContext<WalletContextType | undefined>(undefined);

const E2E_MOCK_WALLET_ADDRESS =
  "GDE2KZQ4QGJZ5Z5QW2Y4B7Y6Q5D3P9V8N7M6L5K4J3H2G1FTEST";

const WALLET_ICONS: Record<string, string> = {
  freighter: "/icons/freighter.png",
  albedo: "/icons/albedo.png",
  xbull: "/icons/xbull.png",
  rabet: "/icons/rabet.png",
  lobstr: "/icons/lobstr.png",
  hana: "/icons/hana.png",
};

export const useWallet = () => {
  const context = useContext(WalletContext);
  if (!context) {
    throw new Error("useWallet must be used within a WalletProvider");
  }
  return context;
};

export const WalletProvider = ({ children }: { children: React.ReactNode }) => {
  const [address, setAddress] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [selectedWalletId, setSelectedWalletId] = useState<string | null>(null);
  const [kit, setKit] = useState<StellarWalletsKit | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const router = useRouter();

  useEffect(() => {
    const walletKit = new StellarWalletsKit({
      network: WalletNetwork.TESTNET,
      selectedWalletId: "freighter",
      modules: allowAllModules(),
    });
    setKit(walletKit);

    const savedAddress = localStorage.getItem("inheritx_wallet_address");
    const savedWalletId = localStorage.getItem("inheritx_wallet_id");

    if (savedAddress && savedWalletId) {
      setAddress(savedAddress);
      setSelectedWalletId(savedWalletId);
    }
  }, []);

  useEffect(() => {
    const handleAddressChange = (e: Event) => {
      const customEvent = e as CustomEvent<{ address: string }>;
      if (customEvent.detail?.address) {
        const newAddress = customEvent.detail.address;
        setAddress(newAddress);
        localStorage.setItem("inheritx_wallet_address", newAddress);
      }
    };

    window.addEventListener("stellar-wallet:address-change", handleAddressChange);
    return () => {
      window.removeEventListener("stellar-wallet:address-change", handleAddressChange);
    };
  }, []);

  const supportedWallets = [
    { id: "freighter", name: "Freighter", icon: WALLET_ICONS.freighter },
    { id: "albedo", name: "Albedo", icon: WALLET_ICONS.albedo },
    { id: "xbull", name: "xBull", icon: WALLET_ICONS.xbull },
    { id: "rabet", name: "Rabet", icon: WALLET_ICONS.rabet },
    { id: "lobstr", name: "Lobstr", icon: WALLET_ICONS.lobstr },
    { id: "hana", name: "Hana", icon: WALLET_ICONS.hana },
  ];

  const connectCustom = async (moduleId: string) => {
    if (process.env.NEXT_PUBLIC_E2E_MOCK_WALLET === "true") {
      setIsConnecting(true);
      try {
        setAddress(E2E_MOCK_WALLET_ADDRESS);
        setSelectedWalletId(moduleId);
        localStorage.setItem("inheritx_wallet_address", E2E_MOCK_WALLET_ADDRESS);
        localStorage.setItem("inheritx_wallet_id", moduleId);
        setIsModalOpen(false);
        router.push("/asset-owner");
      } finally {
        setIsConnecting(false);
      }
      return;
    }

    if (!kit) return;
    setIsConnecting(true);
    try {
      kit.setWallet(moduleId);
      const { address } = await kit.getAddress();

      setAddress(address);
      setSelectedWalletId(moduleId);
      localStorage.setItem("inheritx_wallet_address", address);
      localStorage.setItem("inheritx_wallet_id", moduleId);
      setIsModalOpen(false);
      router.push("/asset-owner");
    } catch (error) {
      console.error("Connection failed:", error);
      throw error;
    } finally {
      setIsConnecting(false);
    }
  };

  const disconnect = useCallback(async () => {
    setAddress(null);
    setSelectedWalletId(null);
    localStorage.removeItem("inheritx_wallet_address");
    localStorage.removeItem("inheritx_wallet_id");
    if (kit) {
      try {
        await kit.disconnect();
      } catch {
        // kit.disconnect() may not be supported by all wallets
      }
    }
  }, [kit]);

  const signTransaction = useCallback(
    async (xdr: string): Promise<{ signedTxXdr: string }> => {
      if (!kit) {
        throw new Error("Wallet not connected. Please connect your wallet first.");
      }
      return await kit.signTransaction(xdr);
    },
    [kit]
  );

  const openModal = () => setIsModalOpen(true);
  const closeModal = () => setIsModalOpen(false);

  return (
    <WalletContext.Provider
      value={{
        connect: connectCustom,
        disconnect,
        signTransaction,
        address,
        isConnected: !!address,
        isConnecting,
        selectedWalletId,
        kit,
        openModal,
        closeModal,
        isModalOpen,
        supportedWallets,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
};
