"use client";

import React, { createContext, useContext, useEffect, useState } from "react";
import {
  StellarWalletsKit,
  WalletNetwork,
  allowAllModules,
} from "@creit.tech/stellar-wallets-kit";
import { useRouter } from "next/navigation";

interface WalletContextType {
  connect: (moduleId: string) => Promise<void>;
  disconnect: () => Promise<void>;
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

  // Initialize kit on mount
  useEffect(() => {
    const walletKit = new StellarWalletsKit({
      network: WalletNetwork.TESTNET,
      selectedWalletId: "freighter",
      modules: allowAllModules(),
    });
    setKit(walletKit);

    // Check for persisted session
    const checkSession = async () => {
      // Basic persistence check - in a real app might verify token/session
      // For now, we rely on the kit's internal state if it has one, or we can check simple localStorage
      // But purely client-side:
      const savedAddress = localStorage.getItem("inheritx_wallet_address");
      const savedWalletId = localStorage.getItem("inheritx_wallet_id");

      if (savedAddress && savedWalletId) {
        setAddress(savedAddress);
        setSelectedWalletId(savedWalletId);
        // We technically aren't "connected" in the kit sense until we call something,
        // but for UI purposes we show the address.
        // A robust implementation would verify connection here.
      }
    };

    checkSession();
  }, []);

  const supportedWallets = [
    { id: "freighter", name: "Freighter", icon: "/icons/freighter.png" },
    { id: "albedo", name: "Albedo", icon: "/icons/albedo.png" },
    { id: "xbull", name: "xBull", icon: "/icons/xbull.png" },
    { id: "rabet", name: "Rabet", icon: "/icons/rabet.png" },
    { id: "lobstr", name: "Lobstr", icon: "/icons/rabet.png" },
  ];

  const connectCustom = async (moduleId: string) => {
    if (!kit) return;
    setIsConnecting(true);
    try {
      // Set the wallet module
      kit.setWallet(moduleId);

      // Request address (triggers popup)
      const { address } = await kit.getAddress();

      setAddress(address);
      setSelectedWalletId(moduleId);
      localStorage.setItem("inheritx_wallet_address", address);
      localStorage.setItem("inheritx_wallet_id", moduleId);
      setIsModalOpen(false);
      router.push("/asset-owner");
    } catch (error) {
      console.error("Connection failed:", error);
      // Handle specific errors (user rejected, extension not found)
      throw error;
    } finally {
      setIsConnecting(false);
    }
  };

  const disconnect = async () => {
    setAddress(null);
    setSelectedWalletId(null);
    localStorage.removeItem("inheritx_wallet_address");
    localStorage.removeItem("inheritx_wallet_id");
    // kit.disconnect() if available
  };

  const openModal = () => setIsModalOpen(true);
  const closeModal = () => setIsModalOpen(false);

  return (
    <WalletContext.Provider
      value={{
        connect: connectCustom,
        disconnect,
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
