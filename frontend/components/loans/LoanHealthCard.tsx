"use client";

import { useState } from "react";
import { AlertTriangle, CalendarClock, Landmark, ShieldAlert } from "lucide-react";
import type { LoanPosition } from "@/app/lib/stellar/lendingClient";
import {
  LIQUIDATION_LTV_BPS,
  bufferToLiquidationBps,
  calcHealthFactorBps,
  calcLtvBps,
  formatBps,
  formatTokenAmount,
  getRiskLevel,
} from "@/app/lib/loans/ltv";
import { TOKEN_DECIMALS } from "@/app/lib/stellar/network";
import { formatAddress } from "@/util/address";
import { LtvGauge } from "./LtvGauge";

function displayAsset(asset: string): string {
  return asset.length > 12 ? formatAddress(asset) : asset;
}

function formatDate(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export interface LoanHealthCardProps {
  loan: LoanPosition | null;
  loading?: boolean;
  error?: string | null;
  /** Set right after a confirmed repayment so the card explains the empty gauge. */
  justRepaid?: boolean;
  onRepay?: () => void;
  /** Injectable clock for tests (unix seconds). */
  now?: number;
}

export function LoanHealthCard({ loan, loading, error, justRepaid, onRepay, now }: LoanHealthCardProps) {
  const [mountedAt] = useState(() => Math.floor(Date.now() / 1000));
  const nowSeconds = now ?? mountedAt;

  if (loading) {
    return (
      <section className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5" aria-busy="true">
        <div className="h-5 w-32 rounded bg-white/5 animate-pulse" />
        <div className="mt-4 h-32 rounded-xl bg-white/5 animate-pulse" />
      </section>
    );
  }

  const ltvBps = loan ? calcLtvBps(loan.principal, loan.collateralAmount) : 0;
  const risk = getRiskLevel(ltvBps, !!loan);
  const isOverdue = !!loan && nowSeconds > loan.dueDate;
  const fmt = (value: bigint) => formatTokenAmount(value, TOKEN_DECIMALS);

  return (
    <section
      className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5 space-y-4"
      aria-labelledby="loan-health-heading"
    >
      <div className="flex items-end justify-between gap-3">
        <div>
          <h2 id="loan-health-heading" className="text-sm font-semibold text-white flex items-center gap-2">
            <Landmark size={14} className="text-[#33C5E0]" /> Loan Health
          </h2>
          <p className="text-xs text-slate-500 mt-1">
            Collateralised borrowing against your assets on Soroban.
          </p>
        </div>
        {loan && onRepay && (
          <button
            type="button"
            onClick={onRepay}
            className="rounded-full bg-[#33C5E0] px-4 py-2 text-xs font-medium text-[#0B1114] hover:bg-[#33C5E0]/90"
          >
            Repay loan
          </button>
        )}
      </div>

      {error && (
        <div role="alert" className="rounded-xl border border-red-500/20 bg-red-500/5 p-3 text-xs text-red-300">
          {error}
        </div>
      )}

      {loan && (risk === "liquidatable" || risk === "danger") && (
        <div
          role="alert"
          data-testid="liquidation-warning"
          className={`rounded-xl border p-3 flex gap-3 ${
            risk === "liquidatable"
              ? "border-red-500/30 bg-red-500/10 text-red-200"
              : "border-orange-500/30 bg-orange-500/10 text-orange-200"
          }`}
        >
          <ShieldAlert size={16} className="flex-shrink-0 mt-0.5" />
          <div className="text-xs space-y-1">
            <p className="font-semibold text-sm">
              {risk === "liquidatable" ? "Liquidation risk" : "Approaching liquidation"}
            </p>
            <p>
              {risk === "liquidatable"
                ? `Your LTV of ${formatBps(ltvBps)} is above the ${formatBps(LIQUIDATION_LTV_BPS)} threshold. Once the loan passes its due date and grace period, liquidators can repay your debt and seize collateral at 150% of the amount they cover.`
                : `Your LTV of ${formatBps(ltvBps)} is within ${formatBps(LIQUIDATION_LTV_BPS - ltvBps)} of the ${formatBps(LIQUIDATION_LTV_BPS)} liquidation threshold.`}
            </p>
            {onRepay && (
              <button type="button" onClick={onRepay} className="font-medium underline underline-offset-2">
                Repay now to protect your collateral
              </button>
            )}
          </div>
        </div>
      )}

      {loan && isOverdue && (
        <div role="status" className="rounded-xl border border-yellow-500/20 bg-yellow-500/5 p-3 flex gap-2 text-xs text-yellow-300">
          <AlertTriangle size={14} className="flex-shrink-0 mt-0.5" />
          This loan was due on {formatDate(loan.dueDate)}. Late fees accrue daily once the grace period ends.
        </div>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-[auto_1fr] gap-5 items-center">
        <div className="flex justify-center">
          <LtvGauge ltvBps={ltvBps} hasDebt={!!loan} />
        </div>

        {loan ? (
          <dl className="grid grid-cols-2 gap-3 text-xs">
            <div>
              <dt className="text-slate-500">Borrowed</dt>
              <dd className="text-white font-medium mt-0.5">
                {fmt(loan.principal)} {displayAsset(loan.asset)}
              </dd>
            </div>
            <div>
              <dt className="text-slate-500">Collateral</dt>
              <dd className="text-white font-medium mt-0.5">
                {fmt(loan.collateralAmount)} {displayAsset(loan.collateralToken)}
              </dd>
            </div>
            <div>
              <dt className="text-slate-500">Health factor</dt>
              <dd className="text-white font-medium mt-0.5">
                {formatBps(calcHealthFactorBps(loan.principal, loan.collateralAmount), 0)}
                <span className="text-slate-500"> (min 150%)</span>
              </dd>
            </div>
            <div>
              <dt className="text-slate-500">Buffer to liquidation</dt>
              <dd className="text-white font-medium mt-0.5">{formatBps(bufferToLiquidationBps(ltvBps), 1)}</dd>
            </div>
            <div>
              <dt className="text-slate-500">Interest rate</dt>
              <dd className="text-white font-medium mt-0.5">{formatBps(loan.interestRateBps)} APR</dd>
            </div>
            <div>
              <dt className="text-slate-500 flex items-center gap-1">
                <CalendarClock size={11} /> Due
              </dt>
              <dd className={`font-medium mt-0.5 ${isOverdue ? "text-yellow-300" : "text-white"}`}>
                {formatDate(loan.dueDate)}
              </dd>
            </div>
          </dl>
        ) : (
          <p className="text-sm text-slate-400" role="status">
            {justRepaid
              ? "Loan fully repaid — your collateral has been released and LTV is back to 0%."
              : "No open loan on this wallet."}
          </p>
        )}
      </div>
    </section>
  );
}

export default LoanHealthCard;
