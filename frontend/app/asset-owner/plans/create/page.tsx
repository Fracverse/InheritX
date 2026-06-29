"use client";

import { useMemo, useState } from "react";
import {
  AlertCircle,
  ArrowLeft,
  ArrowRight,
  Check,
  CheckCircle2,
  Loader2,
  Plus,
  Trash2,
  Wallet,
} from "lucide-react";
import inheritanceAPI from "@/app/lib/api/inheritance";
import type { PlanBeneficiaryRequest } from "@/app/lib/api/inheritance";
import {
  getSelectedTokenIdentifier,
  percentageToBasisPoints,
  validateInheritancePlanDraft,
  type DraftBeneficiary,
  type InheritancePlanDraft,
} from "@/app/lib/validation/inheritancePlan";
import { invokeCreatePlan } from "@/app/services/inheritanceContractService";
import { useWallet } from "@/context/WalletContext";
import { formatAddress } from "@/util/address";

const STEP_LABELS = ["Asset", "Amount", "Beneficiaries", "Confirm"];
const DEFAULT_BENEFICIARY: DraftBeneficiary = {
  address: "",
  name: "",
  allocationPercentage: 100,
};

type SubmissionState = "idle" | "signing" | "submitting" | "success" | "error";

function fieldClass(hasError?: boolean) {
  return `rounded-lg border bg-[#0A0F11] px-4 py-3 text-sm text-slate-100 outline-none transition-colors placeholder:text-slate-600 focus:border-primary ${
    hasError ? "border-red-400/70" : "border-white/10"
  }`;
}

function ErrorText({ message }: { message?: string }) {
  if (!message) return null;
  return <p className="mt-1 text-xs text-red-300">{message}</p>;
}

