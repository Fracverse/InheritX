"use client";

import { motion } from "framer-motion";
import { ArrowLeftRight, Trash2 } from "lucide-react";
import { isValidStellarAccount } from "@/app/lib/validation/inheritancePlan";
import type { PlanBeneficiaryRequest } from "@/app/lib/api/inheritance";
import type { ContractBeneficiaryInput } from "@/app/lib/stellar/contractParams";

export interface BeneficiaryDraft {
  address: string;
  name: string;
  /** Allocation in basis points. 10000 bps = 100%. */
  allocationBps: number;
  isFiat: boolean;
  fiatBank: string;
  fiatAccount: string;
  fiatCurrency: string;
  /** Optional daily fiat payout limit, entered as a plain decimal string. */
  fiatDailyLimit: string;
  /** Beneficiary email — required on-chain (must be unique per plan). */
  email: string;
  /** 6-digit numeric claim code (0-999999) the beneficiary will later use to claim. */
  claimCode: string;
}

/** Generates a random 6-digit claim code, zero-padded, matching the contract's 0-999999 range. */
export function generateClaimCode(): string {
  const bytes = new Uint32Array(1);
  if (typeof crypto !== "undefined" && crypto.getRandomValues) {
    crypto.getRandomValues(bytes);
  } else {
    bytes[0] = Math.floor(Math.random() * 1_000_000);
  }
  return String(bytes[0] % 1_000_000).padStart(6, "0");
}

export const DEFAULT_BENEFICIARY_DRAFT: BeneficiaryDraft = {
  address: "",
  name: "",
  allocationBps: 0,
  isFiat: false,
  fiatBank: "",
  fiatAccount: "",
  fiatCurrency: "USD",
  fiatDailyLimit: "",
  email: "",
  claimCode: "",
};

export function totalAllocationBps(beneficiaries: BeneficiaryDraft[]): number {
  return beneficiaries.reduce((sum, b) => sum + (b.allocationBps || 0), 0);
}

/** Formats basis points as a percentage string, e.g. 3333 -> "33.33". */
export function bpsToPercentageLabel(bps: number): string {
  return (bps / 100).toFixed(2).replace(/\.00$/, "");
}

/** Converts a user-entered percentage (up to 2 decimals) into basis points. */
export function percentageToBps(percentage: number): number {
  return Math.round(percentage * 100);
}

interface BeneficiaryValidation {
  /** Per-row error message, keyed by beneficiary index. */
  rowErrors: Record<number, string>;
  /** Set when the total allocation doesn't equal exactly 10,000 bps. */
  totalError?: string;
}

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/**
 * Base validation shared by the create and edit panels: wallet address,
 * name and allocation only. This intentionally does NOT require email/claim
 * code/settlement reference — those are only meaningful for the on-chain
 * `create_inheritance_plan` call (see `validateContractBeneficiaryDrafts`),
 * not for editing an existing off-chain plan record.
 */
export function validateBeneficiaryDrafts(
  beneficiaries: BeneficiaryDraft[]
): BeneficiaryValidation {
  const rowErrors: Record<number, string> = {};
  const seenAddresses = new Set<string>();

  beneficiaries.forEach((b, index) => {
    const address = b.address.trim();

    if (!b.name.trim()) {
      rowErrors[index] = "Name is required.";
      return;
    }
    if (!isValidStellarAccount(address)) {
      rowErrors[index] = "Enter a valid Stellar wallet address (starts with G).";
      return;
    }
    if (seenAddresses.has(address)) {
      rowErrors[index] = "This wallet address is already used by another beneficiary.";
      return;
    }
    seenAddresses.add(address);

    if (!b.allocationBps || b.allocationBps <= 0) {
      rowErrors[index] = "Allocation must be greater than 0%.";
      return;
    }
    if (b.isFiat && !b.fiatBank.trim()) {
      rowErrors[index] = "Bank name is required for fiat off-ramp payouts.";
      return;
    }
    if (b.isFiat && !b.fiatAccount.trim()) {
      rowErrors[index] = "Account number is required for fiat off-ramp payouts.";
      return;
    }
  });

  const total = totalAllocationBps(beneficiaries);
  const totalError =
    total !== 10000
      ? `Allocations must total exactly 100% (10,000 bps) — currently ${bpsToPercentageLabel(total)}%.`
      : undefined;

  return { rowErrors, totalError };
}

