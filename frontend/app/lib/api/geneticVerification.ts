/**
 * Genetic Verification API Client
 *
 * Types below mirror the data model defined in
 * contracts/genetic-verification/src/lib.rs as closely as Rust enums/structs
 * translate to TypeScript. That contract currently only defines types and
 * validation helpers — there is no `#[contractimpl]` block, so there are no
 * callable on-chain entrypoints yet.
 *
 * There is also no backend route for genetic verification yet (unlike will
 * documents, which have `/api/will/documents/...`). Every network method
 * below is a thin, clearly-marked stub against a route shape that mirrors
 * the existing `verification.ts` client. Swap the body for the real
 * endpoint once the backend ships it, and add MSW handlers under
 * `frontend/tests/mocks/` (matching the existing pattern) to test against
 * in the meantime.
 */

// ─── Enums (mirrors contract enums) ────────────────────────────────────────

export type DNAVerificationStatus =
  | "pending"
  | "verified"
  | "rejected"
  | "partial_match"
  | "requires_retest";

export type RelationshipType =
  | "parent"
  | "child"
  | "sibling"
  | "grandparent"
  | "grandchild"
  | "spouse"
  | "other";

/**
 * Mirrors the contract's `GeneticCondition` enum. Exact wire-format keys are
 * a guess (snake_case `kind` discriminant, matching this codebase's existing
 * JSON conventions e.g. `is_active_version`) and should be confirmed against
 * the backend's serde output once that endpoint exists.
 */
export type GeneticCondition =
  | { kind: "hereditary_disease"; name: string }
  | { kind: "life_expectancy_marker" }
  | { kind: "carrier_status"; name: string }
  | { kind: "health_risk_factor"; value: number }
  | { kind: "age_related_condition"; age: number };

// ─── Structs (mirrors contract structs) ────────────────────────────────────

export interface LineageRecord {
  personId: number;
  dnaHash: string;
  parentIds: number[];
  childrenIds: number[];
  relationshipDegree: number;
  verificationStatus: DNAVerificationStatus;
}

export interface VerifiedRelationship {
  person1Id: number;
  person2Id: number;
  relationshipType: RelationshipType;
  /** 0-100 */
  confidenceScore: number;
  verifiedBy: string;
  /** Unix seconds (ledger timestamp) */
  verificationDate: number;
}

/** An already-detected potential relative awaiting confirmation. Distinct
 * from a `FamilyInvitation` below — this comes from DNA comparison, not an
 * email invite. Not yet surfaced in the UI; needs backend matching logic
 * first. */
export interface PendingRelative {
  personId: number;
  dnaHash: string;
  proposedRelationship: RelationshipType;
}

export interface FamilyTree {
  treeId: number;
  rootPerson: number;
  allMembers: LineageRecord[];
  verifiedRelationships: VerifiedRelationship[];
  pendingDiscoveries: PendingRelative[];
}

export interface GeneticInheritance {
  dnaHash: string;
  verifiedLineage: boolean;
  geneticTriggers: GeneticCondition[];
  familyTreeId: number;
  /** Unix seconds (ledger timestamp) */
  verificationTimestamp: number;
  verifyingAuthority: string;
}

// ─── App-level types (not part of the on-chain contract) ──────────────────

/**
 * Privacy preferences are an off-chain/backend concern — the contract has
 * no equivalent type. Defined here for the frontend only.
 */
export interface PrivacySettings {
  shareWithFamily: boolean;
  allowHealthAnalysis: boolean;
  visibleToRelatives: boolean;
  /** -1 means "indefinite" */
  dataRetentionDays: number;
}

export const DEFAULT_PRIVACY_SETTINGS: PrivacySettings = {
  shareWithFamily: false,
  allowHealthAnalysis: false,
  visibleToRelatives: false,
  dataRetentionDays: 365,
};

export interface GeneticUploadResult {
  dnaHash: string;
  status: DNAVerificationStatus;
}

/**
 * Email-based family invitation. This is a separate concept from
 * `PendingRelative` above: an invite can be sent to someone who hasn't
 * submitted DNA at all yet, so it can't carry a `dnaHash` or `personId`.
 */
export type InvitationStatus = "pending" | "accepted" | "expired" | "declined";

export interface FamilyInvitation {
  invitationId: string;
  toEmail: string;
  proposedRelationship: RelationshipType;
  status: InvitationStatus;
  /** ISO timestamp */
  sentAt: string;
}

// ─── API Client ─────────────────────────────────────────────────────────────

export class GeneticVerificationAPI {
  private baseUrl: string;

  constructor(baseUrl: string = "") {
    this.baseUrl = baseUrl;
  }

  /**
   * Compute a SHA-256 hash of a file entirely client-side, mirroring
   * DocumentVerificationAPI.computeFileHash. Raw genetic file bytes are
   * never sent anywhere by this method — only the resulting hash is.
   */
  async computeFileHash(file: File): Promise<string> {
    const arrayBuffer = await file.arrayBuffer();
    const hashBuffer = await crypto.subtle.digest("SHA-256", arrayBuffer);
    return Array.from(new Uint8Array(hashBuffer))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  }

  /**
   * TODO(backend): `/api/genetic/:planId/upload` does not exist yet.
   * Posts the computed hash only.
   */
  async submitGeneticHash(
    planId: string,
    dnaHash: string,
  ): Promise<GeneticUploadResult> {
    const response = await fetch(
      `${this.baseUrl}/api/genetic/${planId}/upload`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ dna_hash: dnaHash }),
      },
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error || `Genetic upload failed with status ${response.status}`,
      );
    }

    const data = await response.json();
    return data.data;
  }

  /** TODO(backend): `/api/genetic/:planId` does not exist yet. */
  async getVerificationStatus(planId: string): Promise<GeneticInheritance> {
    const response = await fetch(`${this.baseUrl}/api/genetic/${planId}`);

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error ||
          `Fetching verification status failed with status ${response.status}`,
      );
    }

    const data = await response.json();
    return data.data;
  }

  /** TODO(backend): `/api/genetic/:planId/privacy` does not exist yet. */
  async updatePrivacySettings(
    planId: string,
    settings: PrivacySettings,
  ): Promise<PrivacySettings> {
    const response = await fetch(
      `${this.baseUrl}/api/genetic/${planId}/privacy`,
      {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(settings),
      },
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error ||
          `Updating privacy settings failed with status ${response.status}`,
      );
    }

    const data = await response.json();
    return data.data;
  }

  /** TODO(backend): `/api/genetic/:planId/family/invite` does not exist yet. */
  async sendFamilyInvitation(
    planId: string,
    email: string,
    proposedRelationship: RelationshipType,
  ): Promise<FamilyInvitation> {
    const response = await fetch(
      `${this.baseUrl}/api/genetic/${planId}/family/invite`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          email,
          proposed_relationship: proposedRelationship,
        }),
      },
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error || `Sending invite failed with status ${response.status}`,
      );
    }

    const data = await response.json();
    return data.data;
  }
}

export function createGeneticVerificationAPI(): GeneticVerificationAPI {
  return new GeneticVerificationAPI("");
}

export default createGeneticVerificationAPI;