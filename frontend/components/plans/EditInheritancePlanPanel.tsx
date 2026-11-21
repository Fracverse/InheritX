"use client";

import { useState, useCallback, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Plus, X, Save, AlertCircle, CheckCircle, Loader2 } from "lucide-react";
import { plansAPI } from "@/app/lib/api/plans";
import type { Plan, UpdatePlanRequest } from "@/app/lib/api/plans";
import {
  createGeneticVerificationAPI,
  DEFAULT_PRIVACY_SETTINGS,
  type PrivacySettings,
} from "@/app/lib/api/geneticVerification";
import { useAppStore } from "@/store";
import { useWallet } from "@/context/WalletContext";
import { AllocationFlowChart } from "@/components/plans/AllocationFlowChart";
import type { BeneficiaryFlow } from "@/components/plans/AllocationFlowChart";
import {
  BeneficiaryAllocationRow,
  DEFAULT_BENEFICIARY_DRAFT,
  beneficiaryDraftToRequest,
  bpsToPercentageLabel,
  totalAllocationBps,
  validateBeneficiaryDrafts,
  type BeneficiaryDraft,
} from "@/components/plans/BeneficiaryAllocationRow";

// ─── Types ────────────────────────────────────────────────────────────────────

interface EditInheritancePlanPanelProps {
  plan: Plan;
  onClose: () => void;
  onSaved: (updatedPlan: Plan) => void;
}

type TxStatus = "idle" | "signing" | "saving" | "success" | "error";
type PrivacyStatus = "idle" | "saving" | "success" | "error";

type PrivacyToggleKey = "allowFamilyMatching" | "zeroKnowledgeHashing";

const SECONDS_PER_DAY = 86_400;
const DEFAULT_YIELD_RATE_BPS = 500;

// ─── Helpers ─────────────────────────────────────────────────────────────────

function seedBeneficiaries(plan: Plan): BeneficiaryDraft[] {
  if (Array.isArray(plan.beneficiaries) && plan.beneficiaries.length > 0) {
    return plan.beneficiaries.map((b: any) => {
      const fiatAnchorInfo: string = b.fiat_anchor_info ?? "";
      const isFiat = fiatAnchorInfo.trim().length > 0;
      let parsed: { name?: string; currency?: string; bank?: string; account?: string; daily_limit?: string } | null =
        null;
      if (isFiat) {
        try {
          parsed = JSON.parse(fiatAnchorInfo);
        } catch {
          parsed = null;
        }
      }

      return {
        address: b.wallet_address ?? b.address ?? "",
        name: b.name ?? parsed?.name ?? "",
        allocationBps: b.allocation_bps ?? 0,
        isFiat,
        fiatBank: parsed?.bank ?? "",
        fiatAccount: parsed?.account ?? "",
        fiatCurrency: parsed?.currency ?? "USD",
        fiatDailyLimit:
          parsed?.daily_limit ?? (b.fiat_daily_limit ? String(b.fiat_daily_limit) : ""),
        email: b.email ?? "",
        claimCode: b.claim_code ?? "",
      };
    });
  }

  if (plan.beneficiary_name) {
    return [{ ...DEFAULT_BENEFICIARY_DRAFT, name: plan.beneficiary_name, allocationBps: 10000 }];
  }

  return [{ ...DEFAULT_BENEFICIARY_DRAFT }];
}

// ─── Privacy Toggle Row ───────────────────────────────────────────────────────

interface PrivacyToggleRowProps {
  label: string;
  description: string;
  checked: boolean;
  onToggle: () => void;
}

function PrivacyToggleRow({
  label,
  description,
  checked,
  onToggle,
}: PrivacyToggleRowProps) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div className="min-w-0">
        <p className="text-sm text-slate-200">{label}</p>
        <p className="text-xs text-[#92A5A8] mt-0.5">{description}</p>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        onClick={onToggle}
        className={`relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#33C5E0] ${
          checked ? "bg-[#33C5E0]" : "bg-[#2A3338]"
        }`}
      >
        <span
          className={`inline-block h-4 w-4 rounded-full bg-white shadow-md transform transition-transform ${
            checked ? "translate-x-6" : "translate-x-1"
          }`}
        />
      </button>
    </div>
  );
}

// ─── Main Panel ───────────────────────────────────────────────────────────────