export default function CreateInheritancePlanPage() {
  const { address, isConnected, openModal, kit, selectedWalletId } = useWallet();
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState<InheritancePlanDraft>({
    owner: address ?? "",
    tokenType: "XLM",
    customTokenAddress: "",
    amount: "",
    earnYield: false,
    beneficiaries: [{ ...DEFAULT_BENEFICIARY }],
    gracePeriodDays: 180,
    timelockDays: 7,
  });
  const [touched, setTouched] = useState(false);
  const [submissionState, setSubmissionState] = useState<SubmissionState>("idle");
  const [submitMessage, setSubmitMessage] = useState("");
  const [createdPlanId, setCreatedPlanId] = useState<string | null>(null);

  const hydratedDraft = useMemo(
    () => ({ ...draft, owner: address ?? draft.owner }),
    [address, draft]
  );
  const validation = useMemo(
    () => validateInheritancePlanDraft(hydratedDraft),
    [hydratedDraft]
  );
  const allocationTotal = draft.beneficiaries.reduce(
    (sum, beneficiary) => sum + (Number(beneficiary.allocationPercentage) || 0),
    0
  );
  const selectedToken = getSelectedTokenIdentifier(
    draft.tokenType,
    draft.customTokenAddress
  );

  const updateDraft = <K extends keyof InheritancePlanDraft>(
    key: K,
    value: InheritancePlanDraft[K]
  ) => {
    setDraft((current) => ({ ...current, [key]: value }));
  };

  const updateBeneficiary = (
    index: number,
    key: keyof DraftBeneficiary,
    value: string | number
  ) => {
    setDraft((current) => ({
      ...current,
      beneficiaries: current.beneficiaries.map((beneficiary, rowIndex) =>
        rowIndex === index ? { ...beneficiary, [key]: value } : beneficiary
      ),
    }));
  };

  const addBeneficiary = () => {
    setDraft((current) => ({
      ...current,
      beneficiaries: [
        ...current.beneficiaries,
        { ...DEFAULT_BENEFICIARY, allocationPercentage: 0 },
      ],
    }));
  };

  const removeBeneficiary = (index: number) => {
    setDraft((current) => ({
      ...current,
      beneficiaries:
        current.beneficiaries.length === 1
          ? current.beneficiaries
          : current.beneficiaries.filter((_, rowIndex) => rowIndex !== index),
    }));
  };

  const canLeaveStep = () => {
    if (step === 0) return !validation.errors.owner && !validation.errors.token;
    if (step === 1)
      return (
        !validation.errors.amount &&
        !validation.errors.gracePeriodDays &&
        !validation.errors.timelockDays
      );
    if (step === 2)
      return !Object.keys(validation.errors).some((key) =>
        key.startsWith("beneficiary")
      ) && !validation.errors.allocationTotal;
    return validation.isValid;
  };

  const handleNext = () => {
    setTouched(true);
    if (!canLeaveStep()) return;
    setTouched(false);
    setStep((current) => Math.min(current + 1, STEP_LABELS.length - 1));
  };

  const handleBack = () => {
    setTouched(false);
    setStep((current) => Math.max(current - 1, 0));
  };

  const buildBeneficiaries = (): PlanBeneficiaryRequest[] =>
    hydratedDraft.beneficiaries.map((beneficiary) => ({
      address: beneficiary.address.trim(),
      name: beneficiary.name.trim(),
      allocation_bps: percentageToBasisPoints(beneficiary.allocationPercentage),
      fiat_anchor_info: "",
    }));

  const handleSubmit = async () => {
    setTouched(true);
    if (!validation.isValid) {
      setSubmitMessage("Resolve the highlighted fields before creating the plan.");
      return;
    }

    setSubmitMessage("");
    setSubmissionState(kit && selectedWalletId ? "signing" : "submitting");

    try {
      const contractResult = await invokeCreatePlan({
        contractInput: {
          owner: hydratedDraft.owner,
          token: selectedToken,
          amount: Number(hydratedDraft.amount),
          beneficiaries: buildBeneficiaries(),
          gracePeriodDays: hydratedDraft.gracePeriodDays,
          earnYield: hydratedDraft.earnYield,
          timelockDays: hydratedDraft.timelockDays,
        },
        kit,
        selectedWalletId,
      });

      setSubmissionState("submitting");
      const created = await inheritanceAPI.createPlan(contractResult.request, {
        headers: {
          "X-Contract-Method": "create_plan",
          ...(contractResult.signedTransactionXdr
            ? { "X-Signed-Transaction-XDR": contractResult.signedTransactionXdr }
            : {}),
        },
      });

      setCreatedPlanId(created.id);
      setSubmissionState("success");
      setSubmitMessage("Inheritance plan created successfully.");
    } catch (error) {
      setSubmissionState("error");
      setSubmitMessage(
        error instanceof Error ? error.message : "Failed to create inheritance plan."
      );
    }
  };

  const showErrors = touched || submissionState === "error";

  return (
    <div className="animate-fade-in space-y-6">
      <div>
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-primary">
          New inheritance plan
        </p>
        <div className="mt-2 flex flex-col gap-2 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <h1 className="text-2xl font-semibold text-foreground">
              Create Inheritance Plan
            </h1>
            <p className="mt-1 max-w-2xl text-sm text-gray-500">
              Configure the asset, lock amount, beneficiaries, inactivity timer, and
              final transaction details before invoking the Soroban contract.
            </p>
          </div>
          <div className="rounded-lg border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-gray-400">
            {isConnected && address ? (
              <span className="font-mono text-primary">{formatAddress(address)}</span>
            ) : (
              <button
                type="button"
                onClick={openModal}
                className="inline-flex items-center gap-2 text-primary"
              >
                <Wallet size={16} />
                Connect wallet
              </button>
            )}
          </div>
        </div>
      </div>

      <section className="rounded-xl border border-white/10 bg-white/[0.03] p-5">
        <div className="grid gap-3 sm:grid-cols-4">
          {STEP_LABELS.map((label, index) => {
            const isActive = index === step;
            const isComplete = index < step;
            return (
              <button
                key={label}
                type="button"
                onClick={() => setStep(index)}
                className={`flex items-center gap-3 rounded-lg border px-3 py-3 text-left transition-colors ${
                  isActive
                    ? "border-primary/60 bg-primary/10 text-primary"
                    : isComplete
                      ? "border-emerald-400/30 bg-emerald-400/10 text-emerald-300"
                      : "border-white/10 bg-black/10 text-gray-500"
                }`}
              >
                <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-current text-xs font-semibold">
                  {isComplete ? <Check size={14} /> : index + 1}
                </span>
                <span className="text-sm font-medium">{label}</span>
              </button>
            );
          })}
        </div>
      </section>

      <section className="rounded-xl border border-white/10 bg-[#0d1117] p-5">
        {step === 0 && (
          <div className="space-y-5">
            <div>
              <h2 className="text-lg font-semibold text-white">Select token type</h2>
              <p className="mt-1 text-sm text-gray-500">
                Choose a native or Stellar asset to lock in the plan.
              </p>
            </div>

            <div className="grid gap-3 md:grid-cols-3">
              {["XLM", "USDC", "CUSTOM"].map((token) => (
                <button
                  key={token}
                  type="button"
                  onClick={() => updateDraft("tokenType", token)}
                  className={`rounded-lg border px-4 py-4 text-left ${
                    draft.tokenType === token
                      ? "border-primary bg-primary/10 text-primary"
                      : "border-white/10 bg-white/[0.03] text-slate-300 hover:border-white/20"
                  }`}
                >
                  <span className="text-sm font-semibold">
                    {token === "CUSTOM" ? "Custom asset" : token}
                  </span>
                  <span className="mt-1 block text-xs text-gray-500">
                    {token === "XLM"
                      ? "Native Stellar lumens"
                      : token === "USDC"
                        ? "Common Stellar USDC asset"
                        : "Paste a Soroban token contract address"}
                  </span>
                </button>
              ))}
            </div>

            {draft.tokenType === "CUSTOM" && (
              <div>
                <label className="text-xs uppercase tracking-wider text-gray-500">
                  Custom token contract address
                </label>
                <input
                  value={draft.customTokenAddress}
                  onChange={(event) =>
                    updateDraft("customTokenAddress", event.target.value)
                  }
                  placeholder="C..."
                  className={`mt-2 w-full font-mono ${fieldClass(
                    showErrors && !!validation.errors.token
                  )}`}
                />
              </div>
            )}

            <ErrorText message={showErrors ? validation.errors.token : undefined} />
            <ErrorText message={showErrors ? validation.errors.owner : undefined} />
          </div>
        )}

        {step === 1 && (
          <div className="space-y-5">
            <div>
              <h2 className="text-lg font-semibold text-white">
                Amount and inactivity settings
              </h2>
              <p className="mt-1 text-sm text-gray-500">
                Set the locked balance, yield behavior, grace period, and timelock.
              </p>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <div>
                <label className="text-xs uppercase tracking-wider text-gray-500">
                  Amount to lock
                </label>
                <input
                  type="number"
                  min="0"
                  step="0.0000001"
                  value={draft.amount}
                  onChange={(event) => updateDraft("amount", event.target.value)}
                  placeholder="0.00"
                  className={`mt-2 w-full ${fieldClass(
                    showErrors && !!validation.errors.amount
                  )}`}
                />
                <ErrorText message={showErrors ? validation.errors.amount : undefined} />
              </div>

              <div>
                <label className="text-xs uppercase tracking-wider text-gray-500">
                  Grace period
                </label>
                <div className="mt-2 flex items-center gap-3">
                  <input
                    type="number"
                    min={1}
                    value={draft.gracePeriodDays}
                    onChange={(event) =>
                      updateDraft("gracePeriodDays", Number(event.target.value))
                    }
                    className={`w-full ${fieldClass(
                      showErrors && !!validation.errors.gracePeriodDays
                    )}`}
                  />
                  <span className="text-sm text-gray-500">days</span>
                </div>
                <ErrorText
                  message={showErrors ? validation.errors.gracePeriodDays : undefined}
                />
              </div>

              <div>
                <label className="text-xs uppercase tracking-wider text-gray-500">
                  Claim timelock
                </label>
                <div className="mt-2 flex items-center gap-3">
                  <input
                    type="number"
                    min={0}
                    value={draft.timelockDays}
                    onChange={(event) =>
                      updateDraft("timelockDays", Number(event.target.value))
                    }
                    className={`w-full ${fieldClass(
                      showErrors && !!validation.errors.timelockDays
                    )}`}
                  />
                  <span className="text-sm text-gray-500">days</span>
                </div>
                <ErrorText
                  message={showErrors ? validation.errors.timelockDays : undefined}
                />
              </div>

              <div className="rounded-lg border border-white/10 bg-white/[0.03] p-4">
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <p className="text-sm font-medium text-slate-200">Earn Yield</p>
                    <p className="mt-1 text-xs text-gray-500">
                      Route idle assets through yield-aware contract settings.
                    </p>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    aria-checked={draft.earnYield}
                    onClick={() => updateDraft("earnYield", !draft.earnYield)}
                    className={`relative h-7 w-12 rounded-full transition-colors ${
                      draft.earnYield ? "bg-primary" : "bg-white/15"
                    }`}
                  >
                    <span
                      className={`absolute top-1 h-5 w-5 rounded-full bg-white transition-transform ${
                        draft.earnYield ? "translate-x-6" : "translate-x-1"
                      }`}
                    />
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-5">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
              <div>
                <h2 className="text-lg font-semibold text-white">Add beneficiaries</h2>
                <p className="mt-1 text-sm text-gray-500">
                  Beneficiary allocations must total exactly 100%.
                </p>
              </div>
              <div
                className={`w-fit rounded-full px-3 py-1 text-xs font-semibold ${
                  allocationTotal === 100
                    ? "bg-emerald-400/10 text-emerald-300"
                    : "bg-red-400/10 text-red-300"
                }`}
              >
                {allocationTotal}% allocated
              </div>
            </div>

            <div className="space-y-3">
              {draft.beneficiaries.map((beneficiary, index) => (
                <div
                  key={index}
                  className="grid gap-3 rounded-lg border border-white/10 bg-white/[0.03] p-4 lg:grid-cols-[1fr_1.4fr_140px_40px]"
                >
                  <div>
                    <label className="text-xs uppercase tracking-wider text-gray-500">
                      Name
                    </label>
                    <input
                      value={beneficiary.name}
                      onChange={(event) =>
                        updateBeneficiary(index, "name", event.target.value)
                      }
                      placeholder="Ada Lovelace"
                      className={`mt-2 w-full ${fieldClass(
                        showErrors && !!validation.errors[`beneficiary.${index}.name`]
                      )}`}
                    />
                    <ErrorText
                      message={
                        showErrors
                          ? validation.errors[`beneficiary.${index}.name`]
                          : undefined
                      }
                    />
                  </div>

                  <div>
                    <label className="text-xs uppercase tracking-wider text-gray-500">
                      Stellar address
                    </label>
                    <input
                      value={beneficiary.address}
                      onChange={(event) =>
                        updateBeneficiary(index, "address", event.target.value)
                      }
                      placeholder="G..."
                      className={`mt-2 w-full font-mono ${fieldClass(
                        showErrors && !!validation.errors[`beneficiary.${index}.address`]
                      )}`}
                    />
                    <ErrorText
                      message={
                        showErrors
                          ? validation.errors[`beneficiary.${index}.address`]
                          : undefined
                      }
                    />
                  </div>

                  <div>
                    <label className="text-xs uppercase tracking-wider text-gray-500">
                      Allocation
                    </label>
                    <div className="mt-2 flex items-center gap-2">
                      <input
                        type="number"
                        min={1}
                        max={100}
                        value={beneficiary.allocationPercentage || ""}
                        onChange={(event) =>
                          updateBeneficiary(
                            index,
                            "allocationPercentage",
                            Number(event.target.value)
                          )
                        }
                        className={`w-full ${fieldClass(
                          showErrors &&
                            !!validation.errors[
                              `beneficiary.${index}.allocationPercentage`
                            ]
                        )}`}
                      />
                      <span className="text-sm text-gray-500">%</span>
                    </div>
                    <ErrorText
                      message={
                        showErrors
                          ? validation.errors[
                              `beneficiary.${index}.allocationPercentage`
                            ]
                          : undefined
                      }
                    />
                  </div>

                  <button
                    type="button"
                    onClick={() => removeBeneficiary(index)}
                    disabled={draft.beneficiaries.length === 1}
                    aria-label={`Remove beneficiary ${index + 1}`}
                    className="mt-7 flex h-10 w-10 items-center justify-center rounded-lg text-red-300 hover:bg-red-400/10 disabled:cursor-not-allowed disabled:opacity-30"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              ))}
            </div>

            <button
              type="button"
              onClick={addBeneficiary}
              className="inline-flex items-center gap-2 rounded-lg border border-primary/40 px-4 py-2 text-sm font-medium text-primary hover:bg-primary/10"
            >
              <Plus size={16} />
              Add beneficiary
            </button>

            <ErrorText
              message={
                showErrors
                  ? validation.errors.allocationTotal || validation.errors.beneficiaries
                  : undefined
              }
            />
          </div>
        )}

        {step === 3 && (
          <div className="space-y-5">
            <div>
              <h2 className="text-lg font-semibold text-white">Review and confirm</h2>
              <p className="mt-1 text-sm text-gray-500">
                This summary is used to invoke the smart contract method{" "}
                <span className="font-mono text-primary">create_plan</span>.
              </p>
            </div>

            <div className="grid gap-4 lg:grid-cols-2">
              <div className="rounded-lg border border-white/10 bg-white/[0.03] p-4">
                <p className="text-xs uppercase tracking-wider text-gray-500">
                  Asset lock
                </p>
                <dl className="mt-3 space-y-2 text-sm">
                  <div className="flex justify-between gap-3">
                    <dt className="text-gray-500">Token</dt>
                    <dd className="font-mono text-slate-200">{selectedToken}</dd>
                  </div>
                  <div className="flex justify-between gap-3">
                    <dt className="text-gray-500">Amount</dt>
                    <dd className="text-slate-200">{draft.amount || "0"}</dd>
                  </div>
                  <div className="flex justify-between gap-3">
                    <dt className="text-gray-500">Earn Yield</dt>
                    <dd className="text-slate-200">{draft.earnYield ? "Yes" : "No"}</dd>
                  </div>
                </dl>
              </div>

              <div className="rounded-lg border border-white/10 bg-white/[0.03] p-4">
                <p className="text-xs uppercase tracking-wider text-gray-500">
                  Timing
                </p>
                <dl className="mt-3 space-y-2 text-sm">
                  <div className="flex justify-between gap-3">
                    <dt className="text-gray-500">Grace period</dt>
                    <dd className="text-slate-200">{draft.gracePeriodDays} days</dd>
                  </div>
                  <div className="flex justify-between gap-3">
                    <dt className="text-gray-500">Claim timelock</dt>
                    <dd className="text-slate-200">{draft.timelockDays} days</dd>
                  </div>
                  <div className="flex justify-between gap-3">
                    <dt className="text-gray-500">Owner</dt>
                    <dd className="font-mono text-slate-200">
                      {hydratedDraft.owner
                        ? formatAddress(hydratedDraft.owner)
                        : "Not connected"}
                    </dd>
                  </div>
                </dl>
              </div>
            </div>

            <div className="rounded-lg border border-white/10 bg-white/[0.03] p-4">
              <p className="text-xs uppercase tracking-wider text-gray-500">
                Beneficiaries
              </p>
              <div className="mt-3 divide-y divide-white/10">
                {draft.beneficiaries.map((beneficiary, index) => (
                  <div
                    key={`${beneficiary.address}-${index}`}
                    className="grid gap-2 py-3 text-sm md:grid-cols-[1fr_1.5fr_120px]"
                  >
                    <span className="text-slate-200">{beneficiary.name || "Unnamed"}</span>
                    <span className="font-mono text-gray-400">
                      {beneficiary.address || "No address"}
                    </span>
                    <span className="text-right text-primary">
                      {beneficiary.allocationPercentage || 0}%
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {submitMessage && (
              <div
                className={`flex items-start gap-3 rounded-lg border p-3 text-sm ${
                  submissionState === "success"
                    ? "border-emerald-400/30 bg-emerald-400/10 text-emerald-300"
                    : "border-red-400/30 bg-red-400/10 text-red-300"
                }`}
              >
                {submissionState === "success" ? (
                  <CheckCircle2 size={16} className="mt-0.5 shrink-0" />
                ) : (
                  <AlertCircle size={16} className="mt-0.5 shrink-0" />
                )}
                <span>
                  {submitMessage}
                  {createdPlanId ? (
                    <span className="ml-1 font-mono">Plan ID: {createdPlanId}</span>
                  ) : null}
                </span>
              </div>
            )}
          </div>
        )}

        <div className="mt-8 flex flex-col-reverse gap-3 border-t border-white/10 pt-5 sm:flex-row sm:items-center sm:justify-between">
          <button
            type="button"
            onClick={handleBack}
            disabled={step === 0 || submissionState === "signing" || submissionState === "submitting"}
            className="inline-flex items-center justify-center gap-2 rounded-lg bg-white/5 px-4 py-2.5 text-sm font-medium text-gray-300 hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-40"
          >
            <ArrowLeft size={16} />
            Back
          </button>

          {step < STEP_LABELS.length - 1 ? (
            <button
              type="button"
              onClick={handleNext}
              className="inline-flex items-center justify-center gap-2 rounded-lg bg-primary px-5 py-2.5 text-sm font-semibold text-black hover:bg-cyan-300"
            >
              Continue
              <ArrowRight size={16} />
            </button>
          ) : (
            <button
              type="button"
              onClick={handleSubmit}
              disabled={
                submissionState === "signing" ||
                submissionState === "submitting" ||
                submissionState === "success"
              }
              className="inline-flex items-center justify-center gap-2 rounded-lg bg-primary px-5 py-2.5 text-sm font-semibold text-black hover:bg-cyan-300 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {submissionState === "signing" || submissionState === "submitting" ? (
                <>
                  <Loader2 size={16} className="animate-spin" />
                  {submissionState === "signing" ? "Signing" : "Creating"}
                </>
              ) : submissionState === "success" ? (
                <>
                  <Check size={16} />
                  Created
                </>
              ) : (
                <>
                  Invoke create_plan
                  <ArrowRight size={16} />
                </>
              )}
            </button>
          )}
        </div>
      </section>
    </div>
  );
}
