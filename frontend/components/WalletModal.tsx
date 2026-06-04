"use client";

import { useWallet } from "../context/WalletContext";
import { motion, AnimatePresence } from "framer-motion";
import { Loader2, Wallet, Check } from "lucide-react";
import React from "react";
import UserIcon from "./userIcon";

export function WalletModal() {
  const { isModalOpen, closeModal, supportedWallets, connect, isConnecting } =
    useWallet();

  // If we wanted to "select" first before connecting, we'd need local state.
  // But standard flow is click -> connect.
  // However, the screenshot shows "Connect Wallet" button at the bottom.
  // This implies: Select a wallet (radio) -> Click specific Connect button.
  // I will implement that flow.

  const [activeSelection, setActiveSelection] = React.useState<string | null>(
    null,
  );

  // Reset selection when modal opens
  React.useEffect(() => {
    if (isModalOpen) setActiveSelection(null);
  }, [isModalOpen]);

  const handleConnectClick = () => {
    if (activeSelection) {
      connect(activeSelection);
    }
  };

  return (
    <AnimatePresence>
      {isModalOpen && (
        <>
          {/* Backdrop - Transparent or very subtle to handle click-outside */}
          <div
            className="fixed inset-0 z-40 bg-transparent"
            onClick={closeModal}
          />
          <div className=" bg-[#161E22CC]">
            <motion.div
              initial={{ opacity: 0, y: -10, x: 20 }}
              animate={{ opacity: 1, y: 0, x: 0 }}
              exit={{ opacity: 0, y: -10, x: 20 }}
              transition={{ duration: 0.2, ease: "easeOut" }}
              className="fixed top-24 right-6 z-50 w-120  rounded-4xl bg-[#161E22CC] p-8 shadow-2xl"
            >
              <div className="border border-[#2A3338] bg-[#161E22] rounded-4xl p-[32px] flex flex-col items-center">
                <div className="text-center mb-2">
                  <h2 className="text-2xl font-medium text-white">
                    Connect Wallet
                  </h2>
                  <p className="mt-2 text-[#92A5A8] text-sm">
                    Connect your wallet to get started with InheritX
                  </p>
                </div>

                <div className="flex flex-col gap-3 mt-8 mb-8">
                  {supportedWallets.map((wallet) => {
                    const isSelected = activeSelection === wallet.id;
                    return (
                      <div key={wallet.id} className="group flex items-center gap-4">
                        <div className="w-1.5 h-8 group-hover:bg-[#1C252A] flex items-center justify-center transition-colors" />
                        <button
                          onClick={() => setActiveSelection(wallet.id)}
                          className={`flex group-hover:bg-[#1C252A] items-center gap-4 w-full p-4 rounded-e-2xl transition-all border ${isSelected ? "bg-[#1a2333] border-[#33C5E0]/30" : "bg-transparent border-transparent hover:bg-[#1a2333]"}`}
                        >
                          {/* Radio Circle */}
                          <div
                            className={`w-6 h-6 rounded-full border flex items-center justify-center transition-colors ${isSelected ? "border-[#33C5E0] bg-[#33C5E0]" : "border-[#2d3b4f] bg-white"}`}
                          >
                            {isSelected && (
                              <Check
                                className="w-4 h-4 text-black"
                                strokeWidth={3}
                              />
                            )}
                          </div>

                          {/* Icon Placeholder */}
                          <div className="text-[#92A5A8]">
                            {/* We can use the icon from context or generic */}
                            {wallet.id.includes("freighter") ? (
                              <Wallet className="w-5 h-5" />
                            ) : (
                              <Wallet className="w-5 h-5" />
                            )}
                          </div>

                          <span className="font-semibold text-sm tracking-wider uppercase text-[#92A5A8]">
                            {wallet.name}
                          </span>
                        </button>
                      </div>
                    );
                  })}
                </div>

                <button
                  onClick={handleConnectClick}
                  disabled={!activeSelection || isConnecting}
                  className={`w-full py-4 rounded-full font-medium text-white transition-all flex items-center justify-center gap-2 ${activeSelection ? "bg-[#1C252A] hover:bg-[#1C252A]" : "bg-[#1C252A] cursor-not-allowed text-gray-500"}`}
                >
                 <UserIcon />
                  <span>Connect Wallet</span>
                </button>
              </div>
            </motion.div>
          </div>
        </>
      )}
    </AnimatePresence>
  );
}
