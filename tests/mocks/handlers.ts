/**
 * MSW request handlers — mock all API endpoints used by the app
 */
import { http, HttpResponse } from "msw";

// ─── Lending ──────────────────────────────────────────────────────────────────

export const lendingHandlers = [
  http.get("/api/lending/pool-state", () =>
    HttpResponse.json({
      total_deposits: "12500000",
      total_borrowed: "8750000",
      utilization_rate: 70,
      current_apy: 8.45,
      reserve_factor: 10,
    }),
  ),

  http.get("/api/lending/shares/:address", ({ params }) =>
    HttpResponse.json({
      shares: "5240",
      underlying_balance: "5240",
      total_earnings: "142.50",
      deposit_history: [],
    }),
  ),

  http.get("/api/lending/current-rate", () =>
    HttpResponse.json({ apy: 8.45 }),
  ),

  http.post("/api/lending/deposit", async ({ request }) => {
    const body = (await request.json()) as { amount: string };
    return HttpResponse.json({ tx_hash: "mock_tx_deposit_" + body.amount });
  }),

  http.post("/api/lending/withdraw", async ({ request }) => {
    const body = (await request.json()) as { shares: string };
    return HttpResponse.json({ tx_hash: "mock_tx_withdraw_" + body.shares });
  }),
];

// ─── Emergency ────────────────────────────────────────────────────────────────

export const emergencyHandlers = [
  http.post("/api/emergency/activate", () =>
    HttpResponse.json({ status: "activated" }),
  ),

  http.post("/api/emergency/contacts", async ({ request }) => {
    const body = (await request.json()) as Record<string, string>;
    return HttpResponse.json({
      id: "contact_1",
      name: body.name,
      email: body.email,
      wallet_address: body.wallet_address,
      added_at: new Date().toISOString(),
    });
  }),

  http.delete("/api/emergency/contacts/:id", () =>
    HttpResponse.json({ success: true }),
  ),

  http.get("/api/emergency/contacts/:planId", () =>
    HttpResponse.json([
      {
        id: "contact_1",
        name: "Alice",
        email: "alice@example.com",
        wallet_address: "GXYZ123",
        added_at: "2024-01-01T00:00:00Z",
      },
    ]),
  ),

  http.post("/api/emergency/guardians", () =>
    HttpResponse.json({ success: true }),
  ),

  http.post("/api/emergency/approve", () =>
    HttpResponse.json({ success: true }),
  ),

  http.post("/api/emergency/revoke", () =>
    HttpResponse.json({ success: true }),
  ),

  http.get("/api/emergency/audit-logs", () =>
    HttpResponse.json([
      {
        id: "log_1",
        action: "ACTIVATE",
        performed_by: "GXYZ123",
        timestamp: "2024-01-01T00:00:00Z",
      },
    ]),
  ),
];

// ─── Messages ─────────────────────────────────────────────────────────────────

export const messagesHandlers = [
  http.post("/api/messages/create", async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      id: "msg_1",
      vault_id: body.vault_id,
      title: body.title,
      content_encrypted: "encrypted_content",
      unlock_at: body.unlock_at,
      status: "DRAFT",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      beneficiary_ids: body.beneficiary_ids,
    });
  }),

  http.get("/api/messages/:id", ({ params }) =>
    HttpResponse.json({
      id: params.id,
      vault_id: "vault_1",
      title: "Test Message",
      content_encrypted: "encrypted",
      unlock_at: "2025-01-01T00:00:00Z",
      status: "DRAFT",
      created_at: "2024-01-01T00:00:00Z",
      updated_at: "2024-01-01T00:00:00Z",
      beneficiary_ids: ["ben_1"],
    }),
  ),

  http.put("/api/messages/:id", async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({ id: "msg_1", ...body });
  }),

  http.post("/api/messages/:id/finalize", () =>
    HttpResponse.json({ success: true }),
  ),

  http.delete("/api/messages/:id", () =>
    HttpResponse.json({ success: true }),
  ),

  http.get("/api/messages/vault/:vaultId", () =>
    HttpResponse.json([]),
  ),

  http.post("/api/messages/:id/unlock", () =>
    HttpResponse.json({ content: "decrypted message content" }),
  ),

  http.get("/api/messages/:id/access-audit", () =>
    HttpResponse.json([]),
  ),
];

// ─── Will Documents ───────────────────────────────────────────────────────────

export const willDocumentsHandlers = [
  http.get("/api/plans/:planId/will/documents", () =>
    HttpResponse.json({
      status: "ok",
      data: [
        {
          document_id: "doc_1",
          plan_id: "plan_1",
          template_used: "standard",
          will_hash: "abc123",
          generated_at: "2024-01-01T00:00:00Z",
          version: 1,
          filename: "will_v1.pdf",
        },
      ],
    }),
  ),

  http.get("/api/will/documents/:documentId", ({ params }) =>
    HttpResponse.json({
      status: "ok",
      data: {
        document_id: params.documentId,
        plan_id: "plan_1",
        template_used: "standard",
        will_hash: "abc123",
        generated_at: "2024-01-01T00:00:00Z",
        version: 1,
        filename: "will_v1.pdf",
      },
    }),
  ),

  http.get("/api/will/documents/:documentId/verify", () =>
    HttpResponse.json({
      status: "ok",
      data: {
        is_valid: true,
        document_id: "doc_1",
        version: 1,
        hash_match: true,
        message: "Document is valid",
      },
    }),
  ),

  http.post("/api/plans/:planId/will/generate", async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      status: "ok",
      data: {
        document_id: "doc_new",
        plan_id: "plan_1",
        template_used: "standard",
        will_hash: "newhash",
        generated_at: new Date().toISOString(),
        version: 2,
        filename: "will_v2.pdf",
      },
    });
  }),

  http.get("/api/plans/:planId/will/events", () =>
    HttpResponse.json({ status: "ok", data: [] }),
  ),

  http.get("/api/plans/:planId/will/events/stats", () =>
    HttpResponse.json({
      status: "ok",
      data: {
        plan_id: "plan_1",
        will_created_count: 1,
        will_updated_count: 0,
        will_finalized_count: 0,
        will_signed_count: 0,
        witness_signed_count: 0,
        will_verified_count: 1,
        total_events: 2,
        first_event_at: "2024-01-01T00:00:00Z",
        last_event_at: "2024-01-01T00:00:00Z",
      },
    }),
  ),
];

// ─── Combined ─────────────────────────────────────────────────────────────────

export const handlers = [
  ...lendingHandlers,
  ...emergencyHandlers,
  ...messagesHandlers,
  ...willDocumentsHandlers,
];
