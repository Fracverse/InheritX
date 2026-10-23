"use client";

import { useCallback, useMemo, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { AlertCircle, CheckCircle, ExternalLink, Loader2, Plus } from "lucide-react";
import { plansAPI } from "@/app/lib/api/plans";
import type { CreatePlanRequest } from "@/app/lib/api/plans";
import {
  getSelectedTokenIdentifier,
  isValidTokenIdentifier,
} from "@/app/lib/validation/inheritancePlan";
import { useWallet } from "@/context/WalletContext";
import {
  invokeCreateInheritancePlan,
  ContractSimulationError,
  TransactionSubmissionError,
  type CreatePlanStage,
} from "@/app/services/inheritanceContractService";
import { DISTRIBUTION_METHODS, type DistributionMethod } from "@/app/lib/stellar/contractParams";
import { CrossChainDepositSection } from "./CrossChainDepositSection";
import {
  BeneficiaryAllocationRow,
  DEFAULT_BENEFICIARY_DRAFT,
  beneficiaryDraftToContractInput,
  beneficiaryDraftToRequest,
  bpsToPercentageLabel,
  generateClaimCode,
  totalAllocationBps,
  validateBeneficiaryDrafts,
  validateContractBeneficiaryDrafts,
  type BeneficiaryDraft,
} from "./BeneficiaryAllocationRow";

const TOKEN_OPTIONS = ["XLM", "USDC", "CUSTOM"] as const;

type SubmitStatus = "idle" | "creating" | "success" | "error";

const STAGE_LABELS: Record<CreatePlanStage, string> = {
  building: "Building transaction…",
  "awaiting-signature": "Awaiting wallet signature…",
  submitting: "Submitting to the Stellar network…",
  confirmed: "Confirmed on-chain.",
};

const DISTRIBUTION_METHOD_LABELS: Record<DistributionMethod, string> = {
  LumpSum: "Lump Sum",
  Monthly: "Monthly",
  Quarterly: "Quarterly",
  Yearly: "Yearly",
};

export function CreateInheritancePlanPanel() {
  const { address, isConnected, openModal, signTransaction } = useWallet();

  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [depositAmount, setDepositAmount] = useState("");
  const [inactivityDays, setInactivityDays] = useState(180);
  const [tokenType, setTokenType] = useState<(typeof TOKEN_OPTIONS)[number]>("XLM");
  const [customTokenAddress, setCustomTokenAddress] = useState("");
  const [distributionMethod, setDistributionMethod] = useState<DistributionMethod>("LumpSum");
  const [isLendable, setIsLendable] = useState(false);
  const [beneficiaries, setBeneficiaries] = useState<BeneficiaryDraft[]>([
    { ...DEFAULT_BENEFICIARY_DRAFT, allocationBps: 10000, claimCode: generateClaimCode() },
  ]);
  const [bridgeTransferId, setBridgeTransferId] = useState<string | null>(null);
  const [createdPlanId, setCreatedPlanId] = useState<string | null>(null);
  const [status, setStatus] = useState<SubmitStatus>("idle");
  const [stage, setStage] = useState<CreatePlanStage | null>(null);
  const [errorMessage, setErrorMessage] = useState("");
  const [touched, setTouched] = useState(false);
  const [txHash, setTxHash] = useState<string | null>(null);
  const [explorerUrl, setExplorerUrl] = useState<string | null>(null);

  const allocationTotalBps = totalAllocationBps(beneficiaries);
  const { rowErrors, totalError } = useMemo(() => {
    const base = validateBeneficiaryDrafts(beneficiaries);
    const onChain = validateContractBeneficiaryDrafts(beneficiaries);
    return {
      // Base (address/name/allocation) errors take priority per row; only
      // fall back to the on-chain-only checks (email/claim code/reference)
      // when the row already clears the base validation.
      rowErrors: { ...onChain.rowErrors, ...base.rowErrors },
      totalError: base.totalError,
    };
  }, [beneficiaries]);
  const beneficiariesValid = Object.keys(rowErrors).length === 0 && !totalError;
  const parsedDeposit = Number.parseFloat(depositAmount) || 0;
  const selectedToken = getSelectedTokenIdentifier(tokenType, customTokenAddress);
  const tokenValid = isValidTokenIdentifier(tokenType, customTokenAddress);

  const handleBeneficiaryChange = useCallback(
    (index: number, field: keyof BeneficiaryDraft, value: string | number | boolean) => {
      setBeneficiaries((prev) =>
        prev.map((b, i) => (i === index ? { ...b, [field]: value } : b))
      );
    },
    []
  );

  const addBeneficiary = () => {
    setBeneficiaries((prev) => [
      ...prev,
      { ...DEFAULT_BENEFICIARY_DRAFT, claimCode: generateClaimCode() },
    ]);
  };

  const removeBeneficiary = (index: number) => {
    setBeneficiaries((prev) => prev.filter((_, i) => i !== index));
  };

  const handleBridgeComplete = useCallback((transferId: string) => {
    setBridgeTransferId(transferId);
  }, []);

  const descriptionValid = description.length <= 500;

  const canSubmit =
    !!title.trim() &&
    !!address &&
    tokenValid &&
    beneficiariesValid &&
    descriptionValid &&
    parsedDeposit > 0 &&
    !!bridgeTransferId;

  const handleCreatePlan = async () => {
    setTouched(true);

    if (!isConnected || !address) {
      openModal();
      return;
    }

    if (!title.trim()) {
      setErrorMessage("Enter a plan title.");
      return;
    }

    if (!tokenValid) {
      setErrorMessage("Choose XLM, USDC, or enter a valid custom Stellar contract address.");
      return;
    }

    if (!beneficiariesValid) {
      setErrorMessage(
        totalError || "Resolve the highlighted beneficiary fields before creating the plan."
      );
      return;
    }

    if (!descriptionValid) {
      setErrorMessage("Description must be 500 characters or fewer.");
      return;
    }

    if (parsedDeposit <= 0) {
      setErrorMessage("Enter a deposit amount greater than zero.");
      return;
    }

    if (!bridgeTransferId) {
      setErrorMessage(
        "Complete the cross-chain deposit before creating the plan."
      );
      return;
    }

    setErrorMessage("");
    setTxHash(null);
    setExplorerUrl(null);
    setStatus("creating");
    setStage("building");

    try {
      const result = await invokeCreateInheritancePlan({
        contractInput: {
          owner: address,
          token: selectedToken,
          planName: title.trim(),
          description: description.trim(),
          amount: depositAmount,
          distributionMethod,
          beneficiaries: beneficiaries.map((b, i) => beneficiaryDraftToContractInput(b, i + 1)),
          isLendable,
        },
        walletAddress: address,
        signTransaction,
        onProgress: (progress) => setStage(progress.stage),
      });

      setTxHash(result.submission.hash);
      setExplorerUrl(result.explorerUrl);
      setStatus("success");

      // Best-effort off-chain record so existing dashboards backed by
      // GET /api/plans keep listing this plan. The on-chain transaction is
      // already final at this point, so a failure here is non-fatal.
      try {
        const request: CreatePlanRequest = {
          owner: address,
          token: selectedToken,
          amount: parsedDeposit,
          beneficiaries: beneficiaries.map(beneficiaryDraftToRequest),
          last_ping: Math.floor(Date.now() / 1000),
          grace_period: inactivityDays * 86400,
          earn_yield: false,
          yield_rate_bps: 0,
          is_active: true,
        };
        const savedPlan = await plansAPI.createPlan(request);
        setCreatedPlanId(savedPlan.id);
      } catch (syncError) {
        console.warn("On-chain plan created, but off-chain sync failed:", syncError);
      }
    } catch (error) {
      setStatus("error");
      if (error instanceof ContractSimulationError || error instanceof TransactionSubmissionError) {
        setErrorMessage(error.message);
      } else {
        setErrorMessage(error instanceof Error ? error.message : "Failed to create plan.");
      }
    } finally {
      setStage(null);
    }
  };

  const showErrors = touched || status === "error";

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
              <div className="flex flex-col gap-1">
                <label className="text-xs text-[#92A5A8]">Destination Asset</label>
                <div className="grid grid-cols-3 gap-2">
                  {TOKEN_OPTIONS.map((token) => (
                    <button
                      key={token}
                      type="button"
                      onClick={() => setTokenType(token)}
                      className={`rounded-lg border px-3 py-2 text-xs font-medium transition-colors ${
                        tokenType === token
                          ? "border-[#33C5E0] bg-[#33C5E014] text-[#33C5E0]"
                          : "border-[#2A3338] text-[#92A5A8] hover:border-[#3A4348]"
                      }`}
                    >
                      {token === "CUSTOM" ? "Custom" : token}
                    </button>
                  ))}
                </div>
                {tokenType === "CUSTOM" && (
                  <input
                    type="text"
                    value={customTokenAddress}
                    onChange={(e) => setCustomTokenAddress(e.target.value)}
                    placeholder="C..."
                    className="mt-1 bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors font-mono"
                  />
                )}
                {showErrors && !tokenValid && (
                  <p className="text-xs text-[#F56565]">
                    Choose XLM, USDC, or enter a valid custom Stellar contract address.
                  </p>
                )}
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
                {beneficiaries.map((beneficiary, index) => (
                  <BeneficiaryAllocationRow
                    key={index}
                    index={index}
                    beneficiary={beneficiary}
                    error={showErrors ? rowErrors[index] : undefined}
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
              className="flex items-center gap-2 text-sm text-[#33C5E0] hover:text-cyan-300 transition-colors"
            >
              <Plus size={15} />
              Add beneficiary
            </button>

            {showErrors && totalError && (
              <p className="text-xs text-[#F56565]">{totalError}</p>
            )}
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

          <section className="space-y-3">
            <h2 className="text-xs font-semibold text-[#33C5E0] uppercase tracking-wider">
              On-Chain Settings
            </h2>
            <div className="flex flex-col gap-1">
              <label className="text-xs text-[#92A5A8]">Distribution method</label>
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                {DISTRIBUTION_METHODS.map((method) => (
                  <button
                    key={method}
                    type="button"
                    onClick={() => setDistributionMethod(method)}
                    className={`rounded-lg border px-3 py-2 text-xs font-medium transition-colors ${
                      distributionMethod === method
                        ? "border-[#33C5E0] bg-[#33C5E014] text-[#33C5E0]"
                        : "border-[#2A3338] text-[#92A5A8] hover:border-[#3A4348]"
                    }`}
                  >
                    {DISTRIBUTION_METHOD_LABELS[method]}
                  </button>
                ))}
              </div>
            </div>
            <label className="flex items-center gap-2 text-xs text-[#92A5A8] cursor-pointer">
              <input
                type="checkbox"
                checked={isLendable}
                onChange={(e) => setIsLendable(e.target.checked)}
                className="h-4 w-4 rounded border-[#2A3338] bg-[#0A0F11] accent-[#33C5E0]"
              />
              Allow this plan&apos;s deposit to earn yield via lending pools
            </label>
          </section>

          <CrossChainDepositSection
            planId={createdPlanId}
            amount={depositAmount}
            onAmountChange={setDepositAmount}
            onBridgeComplete={handleBridgeComplete}
          />

          <AnimatePresence>
            {stage && status === "creating" && (
              <motion.div
                initial={{ opacity: 0, y: -6 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0 }}
                className="flex items-center gap-3 p-3 rounded-lg bg-[#33C5E014] border border-[#33C5E040] text-[#33C5E0] text-sm"
              >
                <Loader2 size={16} className="flex-shrink-0 animate-spin" />
                <span>{STAGE_LABELS[stage]}</span>
              </motion.div>
            )}
          </AnimatePresence>

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
                className="flex items-start gap-3 p-3 rounded-lg bg-[#48BB7814] border border-[#48BB7840] text-[#48BB78] text-sm"
              >
                <CheckCircle size={16} className="mt-0.5 flex-shrink-0" />
                <div className="flex flex-col gap-1">
                  <span>Plan created on-chain and cross-chain deposit locked.</span>
                  {txHash && explorerUrl && (
                    <a
                      href={explorerUrl}
                      target="_blank"
                      rel="noreferrer noopener"
                      className="inline-flex items-center gap-1 text-xs text-[#33C5E0] hover:text-cyan-300 transition-colors font-mono"
                    >
                      {txHash.slice(0, 8)}…{txHash.slice(-8)}
                      <ExternalLink size={12} />
                    </a>
                  )}
                </div>
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
              (touched && !canSubmit) ||
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
