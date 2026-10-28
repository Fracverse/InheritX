"use client";
import { useCallback, useEffect, useId, useRef, useState } from "react";
import { AlertTriangle, ExternalLink, PauseCircle, PlayCircle } from "lucide-react";
import { useWallet } from "@/context/WalletContext";
import {
  invokeContractPause,
  readContractPaused,
  type ContractPauseAction,
  type CreatePlanStage,
} from "@/app/services/inheritanceContractService";

const STAGE_LABELS: Record<CreatePlanStage, string> = {
  building: "Simulating the transaction…",
  "awaiting-signature": "Waiting for your wallet signature…",
  submitting: "Submitting to the Stellar network…",
  confirmed: "Confirmed on-chain.",
};

/** The word the admin must type to confirm each action. */
const CONFIRM_WORD: Record<ContractPauseAction, string> = {
  pause: "PAUSE",
  unpause: "UNPAUSE",
};

/**
 * Emergency pause control for protocol admins/guardians. Pausing calls the
 * inheritance contract's `pause(admin)` as a transaction signed by the
 * connected admin wallet (the contract enforces `require_auth` and the Admin
 * role); unpausing calls `unpause(admin)`. Every action goes through a
 * confirmation dialog first.
 */
export function EmergencyPausePanel() {
  const { address, isConnected, openModal, signTransaction } = useWallet();
  const [paused, setPaused] = useState<boolean | null>(null);
  const [dialogAction, setDialogAction] = useState<ContractPauseAction | null>(null);
  const [stage, setStage] = useState<CreatePlanStage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [explorerUrl, setExplorerUrl] = useState<string | null>(null);

  const refresh = useCallback(() => {
    if (!address) return;
    readContractPaused(address)
      .then(setPaused)
      .catch(() => setPaused(null));
  }, [address]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const busy = stage !== null && stage !== "confirmed";

  const run = async (action: ContractPauseAction) => {
    if (!address) return;
    setDialogAction(null);
    setError(null);
    setExplorerUrl(null);
    try {
      const result = await invokeContractPause({
        admin: address,
        action,
        signTransaction,
        onProgress: ({ stage: next }) => setStage(next),
      });
      setExplorerUrl(result.explorerUrl);
      setPaused(action === "pause");
      refresh();
    } catch (err) {
      setStage(null);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const nextAction: ContractPauseAction = paused ? "unpause" : "pause";

  return (
    <section
      aria-labelledby="emergency-pause-heading"
      className="rounded-xl border border-red-500/20 bg-red-500/5 p-5 space-y-3"
    >
      <div className="flex items-start justify-between gap-4 flex-wrap">
        <div>
          <h2 id="emergency-pause-heading" className="text-sm font-semibold text-white flex items-center gap-2">
            <AlertTriangle size={14} className="text-red-400" aria-hidden="true" />
            Emergency pause
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Contract status:{" "}
            <span data-testid="pause-status" className={paused ? "text-red-400" : "text-emerald-400"}>
              {paused === null ? "unknown" : paused ? "Paused" : "Active"}
            </span>
          </p>
        </div>

        {isConnected && address ? (
          <button
            type="button"
            onClick={() => setDialogAction(nextAction)}
            disabled={busy}
            className={`px-3 py-1.5 text-xs font-medium rounded-md border flex items-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed transition-colors ${
              nextAction === "pause"
                ? "bg-red-500/10 text-red-400 border-red-500/30 hover:bg-red-500/20"
                : "bg-emerald-500/10 text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/20"
            }`}
          >
            {nextAction === "pause" ? <PauseCircle size={14} aria-hidden="true" /> : <PlayCircle size={14} aria-hidden="true" />}
            {nextAction === "pause" ? "Pause contract" : "Resume contract"}
          </button>
        ) : (
          <button
            type="button"
            onClick={openModal}
            className="px-3 py-1.5 text-xs font-medium rounded-md border border-slate-600 text-slate-300 hover:bg-slate-700/40"
          >
            Connect admin wallet
          </button>
        )}
      </div>

      <div role="status" aria-live="polite" className="text-xs">
        {stage && <p className="text-slate-300">{STAGE_LABELS[stage]}</p>}
        {explorerUrl && (
          <a
            href={explorerUrl}
            target="_blank"
            rel="noreferrer"
            className="inline-flex items-center gap-1 text-primary hover:underline"
          >
            View transaction <ExternalLink size={12} aria-hidden="true" />
          </a>
        )}
      </div>
      {error && (
        <p role="alert" className="text-xs text-red-400">
          {error}
        </p>
      )}

      {dialogAction && (
        <ConfirmPauseDialog
          action={dialogAction}
          onCancel={() => setDialogAction(null)}
          onConfirm={() => run(dialogAction)}
        />
      )}
    </section>
  );
}

function ConfirmPauseDialog({
  action,
  onCancel,
  onConfirm,
}: {
  action: ContractPauseAction;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const titleId = useId();
  const descriptionId = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const [typed, setTyped] = useState("");
  const [coAdminConfirmed, setCoAdminConfirmed] = useState(false);
  const word = CONFIRM_WORD[action];
  const ready = typed.trim() === word && coAdminConfirmed;

  useEffect(() => {
    inputRef.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descriptionId}
        className="w-full max-w-md rounded-xl border border-slate-700 bg-slate-900 p-5 space-y-4"
      >
        <h3 id={titleId} className="text-base font-semibold text-white">
          {action === "pause" ? "Pause the inheritance contract?" : "Resume the inheritance contract?"}
        </h3>
        <p id={descriptionId} className="text-sm text-slate-400">
          {action === "pause"
            ? "Every state-changing contract call will be rejected until an admin resumes it."
            : "Contract calls will be accepted again immediately."}{" "}
          Your wallet will be asked to sign the transaction. If the admin account is a multisig
          account, the other signers must also sign before it takes effect.
        </p>

        <label className="flex items-start gap-2 text-sm text-slate-300">
          <input
            type="checkbox"
            checked={coAdminConfirmed}
            onChange={(event) => setCoAdminConfirmed(event.target.checked)}
            className="mt-1"
          />
          A second protocol admin has reviewed and approved this {action}.
        </label>

        <label className="block text-sm text-slate-300">
          Type <strong className="font-mono text-white">{word}</strong> to confirm
          <input
            ref={inputRef}
            value={typed}
            onChange={(event) => setTyped(event.target.value)}
            className="mt-1 w-full rounded-md border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-white"
            autoComplete="off"
          />
        </label>

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="px-3 py-1.5 text-sm rounded-md border border-slate-600 text-slate-300 hover:bg-slate-700/40"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={!ready}
            className="px-3 py-1.5 text-sm rounded-md bg-red-600 text-white hover:bg-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {action === "pause" ? "Pause contract" : "Resume contract"}
          </button>
        </div>
      </div>
    </div>
  );
}
