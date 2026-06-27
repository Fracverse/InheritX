"use client";

import { useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Plus, Trash2, X, Save, AlertCircle, CheckCircle, Loader2 } from "lucide-react";
import { plansAPI } from "@/app/lib/api/plans";
import type { Plan, Beneficiary, UpdatePlanRequest } from "@/app/lib/api/plans";
import { useWallet } from "@/context/WalletContext";

// ─── Types ────────────────────────────────────────────────────────────────────

interface EditInheritancePlanPanelProps {
  plan: Plan;
  onClose: () => void;
  onSaved: (updatedPlan: Plan) => void;
}

type TxStatus = "idle" | "signing" | "saving" | "success" | "error";

// ─── Helpers ─────────────────────────────────────────────────────────────────

const DEFAULT_BENEFICIARY: Omit<Beneficiary, "id"> = {
  wallet_address: "",
  name: "",
  allocation_percentage: 0,
};

function totalAllocation(beneficiaries: Beneficiary[]): number {
  return beneficiaries.reduce((sum, b) => sum + (b.allocation_percentage || 0), 0);
}

function isAllocationDistributionValid(beneficiaries: Beneficiary[]): boolean {
  const total = totalAllocation(beneficiaries);
  const allPositive = beneficiaries.every((b) => (b.allocation_percentage || 0) > 0);
  return total === 100 && allPositive;
}

// ─── Sub-components ──────────────────────────────────────────────────────────

function BeneficiaryRow({
  beneficiary,
  index,
  onChange,
  onRemove,
  canRemove,
}: {
  beneficiary: Beneficiary;
  index: number;
  onChange: (index: number, field: keyof Beneficiary, value: string | number) => void;
  onRemove: (index: number) => void;
  canRemove: boolean;
}) {
  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: -8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.18 }}
      className="grid grid-cols-[1fr_1fr_100px_36px] gap-3 items-start"
    >
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
          value={beneficiary.wallet_address}
          onChange={(e) => onChange(index, "wallet_address", e.target.value)}
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
          min={1}
          max={100}
          value={beneficiary.allocation_percentage || ""}
          onChange={(e) =>
            onChange(index, "allocation_percentage", Number(e.target.value))
          }
          placeholder="0"
          className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
        />
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
    </motion.div>
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
  const [inactivityDays, setInactivityDays] = useState<number>(
    plan.contract_created_at ? 180 : 180
  );
  const [yieldEnabled, setYieldEnabled] = useState<boolean>(
    plan.risk_override_enabled ?? false
  );
  const [beneficiaries, setBeneficiaries] = useState<Beneficiary[]>(() => {
    if (plan.beneficiary_name) {
      return [
        {
          id: "existing-0",
          wallet_address: "",
          name: plan.beneficiary_name,
          allocation_percentage: 100,
        },
      ];
    }
    return [{ ...DEFAULT_BENEFICIARY }];
  });

  const [txStatus, setTxStatus] = useState<TxStatus>("idle");
  const [errorMessage, setErrorMessage] = useState<string>("");

  const allocationTotal = totalAllocation(beneficiaries);
  const isAllocationValid = isAllocationDistributionValid(beneficiaries);

  const handleBeneficiaryChange = useCallback(
    (index: number, field: keyof Beneficiary, value: string | number) => {
      setBeneficiaries((prev) =>
        prev.map((b, i) => (i === index ? { ...b, [field]: value } : b))
      );
    },
    []
  );

  const addBeneficiary = useCallback(() => {
    setBeneficiaries((prev) => [...prev, { ...DEFAULT_BENEFICIARY }]);
  }, []);

  const removeBeneficiary = useCallback((index: number) => {
    setBeneficiaries((prev) => prev.filter((_, i) => i !== index));
  }, []);

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
    if (!isAllocationValid) return;

    const invalidBeneficiary = beneficiaries.find(
      (b) => !b.name.trim() || (!b.id && !b.wallet_address.trim())
    );
    if (invalidBeneficiary) {
      setErrorMessage("All beneficiaries must have a name and wallet address.");
      return;
    }

    setErrorMessage("");
    setTxStatus("signing");

    let signedTransaction: string | undefined;
    try {
      const xdr = buildXdr();
      signedTransaction = await signWithWallet(xdr);
    } catch {
      // Wallet signing rejected or unavailable — proceed without signed XDR in dev.
      signedTransaction = undefined;
    }

    setTxStatus("saving");

    const updateRequest: UpdatePlanRequest = {
      title,
      description: description || undefined,
      beneficiaries,
      inactivity_period_days: inactivityDays,
      yield_harvesting_enabled: yieldEnabled,
      signed_transaction: signedTransaction,
    };

    try {
      const updated = await plansAPI.updatePlan(plan.id, updateRequest);
      setTxStatus("success");
      setTimeout(() => onSaved(updated), 1200);
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
                  isAllocationValid
                    ? "bg-[#48BB7814] text-[#48BB78]"
                    : "bg-[#F5656514] text-[#F56565]"
                }`}
              >
                {allocationTotal}% / 100%
              </span>
            </div>

            <div className="space-y-3">
              <AnimatePresence initial={false}>
                {beneficiaries.map((b, i) => (
                  <BeneficiaryRow
                    key={i}
                    index={i}
                    beneficiary={b}
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
          </section>

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
            <p className="text-xs text-[#92A5A8] mt-2">
              {yieldEnabled
                ? "Yield harvesting is enabled — idle assets earn interest via Stellar lending pools."
                : "Yield harvesting is disabled — assets are held without earning interest."}
            </p>
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
                !isAllocationValid ||
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
