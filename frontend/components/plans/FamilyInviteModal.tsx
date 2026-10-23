"use client";

import { useEffect, useMemo, useState, type FormEvent } from "react";
import { StrKey } from "@stellar/stellar-sdk";
import {
  AlertCircle,
  CheckCircle,
  Clock3,
  Loader2,
  Mail,
  Wallet,
  X,
} from "lucide-react";
import {
  createGeneticVerificationAPI,
  type FamilyInvitation,
  type FamilyInvitationRecipientType,
  type RelationshipType,
} from "@/app/lib/api/geneticVerification";

interface FamilyInviteModalProps {
  planId: string;
  open: boolean;
  onClose: () => void;
}

type InviteMethod = FamilyInvitationRecipientType;

const RELATIONSHIPS: RelationshipType[] = [
  "parent",
  "child",
  "sibling",
  "grandparent",
  "grandchild",
  "spouse",
  "other",
];

function InvitationStatusBadge({ status }: { status: string }) {
  const normalized = status.toLowerCase();
  const isVerified = normalized === "verified";
  const isDisputeFlagged = normalized === "dispute_flagged";
  const label = isVerified
    ? "Verified"
    : isDisputeFlagged
      ? "Dispute Flagged"
      : "Pending";
  const className = isVerified
    ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-400"
    : isDisputeFlagged
      ? "border-red-500/30 bg-red-500/10 text-red-400"
      : "border-amber-500/30 bg-amber-500/10 text-amber-400";
  const Icon = isVerified ? CheckCircle : isDisputeFlagged ? AlertCircle : Clock3;

  return (
    <span
      className={`inline-flex shrink-0 items-center gap-1.5 rounded-full border px-2.5 py-1 text-[10px] font-medium ${className}`}
      aria-label={`Invitation status: ${label}`}
    >
      <Icon size={12} aria-hidden="true" />
      {label}
    </span>
  );
}

