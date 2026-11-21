import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  GeneticVerificationAPI,
  type FamilyInvitation,
} from "@/app/lib/api/geneticVerification";
import { FamilyInviteModal } from "@/components/plans/FamilyInviteModal";

const invitations: FamilyInvitation[] = [
  {
    invitationId: "invite-pending",
    toEmail: "pending@example.com",
    proposedRelationship: "sibling",
    status: "pending",
    sentAt: "2026-10-22T12:00:00Z",
  },
  {
    invitationId: "invite-verified",
    walletAddress: "GVERIFIED",
    proposedRelationship: "parent",
    status: "verified",
    sentAt: "2026-10-22T12:00:00Z",
  },
  {
    invitationId: "invite-disputed",
    toEmail: "disputed@example.com",
    proposedRelationship: "child",
    status: "dispute_flagged",
    sentAt: "2026-10-22T12:00:00Z",
  },
];

describe("FamilyInviteModal", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("shows invitation progress with pending, verified, and dispute badges", async () => {
    vi.spyOn(
      GeneticVerificationAPI.prototype,
      "getFamilyInvitations",
    ).mockResolvedValue(invitations);

    render(
      <FamilyInviteModal planId="plan-1" open onClose={vi.fn()} />,
    );

    expect(await screen.findByText("pending@example.com")).toBeInTheDocument();
    expect(screen.getByText("GVERIFIED")).toBeInTheDocument();
    expect(screen.getByText("disputed@example.com")).toBeInTheDocument();
    expect(
      screen.getByLabelText("Invitation status: Pending"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Invitation status: Verified"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Invitation status: Dispute Flagged"),
    ).toBeInTheDocument();
  });

  it("sends an email invitation with the selected relationship", async () => {
    const user = userEvent.setup();
    vi.spyOn(
      GeneticVerificationAPI.prototype,
      "getFamilyInvitations",
    ).mockResolvedValue([]);
    const sendInvitation = vi
      .spyOn(GeneticVerificationAPI.prototype, "sendFamilyInvitation")
      .mockResolvedValue({
        ...invitations[0],
        invitationId: "invite-new",
        toEmail: "kin@example.com",
      });

    render(
      <FamilyInviteModal planId="plan-1" open onClose={vi.fn()} />,
    );

    await user.type(
      screen.getByLabelText("Family member's email"),
      "kin@example.com",
    );
    await user.selectOptions(
      screen.getByLabelText("Relationship"),
      "parent",
    );
    await user.click(screen.getByRole("button", { name: "Send invitation" }));

    await waitFor(() => {
      expect(sendInvitation).toHaveBeenCalledWith(
        "plan-1",
        "kin@example.com",
        "parent",
        "email",
      );
    });
    expect(await screen.findByText("kin@example.com")).toBeInTheDocument();
  });

  it("offers a wallet-address invitation form", async () => {
    const user = userEvent.setup();
    vi.spyOn(
      GeneticVerificationAPI.prototype,
      "getFamilyInvitations",
    ).mockResolvedValue([]);

    render(
      <FamilyInviteModal planId="plan-1" open onClose={vi.fn()} />,
    );

    await user.click(screen.getByRole("button", { name: "Soroban wallet" }));

    expect(
      screen.getByLabelText("Stellar account or Soroban contract address"),
    ).toBeInTheDocument();
  });
});
