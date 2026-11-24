import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { apiClient } from "@/app/lib/api/client";
import AdminUsersPage from "@/app/admin/users/page";

type UserRecord = {
  id: string;
  wallet_address: string;
  created_at: string;
  kyc_status: string;
  active_plans_count: number;
  tvl: number;
  status: string;
};

const baseUsers = (): UserRecord[] => [
  {
    id: "u1",
    wallet_address: "GUSER1",
    created_at: "2025-01-01T00:00:00Z",
    kyc_status: "pending",
    active_plans_count: 2,
    tvl: 14500,
    status: "active",
  },
  {
    id: "u2",
    wallet_address: "GUSER2",
    created_at: "2025-02-01T00:00:00Z",
    kyc_status: "approved",
    active_plans_count: 0,
    tvl: 0,
    status: "suspended",
  },
];

const listResponse = (data: UserRecord[]) => ({
  status: "success",
  data,
  page: 1,
  limit: 100,
  total_count: data.length,
  total_pages: 1,
  has_next: false,
  has_prev: false,
});

/** Registers the users list handler. */
function serveUsers(users: UserRecord[] = baseUsers()) {
  server.use(http.get("*/api/admin/users", () => HttpResponse.json(listResponse(users))));
}

const rowFor = (wallet: string) => screen.getByText(wallet).closest("tr")!;

beforeEach(() => {
  localStorage.setItem("auth_token", "admin-jwt");
  apiClient.setDefaultRetry({ maxRetries: 0 });
});

