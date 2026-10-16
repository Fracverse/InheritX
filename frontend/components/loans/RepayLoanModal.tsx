"use client";

import { useCallback, useEffect, useId, useState } from "react";
import { AlertTriangle, CheckCircle2, ExternalLink, Loader2, RefreshCw, X } from "lucide-react";
import { useWallet } from "@/context/WalletContext";
import {
  getRepaymentQuote,
  invokeRepayLoan,
  type LoanPosition,
  type RepayLoanResult,
  type RepayLoanStage,
  type RepaymentQuote,
} from "@/app/services/loanRepaymentService";
import { calcLtvBps, formatBps, formatTokenAmount, getRiskLevel } from "@/app/lib/loans/ltv";
import { TOKEN_DECIMALS } from "@/app/lib/stellar/network";
import { formatAddress } from "@/util/address";
import { LtvGauge } from "./LtvGauge";

const STAGE_LABELS: Record<RepayLoanStage, string> = {
  building: "Simulating repayment on Soroban…",
  "awaiting-signature": "Approve the token transfer in your wallet…",
  submitting: "Submitting transaction to the network…",
  confirmed: "Repayment confirmed",
};

function displayAsset(asset: string): string {
  return asset.length > 12 ? formatAddress(asset) : asset;
}

export interface RepayLoanModalProps {
  isOpen: boolean;
  loan: LoanPosition;
  onClose: () => void;
  /** Called once the repayment is confirmed, so the dashboard can refresh the LTV gauge. */
  onRepaid?: (result: RepayLoanResult) => void;
}

export function RepayLoanModal(props: RepayLoanModalProps) {
  // Mount the content only while open so every opening starts from a fresh
  // quote and a clean submission state.
  if (!props.isOpen) return null;
  return <RepayLoanModalContent {...props} />;
}

