/**
 * MSW request handlers — mock all API endpoints used by the app
 * Now with filtering, sorting, and search support
 */
import { http, HttpResponse } from "msw";
import { parseQueryParams, applyQueryParams } from "@/lib/api/filtering";
import {
  mockPlans,
  mockClaims,
  mockMessages,
  mockContacts,
  mockWillDocuments,
  mockAuditLogs,
} from "./data";
import { notificationService } from "@/lib/notifications";

// ─── Plans ────────────────────────────────────────────────────────────────────

export const plansHandlers = [
  // List plans with filtering, sorting, and search
  http.get("/api/plans", ({ request }) => {
    const url = new URL(request.url);
    const params = parseQueryParams(url.searchParams);
    
    const result = applyQueryParams(mockPlans, params, [
      "name",
      "status",
      "type",
      "owner_address",
    ]);

    return HttpResponse.json({
      status: "ok",
      data: result.data,
      pagination: {
        total: result.total,
        page: result.page,
        limit: result.limit,
        totalPages: result.totalPages,
      },
      filters: result.filters,
      sort: result.sort,
    });
  }),

  http.get("/api/plans/:id", ({ params }) =>
    HttpResponse.json({
      status: "ok",
      data: mockPlans.find((p) => p.id === params.id) || null,
    })
  ),
];

// ─── Claims ───────────────────────────────────────────────────────────────────

export const claimsHandlers = [
  // List claims with filtering, sorting, and search
  http.get("/api/claims", ({ request }) => {
    const url = new URL(request.url);
    const params = parseQueryParams(url.searchParams);
    
    const result = applyQueryParams(mockClaims, params, [
      "beneficiary_name",
      "status",
      "claim_type",
    ]);

    return HttpResponse.json({
      status: "ok",
      data: result.data,
      pagination: {
        total: result.total,
        page: result.page,
        limit: result.limit,
        totalPages: result.totalPages,
      },
      filters: result.filters,
      sort: result.sort,
    });
  }),

  http.post("/api/claims", async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      status: "ok",
      data: {
        id: "claim_new",
        ...body,
        submitted_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    });
  }),
];

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

  http.get("/api/emergency/contacts/:planId", ({ params, request }) => {
    const url = new URL(request.url);
    const queryParams = parseQueryParams(url.searchParams);
    
    // Filter by plan_id
    const planContacts = mockContacts.filter((c) => c.plan_id === params.planId);
    
    const result = applyQueryParams(planContacts, queryParams, [
      "name",
      "email",
      "relationship",
    ]);

    return HttpResponse.json({
      status: "ok",
      data: result.data,
      pagination: {
        total: result.total,
        page: result.page,
        limit: result.limit,
        totalPages: result.totalPages,
      },
    });
  }),

  http.post("/api/emergency/guardians", () =>
    HttpResponse.json({ success: true }),
  ),

  http.post("/api/emergency/approve", () =>
    HttpResponse.json({ success: true }),
  ),

  http.post("/api/emergency/revoke", () =>
    HttpResponse.json({ success: true }),
  ),

  http.get("/api/emergency/audit-logs", ({ request }) => {
    const url = new URL(request.url);
    const params = parseQueryParams(url.searchParams);
    
    const result = applyQueryParams(mockAuditLogs, params, [
      "action",
      "entity_type",
      "performed_by",
    ]);

    return HttpResponse.json({
      status: "ok",
      data: result.data,
      pagination: {
        total: result.total,
        page: result.page,
        limit: result.limit,
        totalPages: result.totalPages,
      },
    });
  }),
];

// ─── Messages ─────────────────────────────────────────────────────────────────

export const messagesHandlers = [
  // List messages with filtering, sorting, and search
  http.get("/api/messages", ({ request }) => {
    const url = new URL(request.url);
    const params = parseQueryParams(url.searchParams);
    
    const result = applyQueryParams(mockMessages, params, [
      "title",
      "status",
      "priority",
    ]);

    return HttpResponse.json({
      status: "ok",
      data: result.data,
      pagination: {
        total: result.total,
        page: result.page,
        limit: result.limit,
        totalPages: result.totalPages,
      },
      filters: result.filters,
      sort: result.sort,
    });
  }),

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
  http.get("/api/plans/:planId/will/documents", ({ params, request }) => {
    const url = new URL(request.url);
    const queryParams = parseQueryParams(url.searchParams);
    
    // Filter by plan_id
    const planDocs = mockWillDocuments.filter((d) => d.plan_id === params.planId);
    
    const result = applyQueryParams(planDocs, queryParams, [
      "template_used",
      "status",
      "filename",
    ]);

    return HttpResponse.json({
      status: "ok",
      data: result.data,
      pagination: {
        total: result.total,
        page: result.page,
        limit: result.limit,
        totalPages: result.totalPages,
      },
    });
  }),

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

// ─── Notifications ────────────────────────────────────────────────────────────

export const notificationsHandlers = [
  // Send notification
  http.post("/api/notifications/send", async ({ request }) => {
    const body = (await request.json()) as any;
    
    try {
      const results = await notificationService.send(body);
      return HttpResponse.json({
        status: "ok",
        data: results,
      });
    } catch (error) {
      return HttpResponse.json(
        {
          status: "error",
          message: error instanceof Error ? error.message : "Failed to send notification",
        },
        { status: 400 }
      );
    }
  }),

  // Get user notifications
  http.get("/api/notifications", ({ request }) => {
    const url = new URL(request.url);
    const user_id = url.searchParams.get("user_id");
    const type = url.searchParams.get("type") as any;
    const status = url.searchParams.get("status") as any;
    const category = url.searchParams.get("category") as any;

    if (!user_id) {
      return HttpResponse.json(
        { status: "error", message: "user_id required" },
        { status: 400 }
      );
    }

    const notifications = notificationService.getUserNotifications(user_id, {
      type,
      status,
      category,
    });

    return HttpResponse.json({
      status: "ok",
      data: notifications,
    });
  }),

  // Mark notification as read
  http.put("/api/notifications/:id/read", ({ params }) => {
    const success = notificationService.markAsRead(params.id as string);

    if (!success) {
      return HttpResponse.json(
        { status: "error", message: "Notification not found" },
        { status: 404 }
      );
    }

    return HttpResponse.json({
      status: "ok",
      data: { id: params.id, read: true },
    });
  }),

  // Get notification preferences
  http.get("/api/notifications/preferences/:userId", ({ params }) => {
    const prefs = notificationService.getPreferences(params.userId as string);

    return HttpResponse.json({
      status: "ok",
      data: prefs,
    });
  }),

  // Update notification preferences
  http.put("/api/notifications/preferences/:userId", async ({ params, request }) => {
    const body = (await request.json()) as any;
    const prefs = notificationService.updatePreferences(
      params.userId as string,
      body
    );

    return HttpResponse.json({
      status: "ok",
      data: prefs,
    });
  }),

  // Retry failed notification
  http.post("/api/notifications/:id/retry", async ({ params }) => {
    try {
      const result = await notificationService.retry(params.id as string);
      return HttpResponse.json({
        status: "ok",
        data: result,
      });
    } catch (error) {
      return HttpResponse.json(
        {
          status: "error",
          message: error instanceof Error ? error.message : "Failed to retry",
        },
        { status: 400 }
      );
    }
  }),
];

export const handlers = [
  ...plansHandlers,
  ...claimsHandlers,
  ...lendingHandlers,
  ...emergencyHandlers,
  ...messagesHandlers,
  ...willDocumentsHandlers,
  ...notificationsHandlers,
];