export function FamilyInviteModal({
  planId,
  open,
  onClose,
}: FamilyInviteModalProps) {
  const api = useMemo(() => createGeneticVerificationAPI(), []);
  const [method, setMethod] = useState<InviteMethod>("email");
  const [email, setEmail] = useState("");
  const [walletAddress, setWalletAddress] = useState("");
  const [relationship, setRelationship] = useState<RelationshipType>("sibling");
  const [invitations, setInvitations] = useState<FamilyInvitation[]>([]);
  const [loading, setLoading] = useState(false);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!open) return;

    let cancelled = false;
    const refreshInvitations = async () => {
      setLoading(true);
      try {
        const result = await api.getFamilyInvitations(planId);
        if (cancelled) return;
        setInvitations(result);
        setError("");
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof Error
              ? err.message
              : "Unable to load family invitation statuses.",
          );
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    void refreshInvitations();
    const intervalId = window.setInterval(() => {
      void refreshInvitations();
    }, 10_000);

    return () => {
      cancelled = true;
      window.clearInterval(intervalId);
    };
  }, [api, open, planId]);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const recipient = method === "email" ? email.trim() : walletAddress.trim();

    if (
      method === "wallet" &&
      !(
        StrKey.isValidEd25519PublicKey(recipient) ||
        StrKey.isValidContract(recipient)
      )
    ) {
      setError("Enter a valid Stellar account or Soroban contract address.");
      return;
    }

    setSending(true);
    setError("");
    try {
      const invitation = await api.sendFamilyInvitation(
        planId,
        recipient,
        relationship,
        method,
      );
      setInvitations((current) => [
        invitation,
        ...current.filter(
          (item) => item.invitationId !== invitation.invitationId,
        ),
      ]);
      setEmail("");
      setWalletAddress("");
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Unable to send the invitation.",
      );
    } finally {
      setSending(false);
    }
  };

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-end justify-center md:items-center"
      role="dialog"
      aria-modal="true"
      aria-labelledby="family-invite-title"
    >
      <button
        type="button"
        className="absolute inset-0 bg-black/70 backdrop-blur-sm"
        onClick={onClose}
        aria-label="Close family invitations"
      />
      <section className="relative z-10 max-h-[90dvh] w-full max-w-lg overflow-y-auto rounded-t-2xl border border-[#2A3338] bg-[#161E22] shadow-2xl md:mx-4 md:rounded-2xl">
        <header className="flex items-center justify-between border-b border-[#2A3338] px-5 py-4">
          <div>
            <h2 id="family-invite-title" className="text-base font-semibold text-white">
              Invite family
            </h2>
            <p className="mt-1 text-xs text-[#92A5A8]">
              Invite kin to participate in genetic verification.
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg p-2 text-[#92A5A8] hover:bg-white/5 hover:text-white"
            aria-label="Close"
          >
            <X size={18} />
          </button>
        </header>

        <div className="space-y-5 p-5">
          <div
            className="grid grid-cols-2 gap-2 rounded-xl bg-black/20 p-1"
            role="group"
            aria-label="Invitation method"
          >
            {(["email", "wallet"] as const).map((option) => {
              const Icon = option === "email" ? Mail : Wallet;
              return (
                <button
                  key={option}
                  type="button"
                  aria-pressed={method === option}
                  onClick={() => {
                    setMethod(option);
                    setError("");
                  }}
                  className={`inline-flex items-center justify-center gap-2 rounded-lg px-3 py-2.5 text-xs font-medium transition-colors ${
                    method === option
                      ? "bg-[#33C5E0]/15 text-[#33C5E0]"
                      : "text-[#92A5A8] hover:text-white"
                  }`}
                >
                  <Icon size={14} />
                  {option === "email" ? "Email" : "Soroban wallet"}
                </button>
              );
            })}
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            {method === "email" ? (
              <div>
                <label
                  htmlFor="family-invite-email"
                  className="mb-1.5 block text-xs font-medium text-slate-300"
                >
                  Family member&apos;s email
                </label>
                <input
                  id="family-invite-email"
                  type="email"
                  autoComplete="email"
                  required
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  placeholder="name@example.com"
                  className="w-full rounded-xl border border-[#2A3338] bg-black/20 px-3 py-2.5 text-sm text-white outline-none placeholder:text-slate-600 focus:border-[#33C5E0]/60"
                />
              </div>
            ) : (
              <div>
                <label
                  htmlFor="family-invite-wallet"
                  className="mb-1.5 block text-xs font-medium text-slate-300"
                >
                  Stellar account or Soroban contract address
                </label>
                <input
                  id="family-invite-wallet"
                  type="text"
                  required
                  value={walletAddress}
                  onChange={(event) => setWalletAddress(event.target.value)}
                  placeholder="G… or C…"
                  autoCapitalize="none"
                  spellCheck={false}
                  className="w-full rounded-xl border border-[#2A3338] bg-black/20 px-3 py-2.5 font-mono text-sm text-white outline-none placeholder:text-slate-600 focus:border-[#33C5E0]/60"
                />
              </div>
            )}

            <div>
              <label
                htmlFor="family-invite-relationship"
                className="mb-1.5 block text-xs font-medium text-slate-300"
              >
                Relationship
              </label>
              <select
                id="family-invite-relationship"
                value={relationship}
                onChange={(event) =>
                  setRelationship(event.target.value as RelationshipType)
                }
                className="w-full rounded-xl border border-[#2A3338] bg-[#0A0F11] px-3 py-2.5 text-sm capitalize text-white outline-none focus:border-[#33C5E0]/60"
              >
                {RELATIONSHIPS.map((item) => (
                  <option key={item} value={item} className="capitalize">
                    {item}
                  </option>
                ))}
              </select>
            </div>

            {error && (
              <p
                role="alert"
                className="flex items-start gap-2 rounded-xl border border-red-500/20 bg-red-500/5 p-3 text-xs text-red-300"
              >
                <AlertCircle size={15} className="mt-0.5 shrink-0" />
                {error}
              </p>
            )}

            <button
              type="submit"
              disabled={sending}
              className="inline-flex w-full items-center justify-center gap-2 rounded-xl bg-[#33C5E0] px-4 py-2.5 text-sm font-semibold text-[#0A0F11] transition-colors hover:bg-cyan-300 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {sending ? (
                <Loader2 size={16} className="animate-spin" />
              ) : method === "email" ? (
                <Mail size={16} />
              ) : (
                <Wallet size={16} />
              )}
              {sending ? "Sending invitation…" : "Send invitation"}
            </button>
          </form>

          <section aria-labelledby="family-invite-progress-title">
            <div className="mb-3 flex items-center justify-between gap-3">
              <h3
                id="family-invite-progress-title"
                className="text-xs font-semibold text-white"
              >
                Signature progress
              </h3>
              <span className="text-[10px] text-[#92A5A8]">
                Updates every 10 seconds
              </span>
            </div>
            {loading && invitations.length === 0 ? (
              <p className="flex items-center gap-2 py-3 text-xs text-[#92A5A8]">
                <Loader2 size={13} className="animate-spin" />
                Loading invitation statuses…
              </p>
            ) : invitations.length === 0 ? (
              <p className="rounded-xl border border-[#2A3338] bg-black/10 px-3 py-4 text-center text-xs text-[#92A5A8]">
                No family invitations yet.
              </p>
            ) : (
              <ul className="space-y-2">
                {invitations.map((invitation) => {
                  const target =
                    invitation.toEmail ??
                    invitation.walletAddress ??
                    "Family member";
                  return (
                    <li
                      key={invitation.invitationId}
                      className="flex items-center justify-between gap-3 rounded-xl border border-[#2A3338] bg-black/10 px-3 py-2.5"
                    >
                      <div className="min-w-0">
                        <p className="truncate text-xs text-slate-200">
                          {target}
                        </p>
                        <p className="mt-1 text-[10px] capitalize text-[#92A5A8]">
                          {invitation.proposedRelationship}
                        </p>
                      </div>
                      <InvitationStatusBadge status={invitation.status} />
                    </li>
                  );
                })}
              </ul>
            )}
          </section>
        </div>
      </section>
    </div>
  );
}
