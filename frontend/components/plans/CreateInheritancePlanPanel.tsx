"use client";

import { useCallback, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  AlertCircle,
  CheckCircle,
  Loader2,
  Plus,
  Trash2,
} from "lucide-react";
import { plansAPI } from "@/app/lib/api/plans";
import type { Beneficiary } from "@/app/lib/api/plans";
import { useWallet } from "@/context/WalletContext";
import { CrossChainDepositSection } from "./CrossChainDepositSection";

const DEFAULT_BENEFICIARY: Omit<Beneficiary, "id"> = {
  wallet_address: "",
  name: "",
  allocation_percentage: 100,
};

function totalAllocation(beneficiaries: Beneficiary[]): number {
  return beneficiaries.reduce(
    (sum, b) => sum + (b.allocation_percentage || 0),
    0
  );
}

function isAllocationValid(beneficiaries: Beneficiary[]): boolean {
  const total = totalAllocation(beneficiaries);
  return (
    total === 100 &&
    beneficiaries.every((b) => (b.allocation_percentage || 0) > 0)
  );
}

type SubmitStatus = "idle" | "creating" | "success" | "error";

export function CreateInheritancePlanPanel() {
  const { isConnected, openModal } = useWallet();

  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [depositAmount, setDepositAmount] = useState("");
  const [inactivityDays, setInactivityDays] = useState(180);
  const [beneficiaries, setBeneficiaries] = useState<Beneficiary[]>([
    { ...DEFAULT_BENEFICIARY },
  ]);
  const [bridgeTransferId, setBridgeTransferId] = useState<string | null>(null);
  const [status, setStatus] = useState<SubmitStatus>("idle");
  const [errorMessage, setErrorMessage] = useState("");

  const allocationTotal = totalAllocation(beneficiaries);
  const allocationOk = isAllocationValid(beneficiaries);
  const parsedDeposit = Number.parseFloat(depositAmount) || 0;

  const handleBeneficiaryChange = useCallback(
    (index: number, field: keyof Beneficiary, value: string | number) => {
      setBeneficiaries((prev) =>
        prev.map((b, i) => (i === index ? { ...b, [field]: value } : b))
      );
    },
    []
  );

  const addBeneficiary = () => {
    setBeneficiaries((prev) => [...prev, { ...DEFAULT_BENEFICIARY }]);
  };

  const removeBeneficiary = (index: number) => {
    setBeneficiaries((prev) => prev.filter((_, i) => i !== index));
  };

  const handleBridgeComplete = useCallback((transferId: string) => {
    setBridgeTransferId(transferId);
  }, []);

  const handleCreatePlan = async () => {
    if (!title.trim() || !allocationOk || parsedDeposit <= 0) return;

    if (!isConnected) {
      openModal();
      return;
    }

    if (!bridgeTransferId) {
      setErrorMessage(
        "Complete the cross-chain deposit before creating the plan."
      );
      return;
    }

    setErrorMessage("");
    setStatus("creating");

    try {
      const feeEstimate = parsedDeposit * 0.01;
      await plansAPI.createPlan({
        title: title.trim(),
        description: description.trim() || undefined,
        fee: feeEstimate,
        net_amount: parsedDeposit - feeEstimate,
        currency_preference: "USD",
        two_fa_code: "000000",
        beneficiary_name: beneficiaries[0]?.name,
      });
      setStatus("success");
    } catch (error) {
      setStatus("error");
      setErrorMessage(
        error instanceof Error ? error.message : "Failed to create plan."
      );
    }
  };

  return (
    <div className="max-w-3xl mx-auto space-y-8">
      <div>
        <h1 className="text-2xl font-semibold text-foreground">Create Plan</h1>
        <p className="text-sm text-gray-500 mt-1">
          Lock cross-chain assets into a new inheritance plan on Stellar.
        </p>
      </div>

      <div className="bg-[#161E22] border border-[#2A3338] rounded-2xl overflow-hidden">
        <div className="px-6 py-5 space-y-6">
          <section className="space-y-3">
            <h2 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider">
              Plan Details
            </h2>
            <div className="space-y-3">
              <div className="flex flex-col gap-1">
                <label htmlFor="plan-title" className="text-xs text-[#92A5A8]">
                  Title
                </label>
                <input
                  id="plan-title"
                  type="text"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="Family Trust Plan"
                  className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
                />
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="plan-desc" className="text-xs text-[#92A5A8]">
                  Description (optional)
                </label>
                <textarea
                  id="plan-desc"
                  rows={2}
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 resize-none focus:outline-none focus:border-[#33C5E0] transition-colors"
                />
              </div>
            </div>
          </section>

          <section className="space-y-3">
            <div className="flex items-center justify-between">
              <h2 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider">
                Beneficiaries
              </h2>
              <span
                className={`text-xs font-mono px-2 py-0.5 rounded-full ${
                  allocationOk
                    ? "bg-[#48BB7814] text-[#48BB78]"
                    : "bg-[#F5656514] text-[#F56565]"
                }`}
              >
                {allocationTotal}% / 100%
              </span>
            </div>

            <div className="space-y-3">
              {beneficiaries.map((beneficiary, index) => (
                <div
                  key={index}
                  className="grid grid-cols-1 sm:grid-cols-[1fr_1fr_100px_36px] gap-3 items-start"
                >
                  <div className="flex flex-col gap-1">
                    <label className="text-[10px] text-[#92A5A8] uppercase tracking-wider">
                      Name
                    </label>
                    <input
                      type="text"
                      value={beneficiary.name}
                      onChange={(e) =>
                        handleBeneficiaryChange(index, "name", e.target.value)
                      }
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
                      onChange={(e) =>
                        handleBeneficiaryChange(
                          index,
                          "wallet_address",
                          e.target.value
                        )
                      }
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
                        handleBeneficiaryChange(
                          index,
                          "allocation_percentage",
                          Number(e.target.value)
                        )
                      }
                      className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors"
                    />
                  </div>
                  <button
                    type="button"
                    onClick={() => removeBeneficiary(index)}
                    disabled={beneficiaries.length <= 1}
                    aria-label="Remove beneficiary"
                    className="sm:mt-6 p-2 rounded-lg text-[#F56565] hover:bg-[#F5656514] disabled:opacity-30 disabled:cursor-not-allowed transition-colors justify-self-start"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              ))}
            </div>

            <button
              type="button"
              onClick={addBeneficiary}
              className="flex items-center gap-2 text-sm text-[#33C5E0] hover:text-cyan-300 transition-colors"
            >
              <Plus size={15} />
              Add beneficiary
            </button>
          </section>

          <section className="space-y-3">
            <h2 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider">
              Inactivity Timer
            </h2>
            <div className="flex flex-col sm:flex-row sm:items-end gap-4">
              <div className="flex flex-col gap-1">
                <label htmlFor="inactivity-days" className="text-xs text-[#92A5A8]">
                  Inactivity period (days)
                </label>
                <input
                  id="inactivity-days"
                  type="number"
                  min={1}
                  max={3650}
                  value={inactivityDays}
                  onChange={(e) => setInactivityDays(Number(e.target.value))}
                  className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 focus:outline-none focus:border-[#33C5E0] transition-colors w-full sm:w-40"
                />
              </div>
              <p className="text-xs text-[#92A5A8] pb-1">
                Inheritance triggers after {inactivityDays} days of wallet inactivity.
              </p>
            </div>
          </section>

          <CrossChainDepositSection
            amount={depositAmount}
            onAmountChange={setDepositAmount}
            onBridgeComplete={handleBridgeComplete}
          />

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

          <AnimatePresence>
            {status === "success" && (
              <motion.div
                initial={{ opacity: 0, y: -6 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0 }}
                className="flex items-center gap-3 p-3 rounded-lg bg-[#48BB7814] border border-[#48BB7840] text-[#48BB78] text-sm"
              >
                <CheckCircle size={16} className="flex-shrink-0" />
                <span>Plan created successfully with cross-chain deposit locked.</span>
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        <div className="px-6 py-4 bg-[#161E22] border-t border-[#2A3338] flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
          <p className="text-[11px] text-[#92A5A8]">
            {bridgeTransferId
              ? "Deposit confirmed — ready to create plan."
              : "Bridge assets first, then create your inheritance plan."}
          </p>
          <button
            type="button"
            onClick={handleCreatePlan}
            disabled={
              !title.trim() ||
              !allocationOk ||
              parsedDeposit <= 0 ||
              !bridgeTransferId ||
              status === "creating" ||
              status === "success"
            }
            className="inline-flex items-center justify-center gap-2 px-5 py-2.5 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed w-full sm:w-auto"
          >
            {status === "creating" ? (
              <>
                <Loader2 size={15} className="animate-spin" />
                Creating plan…
              </>
            ) : (
              "Create Plan"
            )}
          </button>
        </div>
      </div>
    </div>
  );
}

export default CreateInheritancePlanPanel;