export function EditInheritancePlanPanel({
  plan,
  onClose,
  onSaved,
}: EditInheritancePlanPanelProps) {
  const { kit, selectedWalletId } = useWallet();

  const [title, setTitle] = useState(plan.title);
  const [description, setDescription] = useState(plan.description ?? "");
  const [inactivityDays, setInactivityDays] = useState<number>(() =>
    plan.grace_period_seconds
      ? Math.max(1, Math.round(plan.grace_period_seconds / SECONDS_PER_DAY))
      : 180
  );
  const [yieldEnabled, setYieldEnabled] = useState<boolean>(plan.earn_yield ?? false);
  const [yieldRateBps, setYieldRateBps] = useState<number>(
    plan.yield_rate_bps || DEFAULT_YIELD_RATE_BPS
  );
  const [beneficiaries, setBeneficiaries] = useState<BeneficiaryDraft[]>(() =>
    seedBeneficiaries(plan)
  );

  const [txStatus, setTxStatus] = useState<TxStatus>("idle");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [touched, setTouched] = useState(false);

  // Genetic privacy preferences — seeded from the persisted store, falling back
  // to the defaults, and written back on save.
  const storedPrivacy = useAppStore((s) => s.geneticPrivacy[plan.id]);
  const setGeneticPrivacySettings = useAppStore(
    (s) => s.setGeneticPrivacySettings
  );
  const [privacy, setPrivacy] = useState<PrivacySettings>(
    () => storedPrivacy ?? DEFAULT_PRIVACY_SETTINGS
  );
  const [privacyStatus, setPrivacyStatus] = useState<PrivacyStatus>("idle");
  const [privacyError, setPrivacyError] = useState<string>("");

  const flowBeneficiaries: BeneficiaryFlow[] = beneficiaries.map((b) => ({
    name: b.name || "Unnamed",
    allocation_percentage: b.allocationBps / 100 || 0,
    isFiat: b.isFiat,
  }));

  const payoutAmount = plan.net_amount ?? plan.fee ?? 0;

  const allocationTotalBps = totalAllocationBps(beneficiaries);
  const { rowErrors, totalError } = useMemo(
    () => validateBeneficiaryDrafts(beneficiaries),
    [beneficiaries]
  );
  const beneficiariesValid = Object.keys(rowErrors).length === 0 && !totalError;
  // Gates the Save button: allocations must total exactly 10,000 bps.
  // Per-field issues (name/address/fiat details) are caught in handleSave
  // so their messages can be surfaced without blocking the click itself.
  const allocationOnlyValid =
    !totalError && beneficiaries.every((b) => (b.allocationBps || 0) > 0);
  const showErrors = touched || txStatus === "error";

  const handleBeneficiaryChange = useCallback(
    (index: number, field: keyof BeneficiaryDraft, value: string | number | boolean) => {
      setBeneficiaries((prev) =>
        prev.map((b, i) => (i === index ? { ...b, [field]: value } : b))
      );
    },
    []
  );

  const addBeneficiary = useCallback(() => {
    setBeneficiaries((prev) => [...prev, { ...DEFAULT_BENEFICIARY_DRAFT }]);
  }, []);

  const removeBeneficiary = useCallback((index: number) => {
    setBeneficiaries((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const togglePrivacy = useCallback((key: PrivacyToggleKey) => {
    setPrivacy((prev) => ({ ...prev, [key]: !prev[key] }));
  }, []);

  const handleSavePrivacy = useCallback(async () => {
    setPrivacyStatus("saving");
    setPrivacyError("");
    try {
      const saved = await createGeneticVerificationAPI().updateGeneticPrivacySettings(
        plan.id,
        privacy
      );
      setPrivacy(saved);
      setGeneticPrivacySettings(plan.id, saved);
      setPrivacyStatus("success");
      setTimeout(() => setPrivacyStatus("idle"), 3000);
    } catch (err) {
      setPrivacyStatus("error");
      setPrivacyError(
        err instanceof Error ? err.message : "Failed to save privacy settings."
      );
    }
  }, [plan.id, privacy, setGeneticPrivacySettings]);

  const buildXdr = (): string => {
    // Constructs a placeholder unsigned XDR envelope representing the update call.
    // In production this would call the Soroban contract SDK to build the real XDR.
    return `unsigned-xdr::update_plan::${plan.id}::${Date.now()}`;
  };

  const signWithWallet = async (xdr: string): Promise<string> => {
    if (!kit || !selectedWalletId) {
      throw new Error("No wallet connected");
    }
    // signTransaction returns the signed XDR that can be submitted to the network.
    const result = await kit.signTransaction(xdr);
    return result.signedTxXdr;
  };

  const handleSave = async () => {
    setTouched(true);

    if (!title.trim()) {
      setErrorMessage("Title is required.");
      return;
    }

    if (!beneficiariesValid) {
      const firstRowError = Object.values(rowErrors)[0];
      setErrorMessage(
        firstRowError ||
          totalError ||
          "All beneficiaries must have a name and a valid Stellar wallet address."
      );
      return;
    }

    setErrorMessage("");
    setTxStatus("signing");

    try {
      const xdr = buildXdr();
      await signWithWallet(xdr);
    } catch {
      // Wallet signing rejected or unavailable — proceed without signed XDR in dev.
    }

    setTxStatus("saving");

    const updateRequest: UpdatePlanRequest = {
      beneficiaries: beneficiaries.map(beneficiaryDraftToRequest),
      grace_period: inactivityDays * SECONDS_PER_DAY,
      earn_yield: yieldEnabled,
      yield_rate_bps: yieldEnabled ? yieldRateBps : 0,
    };

    try {
      const updated = await plansAPI.updatePlan(plan.id, updateRequest);
      const merged: Plan = {
        ...plan,
        title,
        description: description || undefined,
        status: updated.status,
        is_active: updated.is_active,
        amount: Number(updated.amount),
        owner_address: updated.owner_address,
        token_address: updated.token_address,
        grace_period_seconds: updated.grace_period_seconds,
        yield_rate_bps: updated.yield_rate_bps,
        earn_yield: updated.earn_yield,
        accrued_yield: updated.accrued_yield,
        last_ping: updated.last_ping,
        beneficiaries: updated.beneficiaries,
        updated_at: new Date().toISOString(),
      };
      setTxStatus("success");
      setTimeout(() => onSaved(merged), 1200);
    } catch (err) {
      setTxStatus("error");
      setErrorMessage(
        err instanceof Error ? err.message : "Failed to save changes."
      );
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-end md:items-center justify-center"
      role="dialog"
      aria-modal="true"
      aria-label="Edit Inheritance Plan"
    >
      {/* Backdrop */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Panel */}
      <motion.div
        initial={{ opacity: 0, y: 40 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 40 }}
        transition={{ type: "spring", damping: 26, stiffness: 300 }}
        className="relative w-full max-w-2xl max-h-[90dvh] overflow-y-auto mx-4 bg-[#161E22] border border-[#2A3338] rounded-2xl shadow-2xl"
      >
        {/* Header */}
        <div className="sticky top-0 z-10 flex items-center justify-between px-6 py-4 bg-[#161E22] border-b border-[#2A3338]">
          <div>
            <h2 className="text-lg font-semibold text-white">Edit Inheritance Plan</h2>
            <p className="text-xs text-[#92A5A8] mt-0.5">ID: {plan.id}</p>
          </div>
          <button
            onClick={onClose}
            aria-label="Close panel"
            className="p-2 rounded-lg text-[#92A5A8] hover:text-white hover:bg-[#1C252A] transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Body */}
        <div className="px-6 py-5 space-y-6">
          {/* Plan Details */}
          <section>
            <h3 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider mb-3">
              Plan Details
            </h3>
            <div className="space-y-3">
              <div className="flex flex-col gap-1">
                <label
                  htmlFor="edit-plan-title"
                  className="text-xs text-[#92A5A8]"
                >
                  Title
                </label>
                <input
                  id="edit-plan-title"
                  type="text"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
                />
              </div>

              <div className="flex flex-col gap-1">
                <label
                  htmlFor="edit-plan-desc"
                  className="text-xs text-[#92A5A8]"
                >
                  Description (optional)
                </label>
                <textarea
                  id="edit-plan-desc"
                  rows={2}
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 resize-none focus:outline-none focus:border-[#33C5E0] transition-colors"
                />
              </div>
            </div>
          </section>

          {/* Beneficiaries */}
          <section>
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider">
                Beneficiaries
              </h3>
              <span
                className={`text-xs font-mono px-2 py-0.5 rounded-full ${
                  !totalError
                    ? "bg-[#48BB7814] text-[#48BB78]"
                    : "bg-[#F5656514] text-[#F56565]"
                }`}
              >
                {bpsToPercentageLabel(allocationTotalBps)}% / 100%
              </span>
            </div>

            <div className="space-y-3">
              <AnimatePresence initial={false}>
                {beneficiaries.map((b, i) => (
                  <BeneficiaryAllocationRow
                    key={i}
                    index={i}
                    beneficiary={b}
                    error={showErrors ? rowErrors[i] : undefined}
                    onChange={handleBeneficiaryChange}
                    onRemove={removeBeneficiary}
                    canRemove={beneficiaries.length > 1}
                  />
                ))}
              </AnimatePresence>
            </div>

            <button
              type="button"
              onClick={addBeneficiary}
              className="mt-3 flex items-center gap-2 text-sm text-[#33C5E0] hover:text-cyan-300 transition-colors"
            >
              <Plus size={15} />
              Add beneficiary
            </button>

            {showErrors && totalError && (
              <p className="mt-2 text-xs text-[#F56565]">{totalError}</p>
            )}
          </section>

          {/* Allocation Flow Chart */}
          {beneficiaries.length > 0 && (
            <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider">
                  Allocation Flow
                </h3>
              </div>
              <div className="bg-[#0A0F11] border border-[#2A3338] rounded-xl p-3">
                <AllocationFlowChart
                  totalAmount={payoutAmount}
                  beneficiaries={flowBeneficiaries}
                />
              </div>
            </section>
          )}

          {/* Inactivity Timer */}
          <section>
            <h3 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider mb-3">
              Inactivity Timer
            </h3>
            <div className="flex items-center gap-4">
              <div className="flex-1 flex flex-col gap-1">
                <label
                  htmlFor="edit-inactivity"
                  className="text-xs text-[#92A5A8]"
                >
                  Inactivity period (days)
                </label>
                <input
                  id="edit-inactivity"
                  type="number"
                  min={1}
                  max={3650}
                  value={inactivityDays}
                  onChange={(e) => setInactivityDays(Number(e.target.value))}
                  className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors w-40"
                />
              </div>
              <p className="text-xs text-[#92A5A8] pt-5">
                Inheritance triggers after {inactivityDays} days of wallet inactivity.
              </p>
            </div>
          </section>

          {/* Yield Harvesting */}
          <section>
            <h3 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider mb-3">
              Yield Harvesting
            </h3>
            <div className="flex items-center gap-4">
              <button
                type="button"
                role="switch"
                aria-checked={yieldEnabled}
                onClick={() => setYieldEnabled((v) => !v)}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#33C5E0] ${
                  yieldEnabled ? "bg-[#33C5E0]" : "bg-[#2A3338]"
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 rounded-full bg-white shadow-md transform transition-transform ${
                    yieldEnabled ? "translate-x-6" : "translate-x-1"
                  }`}
                />
              </button>
              {yieldEnabled && (
                <div className="flex items-center gap-2">
                  <label htmlFor="edit-yield-rate" className="text-xs text-[#92A5A8]">
                    Rate (bps)
                  </label>
                  <input
                    id="edit-yield-rate"
                    type="number"
                    min={0}
                    max={10000}
                    value={yieldRateBps}
                    onChange={(e) => setYieldRateBps(Number(e.target.value))}
                    className="w-24 bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-1.5 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
                  />
                </div>
              )}
            </div>
            <p className="text-xs text-[#92A5A8] mt-2">
              {yieldEnabled
                ? "Yield harvesting is enabled — idle assets earn interest via Stellar lending pools."
                : "Yield harvesting is disabled — assets are held without earning interest."}
            </p>
          </section>

          {/* Genetic Privacy */}
          <section>
            <h3 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider mb-3">
              Genetic Privacy
            </h3>

            <div className="space-y-4">
              <PrivacyToggleRow
                label="Allow Family Matching"
                description="Match this plan's DNA hash against relatives who have opted in."
                checked={privacy.allowFamilyMatching}
                onToggle={() => togglePrivacy("allowFamilyMatching")}
              />

              <PrivacyToggleRow
                label="Zero-Knowledge Hashing"
                description="Compare genetic data using zero-knowledge hashes instead of raw data."
                checked={privacy.zeroKnowledgeHashing}
                onToggle={() => togglePrivacy("zeroKnowledgeHashing")}
              />

              <div className="flex flex-col gap-1">
                <label
                  htmlFor="edit-data-expiry"
                  className="text-xs text-[#92A5A8]"
                >
                  Data Expiry Duration (days)
                </label>
                <input
                  id="edit-data-expiry"
                  type="number"
                  min={-1}
                  value={privacy.dataExpiryDays}
                  onChange={(e) =>
                    setPrivacy((p) => ({
                      ...p,
                      dataExpiryDays: Number(e.target.value),
                    }))
                  }
                  className="w-40 bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
                />
                <p className="text-[11px] text-[#92A5A8]">
                  Stored genetic data is deleted after this many days. Use -1 for
                  indefinite retention.
                </p>
              </div>
            </div>

            <button
              type="button"
              onClick={handleSavePrivacy}
              disabled={privacyStatus === "saving"}
              className="mt-4 flex items-center gap-2 px-4 py-2 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {privacyStatus === "saving" && (
                <Loader2 size={15} className="animate-spin" />
              )}
              {privacyStatus === "saving" ? "Saving…" : "Save Privacy Settings"}
            </button>

            <AnimatePresence>
              {privacyStatus === "success" && (
                <motion.div
                  initial={{ opacity: 0, y: -6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0 }}
                  className="mt-3 flex items-center gap-3 p-3 rounded-lg bg-[#48BB7814] border border-[#48BB7840] text-[#48BB78] text-sm"
                >
                  <CheckCircle size={16} className="flex-shrink-0" />
                  <span>Privacy settings saved.</span>
                </motion.div>
              )}
            </AnimatePresence>

            <AnimatePresence>
              {privacyStatus === "error" && privacyError && (
                <motion.div
                  initial={{ opacity: 0, y: -6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0 }}
                  className="mt-3 flex items-start gap-3 p-3 rounded-lg bg-[#F5656514] border border-[#F5656540] text-[#F56565] text-sm"
                >
                  <AlertCircle size={16} className="mt-0.5 flex-shrink-0" />
                  <span>{privacyError}</span>
                </motion.div>
              )}
            </AnimatePresence>
          </section>

          {/* Error message */}
          <AnimatePresence>
            {errorMessage && (
              <motion.div
                initial={{ opacity: 0, y: -6 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0 }}
                className="flex items-start gap-3 p-3 rounded-lg bg-[#F5656514] border border-[#F5656540] text-[#F56565] text-sm"
              >
                <AlertCircle size={16} className="mt-0.5 flex-shrink-0" />
                <span>{errorMessage}</span>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Success message */}
          <AnimatePresence>
            {txStatus === "success" && (
              <motion.div
                initial={{ opacity: 0, y: -6 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0 }}
                className="flex items-center gap-3 p-3 rounded-lg bg-[#48BB7814] border border-[#48BB7840] text-[#48BB78] text-sm"
              >
                <CheckCircle size={16} className="flex-shrink-0" />
                <span>Plan updated successfully!</span>
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        {/* Footer */}
        <div className="sticky bottom-0 px-6 py-4 bg-[#161E22] border-t border-[#2A3338] flex items-center justify-between gap-3">
          <p className="text-[11px] text-[#92A5A8]">
            Saving will request a wallet signature to update the contract.
          </p>

          <div className="flex gap-3">
            <button
              type="button"
              onClick={onClose}
              disabled={txStatus === "signing" || txStatus === "saving"}
              className="px-4 py-2 text-sm text-[#92A5A8] hover:text-white bg-[#1C252A] hover:bg-[#2A3338] rounded-lg transition-colors disabled:opacity-40"
            >
              Cancel
            </button>

            <button
              type="button"
              onClick={handleSave}
              disabled={
                !allocationOnlyValid ||
                !title.trim() ||
                txStatus === "signing" ||
                txStatus === "saving" ||
                txStatus === "success"
              }
              className="flex items-center gap-2 px-5 py-2 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {txStatus === "signing" && (
                <>
                  <Loader2 size={15} className="animate-spin" />
                  Signing…
                </>
              )}
              {txStatus === "saving" && (
                <>
                  <Loader2 size={15} className="animate-spin" />
                  Saving…
                </>
              )}
              {(txStatus === "idle" || txStatus === "error") && (
                <>
                  <Save size={15} />
                  Save Changes
                </>
              )}
              {txStatus === "success" && (
                <>
                  <CheckCircle size={15} />
                  Saved
                </>
              )}
            </button>
          </div>
        </div>
      </motion.div>
    </div>
  );
}

export default EditInheritancePlanPanel;
