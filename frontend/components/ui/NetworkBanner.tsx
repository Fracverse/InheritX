"use client";

import { useEffect, useMemo, useState } from "react";
import { AlertTriangle, Check, Copy, ExternalLink } from "lucide-react";
import { Contract, StrKey, rpc } from "@stellar/stellar-sdk";
import {
  SOROBAN_RPC_URL,
  getInheritanceContractId,
  getLendingContractId,
  stellarExpertContractUrl,
} from "@/app/lib/stellar/network";
import { formatAddress } from "@/util/address";

type ContractStatus =
  | "checking"
  | "verified"
  | "missing"
  | "invalid"
  | "not-found"
  | "unverified";

interface ContractEntry {
  label: string;
  envVar: string;
  id: string;
}

/** Local (synchronous) validation of a configured contract id. */
function validateId(id: string): ContractStatus {
  if (!id) return "missing";
  return StrKey.isValidContract(id) ? "checking" : "invalid";
}

/** Asks the configured Soroban RPC whether the contract instance exists. */
async function checkDeployed(
  id: string
): Promise<"verified" | "not-found" | "unverified"> {
  try {
    const server = new rpc.Server(SOROBAN_RPC_URL, {
      allowHttp: SOROBAN_RPC_URL.startsWith("http://"),
    });
    const result = await server.getLedgerEntries(new Contract(id).getFootprint());
    return result.entries.length > 0 ? "verified" : "not-found";
  } catch {
    // An unreachable RPC says nothing about the env vars, so don't flag a mismatch.
    return "unverified";
  }
}

function warningFor(entry: ContractEntry, status: ContractStatus): string | null {
  switch (status) {
    case "missing":
      return `${entry.envVar} is not set.`;
    case "invalid":
      return `${entry.envVar} is not a valid Soroban contract ID.`;
    case "not-found":
      return `${entry.envVar} does not match a contract deployed on this network.`;
    default:
      return null;
  }
}

function CopyButton({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), 2000);
    return () => window.clearTimeout(timer);
  }, [copied]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
    } catch {
      // Clipboard unavailable (e.g. insecure context); leave the UI unchanged.
    }
  };

  return (
    <button
      type="button"
      onClick={() => void handleCopy()}
      aria-label={`Copy ${label} contract ID`}
      className="rounded p-1 text-gray-300 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-cyan-400"
    >
      {copied ? <Check size={14} /> : <Copy size={14} />}
    </button>
  );
}

export default function NetworkBanner() {
  const entries = useMemo<ContractEntry[]>(
    () => [
      {
        label: "Inheritance",
        envVar: "NEXT_PUBLIC_INHERITANCE_CONTRACT_ID",
        id: getInheritanceContractId(),
      },
      {
        label: "Lending",
        envVar: "NEXT_PUBLIC_LENDING_CONTRACT_ID",
        id: getLendingContractId(),
      },
    ],
    []
  );

  const [statuses, setStatuses] = useState<ContractStatus[]>(() =>
    entries.map((entry) => validateId(entry.id))
  );

  useEffect(() => {
    let cancelled = false;
    entries.forEach((entry, index) => {
      if (validateId(entry.id) !== "checking") return;
      void checkDeployed(entry.id).then((result) => {
        if (cancelled) return;
        setStatuses((prev) => prev.map((s, i) => (i === index ? result : s)));
      });
    });
    return () => {
      cancelled = true;
    };
  }, [entries]);

  const warnings = entries
    .map((entry, index) => warningFor(entry, statuses[index]))
    .filter((message): message is string => message !== null);

  return (
    <section
      aria-label="Testnet contract deployment"
      className="w-full border-b border-white/10 bg-[#0F1621] px-4 py-2 text-xs text-gray-300"
    >
      <div className="flex flex-wrap items-center gap-x-6 gap-y-1">
        <span className="font-semibold text-cyan-300">Testnet contracts</span>
        {entries.map((entry) =>
          entry.id ? (
            <span key={entry.envVar} className="inline-flex items-center gap-1">
              <span className="opacity-70">{entry.label}:</span>
              <code title={entry.id} className="font-mono">
                {formatAddress(entry.id)}
              </code>
              <CopyButton value={entry.id} label={entry.label} />
              <a
                href={stellarExpertContractUrl(entry.id)}
                target="_blank"
                rel="noopener noreferrer"
                aria-label={`View ${entry.label} contract on Stellar Expert`}
                className="rounded p-1 text-gray-300 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-cyan-400"
              >
                <ExternalLink size={14} />
              </a>
            </span>
          ) : (
            <span key={entry.envVar} className="opacity-70">
              {entry.label}: not configured
            </span>
          )
        )}
      </div>
      {warnings.length > 0 && (
        <ul
          role="alert"
          className="mt-2 space-y-1 rounded-lg border border-amber-500/30 bg-amber-500/10 p-2 text-amber-300"
        >
          {warnings.map((message) => (
            <li key={message} className="flex items-center gap-2">
              <AlertTriangle size={14} className="shrink-0" />
              <span>{message}</span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