function RepayLoanModalContent({ loan, onClose, onRepaid }: RepayLoanModalProps) {
  const { signTransaction } = useWallet();
  const titleId = useId();
  const [quote, setQuote] = useState<RepaymentQuote | null>(null);
  const [quoteError, setQuoteError] = useState<string | null>(null);
  const [isQuoting, setIsQuoting] = useState(true);
  const [stage, setStage] = useState<RepayLoanStage | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [result, setResult] = useState<RepayLoanResult | null>(null);

  const isSubmitting = stage !== null && stage !== "confirmed" && !submitError;

  const fetchQuote = useCallback(() => {
    let cancelled = false;
    getRepaymentQuote(loan)
      .then((next) => {
        if (!cancelled) { setQuote(next); setQuoteError(null); }
      })
      .catch((error) => {
        if (!cancelled) {
          setQuote(null);
          setQuoteError(error instanceof Error ? error.message : "Couldn't load the repayment amount.");
        }
      })
      .finally(() => { if (!cancelled) setIsQuoting(false); });
    return () => { cancelled = true; };
  }, [loan]);

  useEffect(() => fetchQuote(), [fetchQuote]);

  const retryQuote = () => {
    setIsQuoting(true);
    setQuoteError(null);
    fetchQuote();
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !isSubmitting) onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [isSubmitting, onClose]);

  const currentLtvBps = calcLtvBps(loan.principal, loan.collateralAmount);
  const risk = getRiskLevel(currentLtvBps);
  const asset = displayAsset(loan.asset);
  const collateral = displayAsset(loan.collateralToken);
  const fmt = (value: bigint) => formatTokenAmount(value, TOKEN_DECIMALS);

  const handleRepay = async () => {
    if (!quote) return;
    setSubmitError(null);
    try {
      const repaid = await invokeRepayLoan({ loan, quote, signTransaction, onProgress: setStage });
      setResult(repaid);
      onRepaid?.(repaid);
    } catch (error) {
      setSubmitError(error instanceof Error ? error.message : "Repayment failed.");
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={() => !isSubmitting && onClose()}
        aria-hidden="true"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="relative w-full max-w-lg rounded-3xl border border-white/[0.08] bg-[#161E22] p-6 shadow-2xl"
      >
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 id={titleId} className="text-lg font-semibold text-white">
              {result ? "Loan repaid" : "Repay loan"}
            </h2>
            <p className="text-xs text-slate-500 mt-1">
              Loan #{loan.loanId.toString()} · {asset} borrowed against {collateral}
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            disabled={isSubmitting}
            aria-label="Close"
            className="rounded-lg p-1.5 text-slate-400 hover:bg-white/5 hover:text-white disabled:opacity-40"
          >
            <X size={16} />
          </button>
        </div>

        {result ? (
          <div className="mt-6 space-y-5">
            <div className="flex justify-center">
              <LtvGauge ltvBps={0} hasDebt={false} label="Loan-to-value after repayment" />
            </div>
            <div className="rounded-2xl border border-emerald-500/20 bg-emerald-500/5 p-4 flex gap-3">
              <CheckCircle2 size={18} className="text-emerald-400 flex-shrink-0 mt-0.5" />
              <div className="text-sm text-emerald-200">
                <p>
                  Paid {fmt(result.quote.total)} {asset}. {fmt(loan.collateralAmount)} {collateral} collateral has been
                  released to your wallet.
                </p>
                {result.explorerUrl ? (
                  <a
                    href={result.explorerUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="mt-2 inline-flex items-center gap-1 text-xs text-[#33C5E0] hover:underline"
                  >
                    View on Stellar Expert <ExternalLink size={11} />
                  </a>
                ) : (
                  <p className="mt-1 text-xs text-emerald-300/70">Demo mode — no transaction was submitted.</p>
                )}
              </div>
            </div>
            <button
              type="button"
              onClick={onClose}
              className="w-full rounded-full bg-[#33C5E0] py-3 text-sm font-medium text-[#0B1114] hover:bg-[#33C5E0]/90"
            >
              Done
            </button>
          </div>
        ) : (
          <div className="mt-6 space-y-5">
            <div className="flex items-center justify-around gap-4">
              <LtvGauge ltvBps={currentLtvBps} projectedLtvBps={0} size={180} />
              <div className="text-xs text-slate-400 space-y-1">
                <p>
                  Now: <span className="text-white font-medium">{formatBps(currentLtvBps)}</span>
                </p>
                <p>
                  After repayment: <span className="text-[#33C5E0] font-medium">0.00%</span>
                </p>
                <p>Collateral released: {fmt(loan.collateralAmount)} {collateral}</p>
              </div>
            </div>

            {(risk === "danger" || risk === "liquidatable") && (
              <div role="alert" className="rounded-2xl border border-red-500/20 bg-red-500/5 p-3 flex gap-2 text-xs text-red-300">
                <AlertTriangle size={14} className="flex-shrink-0 mt-0.5" />
                Repaying now closes the position before it can be liquidated.
              </div>
            )}

            <div className="rounded-2xl border border-white/[0.08] bg-white/[0.02] p-4">
              {isQuoting ? (
                <div className="space-y-2" aria-busy="true">
                  {[1, 2, 3].map((i) => (
                    <div key={i} className="h-4 rounded bg-white/5 animate-pulse" />
                  ))}
                </div>
              ) : quoteError ? (
                <div className="flex items-center justify-between gap-3 text-xs text-red-300">
                  <span role="alert">{quoteError}</span>
                  <button
                    type="button"
                    onClick={retryQuote}
                    className="inline-flex items-center gap-1 text-[#33C5E0] hover:underline"
                  >
                    <RefreshCw size={11} /> Retry
                  </button>
                </div>
              ) : quote ? (
                <dl className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <dt className="text-slate-400">Principal</dt>
                    <dd className="text-white">{fmt(quote.principal)} {asset}</dd>
                  </div>
                  <div className="flex justify-between">
                    <dt className="text-slate-400">Accrued interest ({formatBps(loan.interestRateBps)} APR)</dt>
                    <dd className="text-white">{fmt(quote.interest)} {asset}</dd>
                  </div>
                  {quote.lateFee > BigInt(0) && (
                    <div className="flex justify-between">
                      <dt className="text-orange-300">Late fee</dt>
                      <dd className="text-orange-300">{fmt(quote.lateFee)} {asset}</dd>
                    </div>
                  )}
                  <div className="flex justify-between border-t border-white/[0.08] pt-2 font-semibold">
                    <dt className="text-white">Total due</dt>
                    <dd className="text-white" data-testid="repay-total">
                      {fmt(quote.total)} {asset}
                    </dd>
                  </div>
                </dl>
              ) : null}
            </div>

            <p className="text-[11px] text-slate-500">
              Repayment settles the full balance in one Soroban call: the lending contract transfers principal +
              interest from your wallet and returns all collateral. Interest keeps accruing until the transaction
              lands, so keep a small buffer above the total.
            </p>

            {stage && !submitError && (
              <p className="flex items-center gap-2 text-xs text-[#33C5E0]" aria-live="polite">
                <Loader2 size={12} className="animate-spin" /> {STAGE_LABELS[stage]}
              </p>
            )}
            {submitError && (
              <p role="alert" className="rounded-xl border border-red-500/20 bg-red-500/5 p-3 text-xs text-red-300">
                {submitError}
              </p>
            )}

            <div className="flex gap-3">
              <button
                type="button"
                onClick={onClose}
                disabled={isSubmitting}
                className="flex-1 rounded-full border border-white/[0.08] py-3 text-sm text-slate-300 hover:bg-white/5 disabled:opacity-40"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => void handleRepay()}
                disabled={!quote || isQuoting || isSubmitting}
                className="flex-1 rounded-full bg-[#33C5E0] py-3 text-sm font-medium text-[#0B1114] hover:bg-[#33C5E0]/90 disabled:opacity-40 disabled:cursor-not-allowed inline-flex items-center justify-center gap-2"
              >
                {isSubmitting && <Loader2 size={14} className="animate-spin" />}
                {quote ? `Repay ${fmt(quote.total)} ${asset}` : "Repay"}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default RepayLoanModal;
