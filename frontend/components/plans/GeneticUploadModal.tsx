"use client";

import { useEffect, useRef, useState } from "react";
import { Upload, X, Loader2 } from "lucide-react";
import { useWallet } from "@/context/WalletContext";
import {
  GENETIC_FILE_ACCEPT,
  GeneticVerificationAPI,
  validateGeneticFile,
  type GeneticUploadStage,
} from "@/app/lib/api/geneticVerification";

const api = new GeneticVerificationAPI(
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080",
);
const stageLabels: Record<GeneticUploadStage, string> = {
  hashing: "Preparing your file locally",
  signing: "Approve the proof in your wallet",
  uploading: "Submitting your signed proof",
};

export function GeneticUploadModal({
  planId,
  planTitle,
  onClose,
}: {
  planId: string;
  planTitle: string;
  onClose: () => void;
}) {
  const { address, kit, networkPassphrase } = useWallet();
  const dialog = useRef<HTMLDialogElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const active = useRef<AbortController | null>(null);
  const [file, setFile] = useState<File | null>(null);
  const [stage, setStage] = useState<GeneticUploadStage | null>(null);
  const [error, setError] = useState("");
  const [submitted, setSubmitted] = useState(false);
  const [dragging, setDragging] = useState(false);

  useEffect(() => {
    const element = dialog.current;
    const previousFocus =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    const previousOverflow = document.body.style.overflow;
    element?.showModal();
    document.body.style.overflow = "hidden";
    return () => {
      active.current?.abort();
      element?.close();
      document.body.style.overflow = previousOverflow;
      if (previousFocus?.isConnected) previousFocus.focus();
    };
  }, []);

  useEffect(
    () => () => {
      active.current?.abort();
    },
    [address, kit, networkPassphrase, planId],
  );

  const chooseFile = (files: FileList | File[]) => {
    if (active.current) return;
    setError("");
    setSubmitted(false);
    setFile(null);
    try {
      if (files.length !== 1)
        throw new Error("Choose one genetic file at a time.");
      validateGeneticFile(files[0]);
      setFile(files[0]);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Unable to read this file.",
      );
    }
  };

  const close = () => {
    active.current?.abort();
    onClose();
  };

  const submit = async () => {
    if (!file || active.current || submitted) return;
    if (!address || !kit) {
      setError("Connect your wallet before submitting a genetic proof.");
      return;
    }
    const controller = new AbortController();
    active.current = controller;
    setError("");
    try {
      await api.uploadGeneticData(planId, file, {
        address,
        networkPassphrase,
        signal: controller.signal,
        onStage: setStage,
        signPayload: async (payload) => {
          const result = await kit.signMessage(payload, {
            address,
            networkPassphrase,
          });
          if (result.signerAddress !== address)
            throw new Error("Wallet account changed. Please try again.");
          return result.signedMessage;
        },
      });
      if (!controller.signal.aborted) {
        setSubmitted(true);
        setFile(null);
      }
    } catch (err) {
      if (!controller.signal.aborted) {
        setError(
          err instanceof Error
            ? err.message
            : "Submission failed. Please try again.",
        );
      }
    } finally {
      if (active.current === controller) {
        active.current = null;
        setStage(null);
      }
    }
  };

  return (
    <dialog
      ref={dialog}
      aria-labelledby="genetic-upload-title"
      aria-describedby="genetic-upload-description"
      onCancel={(event) => {
        event.preventDefault();
        close();
      }}
      onClick={(event) => {
        if (event.target === dialog.current) close();
      }}
      className="m-auto w-[calc(100%-2rem)] max-w-lg max-h-[90dvh] overflow-y-auto rounded-2xl border border-[#2A3338] bg-[#161E22] p-6 text-white shadow-2xl backdrop:bg-black/70"
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 id="genetic-upload-title" className="text-lg font-semibold">
            Upload genetic proof
          </h2>
          <p className="mt-1 break-words text-sm text-[#92A5A8]">{planTitle}</p>
        </div>
        <button
          type="button"
          onClick={close}
          aria-label="Close genetic upload"
          className="rounded-lg p-2 hover:bg-[#2A3338] focus-visible:outline-2 focus-visible:outline-cyan-300"
        >
          <X size={20} />
        </button>
      </div>
      <p
        id="genetic-upload-description"
        className="mt-4 text-sm text-[#92A5A8]"
      >
        Your file stays on this device. We submit its fingerprint and your
        wallet signature on Testnet. Submission does not confirm a genetic
        match.
      </p>
      {!submitted && (
        <>
          <div
            className={`mt-5 rounded-xl border-2 border-dashed p-6 text-center ${dragging ? "border-cyan-300 bg-cyan-950" : "border-[#4A5568]"}`}
            onDragOver={(event) => {
              event.preventDefault();
              if (!stage) setDragging(true);
            }}
            onDragLeave={() => setDragging(false)}
            onDrop={(event) => {
              event.preventDefault();
              setDragging(false);
              chooseFile(event.dataTransfer.files);
            }}
          >
            <Upload
              className="mx-auto mb-3 text-[#33C5E0]"
              aria-hidden="true"
            />
            <label htmlFor="genetic-file" className="block text-sm font-medium">
              Choose a file or drag it here
            </label>
            <p className="mt-2 text-xs text-[#92A5A8]">
              .vcf, .23andme, .fasta · Maximum 25 MB
            </p>
            <button
              type="button"
              disabled={!!stage}
              onClick={() => input.current?.click()}
              className="mt-4 rounded-lg bg-[#33C5E0] px-3 py-2 text-sm text-black disabled:opacity-40"
            >
              Browse files
            </button>
            <input
              ref={input}
              id="genetic-file"
              type="file"
              accept={GENETIC_FILE_ACCEPT}
              disabled={!!stage}
              className="sr-only"
              tabIndex={-1}
              onChange={(event) => {
                if (event.target.files?.length) chooseFile(event.target.files);
                event.target.value = "";
              }}
            />
          </div>
          {file && (
            <div className="mt-3 flex items-center justify-between gap-3 text-sm">
              <span className="min-w-0 break-all">
                {file.name}{" "}
                <span className="text-[#92A5A8]">
                  (
                  {file.size < 1024
                    ? `${file.size} bytes`
                    : `${(file.size / 1024 / 1024).toFixed(2)} MB`}
                  )
                </span>
              </span>
              <button
                type="button"
                disabled={!!stage}
                onClick={() => setFile(null)}
                className="text-[#33C5E0] disabled:opacity-40"
              >
                Remove
              </button>
            </div>
          )}
        </>
      )}
      {stage && (
        <div
          role="status"
          className="mt-4 flex items-center gap-2 text-sm text-[#33C5E0]"
        >
          <Loader2
            size={18}
            className="animate-spin motion-reduce:animate-none"
          />
          {stageLabels[stage]}
        </div>
      )}
      {error && (
        <div
          role="alert"
          className="mt-4 rounded-lg border border-red-400/40 bg-red-950/40 p-3 text-sm text-red-200"
        >
          {error}
        </div>
      )}
      {submitted && (
        <p
          role="status"
          className="mt-5 rounded-lg bg-emerald-950 p-4 text-sm text-emerald-200"
        >
          Proof submitted. The verification service will determine the result.
        </p>
      )}
      <div className="mt-6 flex flex-wrap justify-end gap-3">
        <button
          type="button"
          onClick={close}
          className="rounded-lg border border-[#4A5568] px-4 py-2 text-sm"
        >
          {submitted ? "Done" : "Cancel"}
        </button>
        {!submitted && (
          <button
            type="button"
            onClick={submit}
            disabled={!file || !!stage || !address}
            className="rounded-lg bg-[#33C5E0] px-4 py-2 text-sm font-medium text-black disabled:opacity-40"
          >
            Sign and submit proof
          </button>
        )}
      </div>
    </dialog>
  );
}
