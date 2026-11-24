import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { AdminAPI } from "@/app/lib/api/admin";
import { ApiError, apiClient } from "@/app/lib/api/client";

const api = new AdminAPI();

const userRecord = (over: Record<string, unknown> = {}) => ({
  id: "u1",
  wallet_address: "GUSER1",
  created_at: "2025-01-01T00:00:00Z",
  kyc_status: "pending",
  active_plans_count: 2,
  tvl: "14500.50",
  status: "active",
  ...over,
});

const planRecord = (over: Record<string, unknown> = {}) => ({
  id: "p1",
  owner_address: "GOWNER1",
  token_address: "GTOKEN1",
  amount: "32000",
  yield_rate_bps: 450,
  status: "ACTIVE",
  last_ping: 1_752_487_200,
  heirs_count: 2,
  ...over,
});

const page = (data: unknown[], hasNext = false) => ({
  status: "success",
  data,
  page: 1,
  limit: 100,
  total_count: data.length,
  total_pages: 1,
  has_next: hasNext,
  has_prev: false,
});

beforeEach(() => {
  localStorage.setItem("auth_token", "admin-jwt");
  apiClient.setDefaultRetry({ maxRetries: 0 });
});

describe("AdminAPI - users", () => {
  it("sends the admin bearer token and maps records to the UI shape", async () => {
    let auth: string | null = null;
    server.use(
      http.get("*/api/admin/users", ({ request }) => {
        auth = request.headers.get("authorization");
        return HttpResponse.json(page([userRecord()]));
      })
    );

    const users = await api.listUsers();

    expect(auth).toBe("Bearer admin-jwt");
    expect(users).toEqual([
      {
        id: "u1",
        walletAddress: "GUSER1",
        registrationDate: "2025-01-01T00:00:00Z",
        kycStatus: "pending",
        activePlansCount: 2,
        tvl: 14500.5,
        status: "active",
      },
    ]);
  });

  it("follows pagination until the server reports no more pages", async () => {
    const requestedPages: string[] = [];
    server.use(
      http.get("*/api/admin/users", ({ request }) => {
        const p = new URL(request.url).searchParams.get("page")!;
        requestedPages.push(p);
        return p === "1"
          ? HttpResponse.json(page([userRecord({ id: "u1" })], true))
          : HttpResponse.json(page([userRecord({ id: "u2" })], false));
      })
    );

    const users = await api.listUsers();

    expect(requestedPages).toEqual(["1", "2"]);
    expect(users.map((u) => u.id)).toEqual(["u1", "u2"]);
  });

  it("treats a 'submitted' KYC status as pending review", async () => {
    server.use(
      http.get("*/api/admin/users", () =>
        HttpResponse.json(page([userRecord({ kyc_status: "submitted" })]))
      )
    );
    expect((await api.listUsers())[0].kycStatus).toBe("pending");
  });

  it("rejects an unknown KYC status instead of guessing", async () => {
    server.use(
      http.get("*/api/admin/users", () =>
        HttpResponse.json(page([userRecord({ kyc_status: "banana" })]))
      )
    );
    await expect(api.listUsers()).rejects.toMatchObject({ statusCode: 502 });
  });

  it("rejects a list response that is not an array", async () => {
    server.use(http.get("*/api/admin/users", () => HttpResponse.json({ status: "ok" })));
    await expect(api.listUsers()).rejects.toBeInstanceOf(ApiError);
  });

  it("calls the persisted moderation endpoints and returns the server's user", async () => {
    const calls: string[] = [];
    server.use(
      http.post("*/api/admin/kyc/approve", async ({ request }) => {
        calls.push(`kyc ${JSON.stringify(await request.json())}`);
        return HttpResponse.json({ user_id: "u1", status: "approved", created_at: "x" });
      }),
      http.post("*/api/admin/users/:id/suspend", ({ params }) => {
        calls.push(`suspend ${params.id}`);
        return HttpResponse.json(userRecord({ status: "suspended" }));
      }),
      http.post("*/api/admin/users/:id/unsuspend", ({ params }) => {
        calls.push(`unsuspend ${params.id}`);
        return HttpResponse.json(userRecord({ status: "active" }));
      }),
      http.post("*/api/admin/users/:id/terminate-plans", ({ params }) => {
        calls.push(`terminate ${params.id}`);
        return HttpResponse.json(userRecord({ active_plans_count: 0 }));
      })
    );

    expect(await api.approveUserKyc("u1")).toBe("approved");
    expect((await api.suspendUser("u1")).status).toBe("suspended");
    expect((await api.unsuspendUser("u1")).status).toBe("active");
    expect((await api.terminateUserPlans("u1")).activePlansCount).toBe(0);
    expect(calls).toEqual([
      'kyc {"user_id":"u1"}',
      "suspend u1",
      "unsuspend u1",
      "terminate u1",
    ]);
  });

  it("does not report success when the server answers 200 with an empty body", async () => {
    server.use(
      http.post("*/api/admin/users/:id/suspend", () => new HttpResponse(null, { status: 200 }))
    );
    await expect(api.suspendUser("u1")).rejects.toMatchObject({ statusCode: 502 });
  });

  it("propagates server errors with their status code", async () => {
    server.use(
      http.post("*/api/admin/users/:id/suspend", () =>
        HttpResponse.json({ error: "Forbidden" }, { status: 403 })
      )
    );
    await expect(api.suspendUser("u1")).rejects.toMatchObject({
      statusCode: 403,
      message: "Forbidden",
    });
  });
});

describe("AdminAPI - plans", () => {
  it("maps plan records to the UI shape (epoch seconds become an ISO date)", async () => {
    server.use(http.get("*/api/admin/plans", () => HttpResponse.json(page([planRecord()]))));

    const [plan] = await api.listPlans();

    expect(plan).toEqual({
      id: "p1",
      owner: "GOWNER1",
      token: "GTOKEN1",
      amount: 32000,
      yieldRateBps: 450,
      status: "ACTIVE",
      heirs: 2,
      lastPing: new Date(1_752_487_200 * 1000).toISOString(),
    });
  });

  it("terminates a plan and returns the server's updated plan", async () => {
    server.use(
      http.post("*/api/admin/plans/:id/terminate", ({ params }) =>
        HttpResponse.json(planRecord({ id: params.id, status: "DEACTIVATED" }))
      )
    );
    const plan = await api.terminatePlan("p1");
    expect(plan).toMatchObject({ id: "p1", status: "DEACTIVATED" });
  });

  it("does not report success when termination returns an empty body", async () => {
    server.use(
      http.post("*/api/admin/plans/:id/terminate", () => new HttpResponse(null, { status: 200 }))
    );
    await expect(api.terminatePlan("p1")).rejects.toMatchObject({ statusCode: 502 });
  });
});
