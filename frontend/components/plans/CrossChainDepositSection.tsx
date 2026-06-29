"use client";

import { useEffect, useState } from "react";
import { Loader2, Wallet } from "lucide-react";
import {
  getOriginChain,
  getTokensForChain,
  ORIGIN_CHAINS,
} from "@/lib/bridge/chains";
import type { OriginChainId, SourceToken } from "@/lib/bridge/types";
import { useBridgeDeposit } from "@/hooks/useBridgeDeposit";
import { useWallet } from "@/context/WalletContext";
import { formatAddress } from "@/util/address";
import { BridgeFeeBreakdown } from "./BridgeFeeBreakdown";
import { BridgeProgressStepper } from "./BridgeProgressStepper";

interface CrossChainDepositSectionProps {
  amount: string;
  onAmountChange: (amount: string) => void;
  onBridgeComplete?: (transferId: string) => void;
}

export function CrossChainDepositSection({
  amount,
  onAmountChange,
  onBridgeComplete,
}: CrossChainDepositSectionProps) {
  const { address: stellarAddress, isConnected, openModal } = useWallet();
  const [sourceChain, setSourceChain] = useState<OriginChainId>("ETH");
  const [sourceToken, setSourceToken] = useState<SourceToken>(
    getTokensForChain("ETH")[0]
  );

  const {
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
    isSourceWalletConnecting,
    canStartBridge,
    connectSourceWallet,
    startBridge,
  } = useBridgeDeposit({ sourceChain, sourceToken, amount });

  useEffect(() => {
    if (transferId) {
      onBridgeComplete?.(transferId);
    }
  }, [transferId, onBridgeComplete]);

  const handleChainChange = (chainId: OriginChainId) => {
    setSourceChain(chainId);
    const tokens = getTokensForChain(chainId);
    setSourceToken(tokens[0]);
  };

  const chain = getOriginChain(sourceChain);
  const sourceWalletLabel =
    walletType === "solana" ? "Phantom" : "MetaMask";

  return (
    <section className="space-y-5">
      <div>
        <h3 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider mb-1">
          Cross-Chain Deposit
        </h3>
        <p className="text-xs text-[#92A5A8]">
          Bridge assets from EVM or Solana into your Stellar inheritance plan via
          Allbridge.
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div className="flex flex-col gap-1">
          <label htmlFor="origin-chain" className="text-xs text-[#92A5A8]">
            Origin chain
          </label>
          <select
            id="origin-chain"
            value={sourceChain}
            onChange={(e) => handleChainChange(e.target.value as OriginChainId)}
            disabled={isBridging}
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
          >
            {ORIGIN_CHAINS.map((item) => (
              <option key={item.id} value={item.id}>
                {item.name}
              </option>
            ))}
          </select>
        </div>

        <div className="flex flex-col gap-1">
          <label htmlFor="source-token" className="text-xs text-[#92A5A8]">
            Source token
          </label>
          <select
            id="source-token"
            value={sourceToken.address}
            onChange={(e) => {
              const token = getTokensForChain(sourceChain).find(
                (item) => item.address === e.target.value
              );
              if (token) setSourceToken(token);
            }}
            disabled={isBridging}
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
          >
            {getTokensForChain(sourceChain).map((token) => (
              <option key={token.address} value={token.address}>
                {token.symbol} — {token.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="flex flex-col gap-1">
        <label htmlFor="deposit-amount" className="text-xs text-[#92A5A8]">
          Deposit amount ({sourceToken.symbol})
        </label>
        <input
          id="deposit-amount"
          type="number"
          min={0}
          step="0.01"
          value={amount}
          onChange={(e) => onAmountChange(e.target.value)}
          disabled={isBridging}
          placeholder="100.00"
          className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors max-w-xs"
        />
      </div>

      <BridgeFeeBreakdown quote={quote} isLoading={isQuoteLoading} />

      {quoteError && (
        <p className="text-xs text-[#F56565]">{quoteError}</p>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div className="rounded-xl border border-[#2A3338] bg-[#0A0F11] p-4 space-y-2">
          <p className="text-[10px] uppercase tracking-wider text-[#92A5A8]">
            Destination (Freighter)
          </p>
          {isStellarConnected && stellarAddress ? (
            <p className="text-sm font-mono text-[#33C5E0]">
              {formatAddress(stellarAddress)}
            </p>
          ) : (
            <button
              type="button"
              onClick={openModal}
              className="inline-flex items-center gap-2 text-sm text-[#33C5E0] hover:text-cyan-300"
            >
              <Wallet size={14} />
              Connect Freighter
            </button>
          )}
        </div>

        <div className="rounded-xl border border-[#2A3338] bg-[#0A0F11] p-4 space-y-2">
          <p className="text-[10px] uppercase tracking-wider text-[#92A5A8]">
            Source ({sourceWalletLabel} · {chain?.name})
          </p>
          {isSourceWalletConnected && sourceWalletAddress ? (
            <p className="text-sm font-mono text-slate-200 break-all">
              {walletType === "evm"
                ? formatAddress(sourceWalletAddress)
                : sourceWalletAddress.slice(0, 8) + "…" + sourceWalletAddress.slice(-6)}
            </p>
          ) : (
            <button
              type="button"
              onClick={() => connectSourceWallet()}
              disabled={isSourceWalletConnecting}
              className="inline-flex items-center gap-2 text-sm text-[#33C5E0] hover:text-cyan-300 disabled:opacity-50"
            >
              {isSourceWalletConnecting ? (
                <Loader2 size={14} className="animate-spin" />
              ) : (
                <Wallet size={14} />
              )}
              Connect {sourceWalletLabel}
            </button>
          )}
        </div>
      </div>

      {(isBridging || steps.some((s) => s.status !== "pending")) && (
        <div className="rounded-xl border border-[#2A3338] bg-[#161E22] p-4">
          <h4 className="text-xs font-semibold uppercase tracking-wider text-[#33C5E0] mb-4">
            Bridge Progress
          </h4>
          <BridgeProgressStepper steps={steps} />
        </div>
      )}

      {bridgeError && (
        <p className="text-sm text-[#F56565]">{bridgeError}</p>
      )}

      <button
        type="button"
        onClick={() => startBridge()}
        disabled={!canStartBridge}
        className="inline-flex items-center justify-center gap-2 px-5 py-2.5 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed w-full sm:w-auto"
      >
        {isBridging ? (
          <>
            <Loader2 size={15} className="animate-spin" />
            Bridging in progress…
          </>
        ) : (
          "Start Cross-Chain Deposit"
        )}
      </button>

      {!isConnected && (
        <p className="text-xs text-[#ED8936]">
          Connect Freighter to receive bridged tokens on Stellar before starting
          the deposit.
        </p>
      )}
    </section>
  );
}

export default CrossChainDepositSection;