/**
 * Additional validation required before beneficiaries can be submitted to
 * `create_inheritance_plan` on-chain: a unique email per beneficiary, a
 * 6-digit claim code, and a non-empty settlement reference (the contract
 * rejects an empty `bank_account`). Layered on top of, not instead of,
 * `validateBeneficiaryDrafts`.
 */
export function validateContractBeneficiaryDrafts(
  beneficiaries: BeneficiaryDraft[]
): BeneficiaryValidation {
  const rowErrors: Record<number, string> = {};
  const seenEmails = new Set<string>();

  beneficiaries.forEach((b, index) => {
    const email = b.email.trim().toLowerCase();

    if (!EMAIL_PATTERN.test(email)) {
      rowErrors[index] = "Enter a valid email address.";
      return;
    }
    if (seenEmails.has(email)) {
      rowErrors[index] = "This email is already used by another beneficiary.";
      return;
    }
    seenEmails.add(email);

    if (!/^\d{1,6}$/.test(b.claimCode.trim())) {
      rowErrors[index] = "Claim code must be a 6-digit number.";
      return;
    }

    if (!b.fiatAccount.trim()) {
      rowErrors[index] = "Account / settlement reference is required.";
      return;
    }
  });

  return { rowErrors };
}

/** Builds the fiat_anchor_info payload the backend parses on payout. Empty string means crypto payout. */
export function buildFiatAnchorInfo(b: BeneficiaryDraft): string {
  if (!b.isFiat) return "";
  return JSON.stringify({
    name: b.name.trim(),
    currency: b.fiatCurrency.trim() || "USD",
    bank: b.fiatBank.trim(),
    account: b.fiatAccount.trim(),
    ...(b.fiatDailyLimit.trim() ? { daily_limit: b.fiatDailyLimit.trim() } : {}),
  });
}

/**
 * Converts a draft into the on-chain beneficiary tuple input. `priority` is
 * assigned by the caller (1-indexed row order) since the contract requires
 * unique, non-zero priorities and the UI doesn't expose manual reordering.
 */
export function beneficiaryDraftToContractInput(
  b: BeneficiaryDraft,
  priority: number
): ContractBeneficiaryInput {
  return {
    fullName: b.name.trim(),
    email: b.email.trim().toLowerCase(),
    claimCode: Number.parseInt(b.claimCode.trim(), 10),
    bankAccount: b.fiatAccount.trim(),
    allocationBp: b.allocationBps,
    priority,
  };
}

export function beneficiaryDraftToRequest(b: BeneficiaryDraft): PlanBeneficiaryRequest {
  return {
    address: b.address.trim(),
    name: b.name.trim(),
    allocation_bps: b.allocationBps,
    fiat_anchor_info: buildFiatAnchorInfo(b),
  };
}

interface BeneficiaryAllocationRowProps {
  beneficiary: BeneficiaryDraft;
  index: number;
  error?: string;
  onChange: (
    index: number,
    field: keyof BeneficiaryDraft,
    value: string | number | boolean
  ) => void;
  onRemove: (index: number) => void;
  canRemove: boolean;
}

