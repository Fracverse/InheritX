"use client";

import { useEffect, useState } from "react";
import Image from "next/image";
import QRCode from "qrcode";
import { AlertCircle, Check, Copy, Loader2, Wallet } from "lucide-react";
import { plansAPI, type PlanDepositAddress } from "@/app/lib/api/plans";
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
  planId?: string | null;
  amount: string;
  onAmountChange: (amount: string) => void;
  onBridgeComplete?: (transferId: string) => void;
}

type DepositAddressState =
  | {
      planId: string;
      status: "loaded";
      details: PlanDepositAddress;
      qrCode: string;
    }
  | { planId: string; status: "error"; error: string };

export function CrossChainDepositSection({
  planId,
  amount,
  onAmountChange,
  onBridgeComplete,
}: CrossChainDepositSectionProps) {
  const { address: stellarAddress, isConnected, openModal } = useWallet();
  const [depositState, setDepositState] = useState<DepositAddressState | null>(null);
  const [copyError, setCopyError] = useState("");
  const [copiedField, setCopiedField] = useState<"address" | "memo" | null>(null);
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

  useEffect(() => {
    let cancelled = false;

    if (!planId) return;

    plansAPI
      .getDepositAddress(planId)
      .then(async (details) => {
        const paymentUri = `web+stellar:pay?${new URLSearchParams({
          destination: details.address,
          memo: details.memo,
          memo_type: "text",
        }).toString()}`;
        const qrCode = await QRCode.toDataURL(paymentUri, {
          width: 192,
          margin: 2,
        });
        if (!cancelled) {
          setDepositState({ planId, status: "loaded", details, qrCode });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setDepositState({
            planId,
            status: "error",
            error:
              error instanceof Error
                ? error.message
                : "Could not load the Stellar deposit details.",
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [planId]);

  const copyDepositField = async (field: "address" | "memo", value: string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopyError("");
      setCopiedField(field);
      window.setTimeout(() => setCopiedField(null), 2000);
    } catch {
      setCopyError("Clipboard access is unavailable in this browser.");
    }
  };

  const currentDepositState =
    depositState?.planId === planId ? depositState : null;

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

      <div className="rounded-lg border border-[#2A3338] bg-[#0A0F11] p-4">
        <h4 className="text-xs font-semibold uppercase tracking-wider text-[#92A5A8]">
          Stellar plan deposit
        </h4>
        {planId && !currentDepositState ? (
          <p className="mt-3 flex items-center gap-2 text-sm text-[#92A5A8]">
            <Loader2 size={14} className="animate-spin" />
            Loading deposit address…
          </p>
        ) : currentDepositState?.status === "error" ? (
          <p role="alert" className="mt-3 flex items-center gap-2 text-sm text-[#F56565]">
            <AlertCircle size={15} />
            {currentDepositState.error}
          </p>
        ) : currentDepositState?.status === "loaded" ? (
          <div className="mt-3 flex flex-col gap-4 sm:flex-row sm:items-start">
            <Image
              src={currentDepositState.qrCode}
              alt="QR code for the Stellar plan deposit address and memo"
              width={192}
              height={192}
              unoptimized
              className="rounded bg-white p-2"
            />
            <div className="min-w-0 space-y-3 text-sm">
              <div>
                <p className="text-xs text-[#92A5A8]">Deposit address</p>
                <div className="mt-1 flex items-center gap-2">
                  <p className="min-w-0 break-all font-mono text-slate-200">
                    {currentDepositState.details.address}
                  </p>
                  <button
                    type="button"
                    onClick={() => copyDepositField("address", currentDepositState.details.address)}
                    aria-label="Copy deposit address"
                    title="Copy deposit address"
                    className="shrink-0 text-[#33C5E0] hover:text-cyan-300"
                  >
                    {copiedField === "address" ? <Check size={16} /> : <Copy size={16} />}
                  </button>
                </div>
              </div>
              <div role="alert" className="rounded border border-amber-500/40 bg-amber-500/10 p-3 text-amber-200">
                <p className="text-xs font-semibold uppercase tracking-wide">Memo required</p>
                <p className="mt-1 break-all font-mono">{currentDepositState.details.memo}</p>
                <button
                  type="button"
                  onClick={() => copyDepositField("memo", currentDepositState.details.memo)}
                  className="mt-2 inline-flex items-center gap-1.5 text-xs font-medium text-amber-100 hover:text-white"
                >
                  {copiedField === "memo" ? <Check size={14} /> : <Copy size={14} />}
                  {copiedField === "memo" ? "Copied memo" : "Copy memo"}
                </button>
                <p className="mt-2 text-xs">Include this memo with every deposit or funds may not be credited to this plan.</p>
              </div>
              {copyError && <p role="alert" className="text-xs text-[#F56565]">{copyError}</p>}
            </div>
          </div>
        ) : (
          <p className="mt-2 text-xs text-[#92A5A8]">
            Create the plan to generate its unique Stellar deposit address and memo.
          </p>
        )}
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
