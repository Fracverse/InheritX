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

  http.post("/api/plans/:id/trigger", ({ params }) => {
    const id = params.id as string;
    const plan = mockPlans.find(p => p.id === id);
    if (plan) {
      plan.status = "triggered";
    }
    triggerStates.set(id, {
      timestamp: new Date().toISOString(),
      freeze_status: "PENDING",
      recall_progress: 0,
      settlement_status: "PENDING",
      outstanding_loans: [
        { pool: "Soroban USDC-LEND", amount: "4,500 USDC", status: "Active" },
        { pool: "Soroban XLM-POOL", amount: "15,000 XLM", status: "Active" }
      ]
    });
    return HttpResponse.json({ status: "ok", message: "Inheritance triggered successfully" });
  }),

  http.post("/api/plans/:id/freeze-loans", ({ params }) => {
    const id = params.id as string;
    const state = triggerStates.get(id) || {
      timestamp: new Date().toISOString(),
      freeze_status: "PENDING",
      recall_progress: 0,
      settlement_status: "PENDING",
      outstanding_loans: [
        { pool: "Soroban USDC-LEND", amount: "4,500 USDC", status: "Active" },
        { pool: "Soroban XLM-POOL", amount: "15,000 XLM", status: "Active" }
      ]
    };
    state.freeze_status = "FROZEN";
    state.outstanding_loans = state.outstanding_loans.map(l => ({ ...l, status: "Frozen" }));
    triggerStates.set(id, state);
    return HttpResponse.json({ status: "ok", message: "Loans frozen successfully" });
  }),

  http.post("/api/plans/:id/recall-loans", ({ params }) => {
    const id = params.id as string;
    const state = triggerStates.get(id);
    if (state) {
      state.recall_progress = 100;
      state.outstanding_loans = state.outstanding_loans.map(l => ({ ...l, status: "Recalled" }));
      triggerStates.set(id, state);
    }
    return HttpResponse.json({ status: "ok", message: "Loans recalled successfully" });
  }),

  http.post("/api/plans/:id/liquidate-settle", ({ params }) => {
    const id = params.id as string;
    const state = triggerStates.get(id);
    if (state) {
      state.settlement_status = "SETTLED";
      const plan = mockPlans.find(p => p.id === id);
      if (plan) {
        plan.status = "claimable";
      }
      triggerStates.set(id, state);
    }
    return HttpResponse.json({ status: "ok", message: "Collateral liquidated and plan settled successfully" });
  }),

  http.get("/api/plans/:id/trigger-info", ({ params }) => {
    const id = params.id as string;
    const state = triggerStates.get(id) || {
      timestamp: null,
      freeze_status: "PENDING",
      recall_progress: 0,
      settlement_status: "PENDING",
      outstanding_loans: [
        { pool: "Soroban USDC-LEND", amount: "4,500 USDC", status: "Active" },
        { pool: "Soroban XLM-POOL", amount: "15,000 XLM", status: "Active" }
      ]
    };
    return HttpResponse.json({ status: "ok", data: state });
  }),
];

// Keep track of trigger states in memory
const triggerStates = new Map<string, {
  timestamp: string | null;
  freeze_status: "PENDING" | "PROCESSING" | "FROZEN";
  recall_progress: number;
  settlement_status: "PENDING" | "PROCESSING" | "LIQUIDATED" | "SETTLED";
  outstanding_loans: Array<{ pool: string; amount: string; status: string }>;
}>();

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

// ─── AI Optimization ──────────────────────────────────────────────────────────

const MOCK_AI_RECOMMENDATION = {
  id: "rec_001",
  planId: 1,
  recommendedAllocations: [
    {
      assetSymbol: "XLM",
      chain: "Stellar",
      currentPercentage: 45,
      recommendedPercentage: 30,
      adjustmentReason: "Reduce concentration risk",
      expectedImpact: "Lower volatility exposure",
    },
    {
      assetSymbol: "USDC",
      chain: "Stellar",
      currentPercentage: 25,
      recommendedPercentage: 35,
      adjustmentReason: "Increase stable allocation",
      expectedImpact: "Improved capital preservation",
    },
    {
      assetSymbol: "BTC",
      chain: "Bitcoin",
      currentPercentage: 20,
      recommendedPercentage: 22,
      adjustmentReason: "Long-term store of value",
      expectedImpact: "Enhanced 10-year value projection",
    },
    {
      assetSymbol: "ETH",
      chain: "Ethereum",
      currentPercentage: 10,
      recommendedPercentage: 13,
      adjustmentReason: "DeFi yield-generating assets",
      expectedImpact: "Additional yield ~4.2% APY",
    },
  ],
  confidenceScore: 87,
  expectedReturn: 14.3,
  riskScore: 42,
  reasoning: "AI-generated optimization based on historical volatility analysis.",
  generatedAt: new Date().toISOString(),
  projectedOutcomes: {
    estimatedValue1Year: 114300,
    estimatedValue5Year: 197600,
    estimatedValue10Year: 389200,
    riskMetrics: {
      volatility: 18.4,
      sharpeRatio: 1.34,
      maxDrawdown: 28.7,
      valueAtRisk: 8.2,
    },
  },
};