export function BeneficiaryAllocationRow({
  beneficiary,
  index,
  error,
  onChange,
  onRemove,
  canRemove,
}: BeneficiaryAllocationRowProps) {
  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: -8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.18 }}
      className="space-y-2 rounded-lg border border-[#2A3338] p-3"
    >
      <div className="grid grid-cols-1 sm:grid-cols-[1fr_1fr_90px_90px_36px] gap-3 items-start">
        <div className="flex flex-col gap-1">
          <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
            Name
          </label>
          <input
            type="text"
            value={beneficiary.name}
            onChange={(e) => onChange(index, "name", e.target.value)}
            placeholder="Alice Smith"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
            Wallet Address
          </label>
          <input
            type="text"
            value={beneficiary.address}
            onChange={(e) => onChange(index, "address", e.target.value)}
            placeholder="G..."
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors font-mono"
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
            Share (%)
          </label>
          <input
            type="number"
            min={0}
            max={100}
            step={0.01}
            value={beneficiary.allocationBps ? beneficiary.allocationBps / 100 : ""}
            onChange={(e) =>
              onChange(index, "allocationBps", percentageToBps(Number(e.target.value) || 0))
            }
            placeholder="0"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
            Payout
          </label>
          <button
            type="button"
            onClick={() => onChange(index, "isFiat", !beneficiary.isFiat)}
            aria-label={`Toggle fiat off-ramp for ${beneficiary.name || index + 1}`}
            className={`flex items-center justify-center gap-1.5 px-2 py-2 rounded-lg text-xs font-medium border transition-colors ${
              beneficiary.isFiat
                ? "bg-[#F59E0B14] border-[#F59E0B40] text-[#F59E0B]"
                : "bg-[#48BB7814] border-[#48BB7840] text-[#48BB78]"
            }`}
          >
            <ArrowLeftRight size={12} />
            {beneficiary.isFiat ? "Fiat" : "Token"}
          </button>
        </div>

        <button
          type="button"
          onClick={() => onRemove(index)}
          disabled={!canRemove}
          aria-label={`Remove beneficiary ${beneficiary.name || index + 1}`}
          className="mt-6 p-2 rounded-lg text-[#F56565] hover:bg-[#F5656514] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          <Trash2 size={16} />
        </button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-1 border-t border-[#2A3338]/60">
        <div className="flex flex-col gap-1 pt-2">
          <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
            Email
          </label>
          <input
            type="email"
            value={beneficiary.email}
            onChange={(e) => onChange(index, "email", e.target.value)}
            placeholder="alice@example.com"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>
        <div className="flex flex-col gap-1 pt-2">
          <div className="flex items-center justify-between">
            <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
              Claim Code
            </label>
            <button
              type="button"
              onClick={() => onChange(index, "claimCode", generateClaimCode())}
              className="text-[10px] text-[#33C5E0] hover:text-cyan-300 transition-colors"
            >
              Generate
            </button>
          </div>
          <input
            type="text"
            inputMode="numeric"
            maxLength={6}
            value={beneficiary.claimCode}
            onChange={(e) => onChange(index, "claimCode", e.target.value.replace(/\D/g, ""))}
            placeholder="000000"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors font-mono"
          />
        </div>
        <div className="flex flex-col gap-1 pt-2">
          <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
            Account / Settlement Reference
          </label>
          <input
            type="text"
            value={beneficiary.fiatAccount}
            onChange={(e) => onChange(index, "fiatAccount", e.target.value)}
            placeholder="0123456789"
            className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
          />
        </div>
      </div>

      {beneficiary.isFiat && (
        <div className="grid grid-cols-2 sm:grid-cols-2 gap-3 pt-1 border-t border-[#2A3338]/60">
          <div className="flex flex-col gap-1 pt-2">
            <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
              Bank Name
            </label>
            <input
              type="text"
              value={beneficiary.fiatBank}
              onChange={(e) => onChange(index, "fiatBank", e.target.value)}
              placeholder="First Bank"
              className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
            />
          </div>
          <div className="flex flex-col gap-1 pt-2">
            <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
              Currency
            </label>
            <input
              type="text"
              value={beneficiary.fiatCurrency}
              onChange={(e) => onChange(index, "fiatCurrency", e.target.value.toUpperCase())}
              placeholder="NGN"
              maxLength={3}
              className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors uppercase"
            />
          </div>
          <div className="flex flex-col gap-1 pt-2">
            <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
              Daily Limit (optional)
            </label>
            <input
              type="number"
              min={0}
              value={beneficiary.fiatDailyLimit}
              onChange={(e) => onChange(index, "fiatDailyLimit", e.target.value)}
              placeholder="Unlimited"
              className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
            />
          </div>
        </div>
      )}

      {error && <p className="text-xs text-[#F56565]">{error}</p>}
    </motion.div>
  );
}
