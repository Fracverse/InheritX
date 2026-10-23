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

import type { RetryConfig } from "./client";

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
  /** Opt in to matching this plan's DNA hash against relatives who have opted in. */
  allowFamilyMatching: boolean;
  /** Compare genetic data via zero-knowledge hashes instead of raw data. */
  zeroKnowledgeHashing: boolean;
  /** Days until stored genetic data expires. -1 means "indefinite". */
  dataExpiryDays: number;
}

export const DEFAULT_PRIVACY_SETTINGS: PrivacySettings = {
  shareWithFamily: false,
  allowHealthAnalysis: false,
  visibleToRelatives: false,
  allowFamilyMatching: false,
  // Privacy-preserving default: hashing is on unless the user opts out.
  zeroKnowledgeHashing: true,
  dataExpiryDays: 365,
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
export type InvitationStatus =
  | "pending"
  | "accepted"
  | "expired"
  | "declined"
  | "verified"
  | "dispute_flagged";

export type FamilyInvitationRecipientType = "email" | "wallet";

export interface FamilyInvitation {
  invitationId: string;
  toEmail?: string;
  walletAddress?: string;
  recipientType?: FamilyInvitationRecipientType;
  proposedRelationship: RelationshipType;
  status: InvitationStatus;
  /** ISO timestamp */
  sentAt: string;
}

// ─── API Client ─────────────────────────────────────────────────────────────

/**
 * Retry policy for privacy-settings writes: transient network failures and the
 * listed statuses are retried with exponential backoff and full jitter
 * (mirrors `ApiClient`'s default policy in ./client.ts).
 */
const PRIVACY_RETRY: RetryConfig = {
  maxRetries: 3,
  baseDelayMs: 1000,
  maxDelayMs: 10_000,
  retryOnStatuses: [408, 429, 500, 502, 503, 504],
};

export class GeneticVerificationAPI {
  private baseUrl: string;

  constructor(baseUrl: string = "") {
    this.baseUrl = baseUrl;
  }

  /** Exponential backoff with full jitter for a given retry attempt. */
  private computeRetryDelay(attempt: number): number {
    const exponentialDelay = PRIVACY_RETRY.baseDelayMs * Math.pow(2, attempt);
    const cappedDelay = Math.min(exponentialDelay, PRIVACY_RETRY.maxDelayMs);
    const randomBytes = new Uint32Array(1);
    globalThis.crypto?.getRandomValues(randomBytes);
    return ((randomBytes[0] ?? 0) / 0xffffffff) * cappedDelay;
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
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

  /**
   * Persist a plan's genetic privacy preferences.
   *
   * Transient failures (network errors and the statuses listed in
   * `PRIVACY_RETRY.retryOnStatuses`) are retried with exponential backoff and
   * full jitter; non-retryable responses (e.g. 400/403) fail immediately.
   *
   * TODO(backend): `/api/genetic/:planId/privacy` does not exist yet.
   */
  async updateGeneticPrivacySettings(
    planId: string,
    settings: PrivacySettings,
  ): Promise<PrivacySettings> {
    const maxAttempts = PRIVACY_RETRY.maxRetries + 1;

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      if (attempt > 0) {
        await this.delay(this.computeRetryDelay(attempt - 1));
      }

      let response: Response;
      try {
        response = await fetch(
          `${this.baseUrl}/api/genetic/${planId}/privacy`,
          {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(settings),
          },
        );
      } catch (err) {
        // Network-level failure — retry until attempts are exhausted.
        if (attempt < maxAttempts - 1) continue;
        throw err;
      }

      if (response.ok) {
        const data = await response.json();
        return data.data;
      }

      if (
        PRIVACY_RETRY.retryOnStatuses.includes(response.status) &&
        attempt < maxAttempts - 1
      ) {
        continue;
      }

      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error ||
          `Updating privacy settings failed with status ${response.status}`,
      );
    }

    throw new Error("Updating privacy settings failed after retries");
  }

  /**
   * TODO(backend): `/api/genetic/:planId/family/invite` does not exist yet.
   * Email and wallet invitations share the endpoint; wallets are addressed by
   * their Stellar account or Soroban contract address.
   */
  async sendFamilyInvitation(
    planId: string,
    recipient: string,
    proposedRelationship: RelationshipType,
    recipientType: FamilyInvitationRecipientType = "email",
  ): Promise<FamilyInvitation> {
    const response = await fetch(
      `${this.baseUrl}/api/genetic/${planId}/family/invite`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          ...(recipientType === "email"
            ? { email: recipient }
            : { wallet_address: recipient }),
          recipient_type: recipientType,
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

  /** TODO(backend): `/api/genetic/:planId/family/invitations` does not exist yet. */
  async getFamilyInvitations(planId: string): Promise<FamilyInvitation[]> {
    const response = await fetch(
      `${this.baseUrl}/api/genetic/${planId}/family/invitations`,
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(
        error.error ||
          `Fetching family invitations failed with status ${response.status}`,
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