describe("AdminUsersPage - initial loading", () => {
  it("shows a loading state, then the users returned by the server", async () => {
    let release!: () => void;
    const gate = new Promise<void>((r) => (release = r));
    server.use(
      http.get("*/api/admin/users", async () => {
        await gate;
        return HttpResponse.json(listResponse(baseUsers()));
      })
    );

    render(<AdminUsersPage />);
    expect(screen.getByRole("status")).toHaveTextContent("Loading users");

    release();
    expect(await screen.findByText("GUSER1")).toBeInTheDocument();
    expect(screen.getByText("GUSER2")).toBeInTheDocument();
    expect(screen.getByText(/Showing 2 of 2 users/)).toBeInTheDocument();
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("requests users with the admin bearer token and no longer shows mock records", async () => {
    let auth: string | null = null;
    server.use(
      http.get("*/api/admin/users", ({ request }) => {
        auth = request.headers.get("authorization");
        return HttpResponse.json(listResponse(baseUsers()));
      })
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");

    expect(auth).toBe("Bearer admin-jwt");
    expect(screen.queryByText("GCKF4X...3X9P")).not.toBeInTheDocument(); // old mock wallet
  });

  it("shows an empty state, not an error, when the server has no users", async () => {
    serveUsers([]);
    render(<AdminUsersPage />);

    expect(await screen.findByText(/No users found/)).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});

describe("AdminUsersPage - persisted actions", () => {
  it("approves KYC via the API and shows the status the server returned", async () => {
    const user = userEvent.setup();
    let body: unknown;
    serveUsers();
    server.use(
      http.post("*/api/admin/kyc/approve", async ({ request }) => {
        body = await request.json();
        return HttpResponse.json({ user_id: "u1", status: "approved", created_at: "x" });
      })
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    expect(within(rowFor("GUSER1")).getByText("Pending")).toBeInTheDocument();

    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Approve KYC" }));

    await waitFor(() =>
      expect(within(rowFor("GUSER1")).getByText("Approved")).toBeInTheDocument()
    );
    expect(body).toEqual({ user_id: "u1" });
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("follows the server's response, not an assumption: stays pending if the server says so", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(
      http.post("*/api/admin/kyc/approve", () =>
        HttpResponse.json({ user_id: "u1", status: "pending", created_at: "x" })
      )
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Approve KYC" }));

    await waitFor(() =>
      expect(within(rowFor("GUSER1")).getByRole("button", { name: "Approve KYC" })).toBeEnabled()
    );
    expect(within(rowFor("GUSER1")).getByText("Pending")).toBeInTheDocument();
  });

  it("suspends an active user via the API", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(
      http.post("*/api/admin/users/u1/suspend", () =>
        HttpResponse.json({ ...baseUsers()[0], status: "suspended" })
      )
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Suspend" }));

    expect(
      await within(rowFor("GUSER1")).findByRole("button", { name: "Unsuspend" })
    ).toBeInTheDocument();
    expect(within(rowFor("GUSER1")).getByText("Suspended")).toBeInTheDocument();
  });

  it("unsuspends a suspended user via the API", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(
      http.post("*/api/admin/users/u2/unsuspend", () =>
        HttpResponse.json({ ...baseUsers()[1], status: "active" })
      )
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER2");
    await user.click(within(rowFor("GUSER2")).getByRole("button", { name: "Unsuspend" }));

    expect(
      await within(rowFor("GUSER2")).findByRole("button", { name: "Suspend" })
    ).toBeInTheDocument();
  });

  it("terminates a user's plans via the API", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(
      http.post("*/api/admin/users/u1/terminate-plans", () =>
        HttpResponse.json({ ...baseUsers()[0], active_plans_count: 0 })
      )
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Terminate Plan" }));

    await waitFor(() =>
      expect(
        within(rowFor("GUSER1")).queryByRole("button", { name: "Terminate Plan" })
      ).not.toBeInTheDocument()
    );
  });

  it("survives a refresh: remounting loads the state the server persisted", async () => {
    const user = userEvent.setup();
    const db = baseUsers(); // stands in for the server's database
    server.use(
      http.get("*/api/admin/users", () => HttpResponse.json(listResponse(db))),
      http.post("*/api/admin/users/:id/suspend", ({ params }) => {
        const record = db.find((u) => u.id === params.id)!;
        record.status = "suspended";
        return HttpResponse.json(record);
      })
    );

    const first = render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Suspend" }));
    await within(rowFor("GUSER1")).findByRole("button", { name: "Unsuspend" });
    first.unmount();

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    expect(within(rowFor("GUSER1")).getByRole("button", { name: "Unsuspend" })).toBeInTheDocument();
  });
});

describe("AdminUsersPage - API errors", () => {
  it("shows an actionable error and leaves the list unchanged when loading fails (401)", async () => {
    server.use(
      http.get("*/api/admin/users", () =>
        HttpResponse.json({ error: "Unauthorized" }, { status: 401 })
      )
    );

    render(<AdminUsersPage />);

    expect(await screen.findByRole("alert")).toHaveTextContent(/Loading users failed.*Sign in again/);
    // No table and no fabricated/mock rows behind the error.
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
    expect(screen.queryByText(/No users found/)).not.toBeInTheDocument();
  });

  it("recovers when the admin retries a failed load", async () => {
    const user = userEvent.setup();
    serveUsers();
    // Registered last, so it takes priority for the first request only.
    server.use(
      http.get(
        "*/api/admin/users",
        () => HttpResponse.json({ error: "Service unavailable" }, { status: 503 }),
        { once: true }
      )
    );

    render(<AdminUsersPage />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Service unavailable");

    await user.click(screen.getByRole("button", { name: "Retry" }));

    expect(await screen.findByText("GUSER1")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("keeps the row unchanged and explains a permission failure (403)", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(
      http.post("*/api/admin/kyc/approve", () =>
        HttpResponse.json({ error: "Forbidden" }, { status: 403 })
      )
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Approve KYC" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Approving KYC for GUSER1 failed: your account is not permitted to do this."
    );
    expect(within(rowFor("GUSER1")).getByText("Pending")).toBeInTheDocument();
    expect(within(rowFor("GUSER1")).getByRole("button", { name: "Approve KYC" })).toBeEnabled();
  });

  it("shows the server's message on a 500 and does not flip the suspension locally", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(
      http.post("*/api/admin/users/u1/suspend", () =>
        HttpResponse.json({ error: "Database unavailable" }, { status: 500 })
      )
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Suspend" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Suspending GUSER1 failed: Database unavailable"
    );
    expect(within(rowFor("GUSER1")).getByText("Active")).toBeInTheDocument();
    expect(within(rowFor("GUSER1")).getByRole("button", { name: "Suspend" })).toBeInTheDocument();
  });

  it("reports a network failure without changing the row", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(http.post("*/api/admin/users/u1/terminate-plans", () => HttpResponse.error()));

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Terminate Plan" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(/could not reach the server/);
    expect(within(rowFor("GUSER1")).getByRole("button", { name: "Terminate Plan" })).toBeEnabled();
  });

  it("treats an empty 200 response as a failure, not a success", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(
      http.post("*/api/admin/users/u1/suspend", () => new HttpResponse(null, { status: 200 }))
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Suspend" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(/unexpected response/);
    expect(within(rowFor("GUSER1")).getByText("Active")).toBeInTheDocument();
  });

  it("lets the admin dismiss an action error", async () => {
    const user = userEvent.setup();
    serveUsers();
    server.use(
      http.post("*/api/admin/users/u1/suspend", () =>
        HttpResponse.json({ error: "nope" }, { status: 409 })
      )
    );

    render(<AdminUsersPage />);
    await screen.findByText("GUSER1");
    await user.click(within(rowFor("GUSER1")).getByRole("button", { name: "Suspend" }));
    await screen.findByRole("alert");

    await user.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