export const aiOptimizationHandlers = [
  http.get("/api/ai/optimize/:planId", ({ params }) =>
    HttpResponse.json({ ...MOCK_AI_RECOMMENDATION, planId: Number(params.planId) }),
  ),

  http.post("/api/ai/recommendations/:id/respond", async ({ params, request }) => {
    const body = (await request.json()) as { action: string; reason?: string };
    const status = body.action === "accept" ? "accepted" : "rejected";
    return HttpResponse.json({
      status,
      reason: body.reason,
      appliedAt: new Date().toISOString(),
    });
  }),

  http.post("/api/ai/optimize/:planId/custom", async ({ params, request }) => {
    const body = (await request.json()) as { allocations: unknown[] };
    return HttpResponse.json({
      allocations: body.allocations,
      projectedOutcomes: MOCK_AI_RECOMMENDATION.projectedOutcomes,
      expectedReturn: 12.1,
      riskScore: 38,
    });
  }),
];

// ─── Compliance ──────────────────────────────────────────────────────────────

export const complianceHandlers = [
  http.get("/api/compliance/velocity-alerts", () => {
    return HttpResponse.json({
      status: "ok",
      data: [
        {
          id: "alert_vel_1",
          type: "velocity",
          address: "GDRISK7W7YQF4LQRYR6D2AH6FZKBX6E5D3EXAMPLEADDRESS",
          amount: 15,
          asset_code: "XLM",
          event_count: 12,
          threshold: 5,
          window_minutes: 10,
          severity: "high",
          status: "open",
          reason: "High transaction velocity: 12 transfers in 10 minutes",
          created_at: new Date().toISOString(),
        },
      ],
    });
  }),

  http.get("/api/compliance/volume-alerts", () => {
    return HttpResponse.json({
      status: "ok",
      data: [
        {
          id: "alert_vol_1",
          type: "volume",
          address: "GDRISK7W7YQF4LQRYR6D2AH6FZKBX6E5D3EXAMPLEADDRESS",
          amount: 150000,
          asset_code: "USDC",
          threshold: 100000,
          severity: "critical",
          status: "open",
          reason: "Large transfer volume: 150,000 USDC exceeds threshold of 100,000",
          created_at: new Date().toISOString(),
        },
      ],
    });
  }),

  http.get("/api/compliance/risk-score/:address", ({ params }) => {
    const address = params.address as string;
    const isHighRisk = address.includes("RISK");
    return HttpResponse.json({
      status: "ok",
      data: {
        address,
        score: isHighRisk ? 85 : 15,
        level: isHighRisk ? "critical" : "low",
        factors: isHighRisk
          ? [
              {
                label: "Sanctions Association",
                impact: "negative",
                score: 90,
                description: "Direct interaction with flagged mixer smart contract.",
              },
            ]
          : [],
        last_evaluated_at: new Date().toISOString(),
      },
    });
  }),

  http.post("/api/compliance/risk-override", async ({ request }) => {
    const body = (await request.json()) as { address: string; score: number; level: string; justification: string };
    return HttpResponse.json({
      status: "ok",
      data: {
        id: "override_1",
        address: body.address,
        score: body.score,
        level: body.level,
        justification: body.justification,
        admin_id: "admin_123",
        created_at: new Date().toISOString(),
      },
    });
  }),

  http.get("/api/compliance/sanctions-check/:address", ({ params }) => {
    const address = params.address as string;
    const isHighRisk = address.includes("RISK");
    return HttpResponse.json({
      status: "ok",
      data: {
        address,
        is_flagged: isHighRisk,
        status: isHighRisk ? "flagged" : "clear",
        lists: isHighRisk ? ["OFAC SDN List", "EU Consolidated List"] : [],
        match_score: isHighRisk ? 95 : 0,
        checked_at: new Date().toISOString(),
        recommendation: isHighRisk
          ? "Reject transaction and freeze associated assets immediately."
          : "No action required.",
      },
    });
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
  ...aiOptimizationHandlers,
  ...complianceHandlers,
];

