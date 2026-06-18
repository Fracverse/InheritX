"use client";

import { useState } from "react";
import { UserPlus, Mail } from "lucide-react";
import {
  createGeneticVerificationAPI,
  FamilyInvitation,
  RelationshipType,
} from "@/app/lib/api/geneticVerification";

interface Props {
  planId: string;
  invitations: FamilyInvitation[];
  onInviteSent: (invite: FamilyInvitation) => void;
}

const RELATIONSHIP_OPTIONS: { value: RelationshipType; label: string }[] = [
  { value: "parent", label: "Parent" },
  { value: "child", label: "Child" },
  { value: "sibling", label: "Sibling" },
  { value: "grandparent", label: "Grandparent" },
  { value: "grandchild", label: "Grandchild" },
  { value: "spouse", label: "Spouse" },
  { value: "other", label: "Other" },
];

export default function FamilyInviteComponent({
  planId,
  invitations,
  onInviteSent,
}: Props) {
  const [email, setEmail] = useState("");
  const [relationship, setRelationship] = useState<RelationshipType>("sibling");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const api = createGeneticVerificationAPI();

  const handleInvite = async () => {
    if (!planId) {
      setError("Missing plan ID. Open this page from one of your plans.");
      return;
    }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      setError("Enter a valid email address");
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const invite = await api.sendFamilyInvitation(
        planId,
        email,
        relationship,
      );
      onInviteSent(invite);
      setEmail("");
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Could not send the invite.",
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
        <h2 className="mb-1 text-xl font-semibold text-white">
          Invite a Relative
        </h2>
        <p className="mb-6 text-sm text-[#92A5A8]">
          Invite a family member to link their verification to your family
          tree.
        </p>

        <div className="flex flex-col gap-4 sm:flex-row">
          <div className="flex-1">
            <label className="mb-2 block text-sm text-[#92A5A8]">
              Email address
            </label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="relative@email.com"
              className="w-full rounded-lg border border-[#2A3338] bg-transparent px-4 py-3 text-[#FCFFFF] placeholder-[#6B7C7F] focus:border-[#33C5E0] focus:outline-none"
              disabled={loading}
            />
          </div>
          <div className="sm:w-48">
            <label className="mb-2 block text-sm text-[#92A5A8]">
              Relationship
            </label>
            <select
              value={relationship}
              onChange={(e) =>
                setRelationship(e.target.value as RelationshipType)
              }
              className="w-full rounded-lg border border-[#2A3338] bg-transparent px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] focus:outline-none"
              disabled={loading}
            >
              {RELATIONSHIP_OPTIONS.map((opt) => (
                <option key={opt.value} className="bg-[#1C252A]" value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </div>

        {error && (
          <div className="mt-4 rounded-lg border border-[#F56565]/30 bg-[#F56565]/10 p-3 text-sm text-[#F56565]">
            {error}
          </div>
        )}

        <button
          onClick={handleInvite}
          disabled={loading || !email}
          className="mt-6 flex items-center gap-2 rounded-lg bg-[#33C5E0] px-6 py-3 font-semibold text-[#0D1417] transition-colors hover:bg-[#2AB8D3] disabled:cursor-not-allowed disabled:opacity-50"
        >
          <UserPlus size={18} />
          {loading ? "Sending…" : "Send Invite"}
        </button>
      </div>

      <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
        <h2 className="mb-4 text-xl font-semibold text-white">
          Pending Invitations
        </h2>
        {invitations.length === 0 ? (
          <p className="text-sm text-[#92A5A8]">No pending invitations.</p>
        ) : (
          <ul className="space-y-3">
            {invitations.map((invite) => (
              <li
                key={invite.invitationId}
                className="flex items-center justify-between rounded-lg border border-[#1C252A] bg-[#0A0F11] px-4 py-3"
              >
                <div className="flex items-center gap-3 text-sm text-white">
                  <Mail size={16} className="text-[#33C5E0]" />
                  {invite.toEmail}
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-xs capitalize text-[#92A5A8]">
                    {invite.proposedRelationship}
                  </span>
                  <span className="rounded-full bg-[#ECC94B]/20 px-2 py-0.5 text-xs font-medium text-[#ECC94B]">
                    {invite.status}
                  </span>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}