import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { CreateCommitmentStepConfigure } from "@/components/CreateCommitmentStepConfigure";

// ─── Helpers ──────────────────────────────────────────────────────────────────

const VALID: Record<string, string> = {
  title: "My Will",
  net_amount: "1000",
  currency_preference: "USDC",
  beneficiary_name: "Alice",
  bank_account_number: "123456",
  bank_name: "Zenith",
  inactivity_days: "365",
};

async function fillForm(overrides: Partial<typeof VALID> = {}) {
  const vals = { ...VALID, ...overrides };
  const user = userEvent.setup({ delay: null });
  for (const [id, value] of Object.entries(vals)) {
    const el = document.getElementById(id);
    if (!el) continue;
    if (el.tagName === "SELECT") {
      await user.selectOptions(el as HTMLSelectElement, value);
    } else {
      await user.clear(el);
      if (value) await user.type(el, value);
    }
  }
  return user;
}

/** Simulate debounce fire with fake timers and flush pending microtasks. */
async function fireDebounce() {
  await act(async () => { vi.advanceTimersByTime(500); });
}

/** Build a resolved fetch mock for a given validate response. */
function mockFetch(body: object) {
  return vi.spyOn(globalThis, "fetch").mockResolvedValue(
    new Response(JSON.stringify(body), {
      headers: { "Content-Type": "application/json" },
    })
  );
}

// ─── Tests ────────────────────────────────────────────────────────────────────

describe("CreateCommitmentStepConfigure", () => {
  const onAdvance = vi.fn();

  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    onAdvance.mockClear();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  // ── Renders ────────────────────────────────────────────────────────────────

  it("renders all required fields", () => {
    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    expect(screen.getByLabelText(/title/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/amount/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/currency/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/beneficiary name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/bank account/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/bank name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/inactivity/i)).toBeInTheDocument();
  });

  it("renders with initialValues pre-filled", () => {
    render(
      <CreateCommitmentStepConfigure
        initialValues={{ title: "Pre-filled" }}
        onAdvance={onAdvance}
      />
    );
    expect(screen.getByDisplayValue("Pre-filled")).toBeInTheDocument();
  });

  // ── Debounce ───────────────────────────────────────────────────────────────

  it("does not call validate immediately on change", async () => {
    const spy = mockFetch({ valid: true, errors: [] });
    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await userEvent.setup({ delay: null }).type(document.getElementById("title")!, "Hi");
    expect(spy).not.toHaveBeenCalled();
  });

  it("calls validate after 500 ms debounce", async () => {
    const spy = mockFetch({ valid: true, errors: [] });
    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await userEvent.setup({ delay: null }).type(document.getElementById("title")!, "Hi");
    await fireDebounce();
    await waitFor(() => expect(spy).toHaveBeenCalled());
  });

  it("resets the debounce timer on every keystroke", async () => {
    const spy = mockFetch({ valid: true, errors: [] });
    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    const input = document.getElementById("title")!;
    const user = userEvent.setup({ delay: null });

    await user.type(input, "A");
    await act(async () => { vi.advanceTimersByTime(300); });
    await user.type(input, "B");
    await act(async () => { vi.advanceTimersByTime(300); });

    // Timer reset — should not have fired yet
    expect(spy).not.toHaveBeenCalled();

    await act(async () => { vi.advanceTimersByTime(200); });
    await waitFor(() => expect(spy).toHaveBeenCalledTimes(1));
  });

  // ── Field error mapping ────────────────────────────────────────────────────

  it("shows server field errors next to the correct inputs", async () => {
    mockFetch({
      valid: false,
      errors: [
        { field: "title", message: "Title must be at least 3 characters." },
        { field: "net_amount", message: "Amount must be a positive number." },
      ],
    });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();

    await waitFor(() =>
      expect(screen.getByText("Title must be at least 3 characters.")).toBeInTheDocument()
    );
    expect(screen.getByText("Amount must be a positive number.")).toBeInTheDocument();
  });

  it("sets aria-invalid on fields with errors", async () => {
    mockFetch({
      valid: false,
      errors: [{ field: "bank_name", message: "Bank name must be at least 2 characters." }],
    });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();

    await waitFor(() =>
      expect(document.getElementById("bank_name")).toHaveAttribute("aria-invalid", "true")
    );
  });

  it("sets aria-describedby pointing to the error span", async () => {
    mockFetch({
      valid: false,
      errors: [{ field: "bank_name", message: "Bank name must be at least 2 characters." }],
    });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();

    await waitFor(() => {
      const input = document.getElementById("bank_name")!;
      expect(input).toHaveAttribute("aria-describedby", "bank_name-error");
      expect(document.getElementById("bank_name-error")).toBeInTheDocument();
    });
  });

  it("clears errors when validation returns valid", async () => {
    const spy = mockFetch({
      valid: false,
      errors: [{ field: "title", message: "Title must be at least 3 characters." }],
    });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();
    await waitFor(() =>
      expect(screen.getByText("Title must be at least 3 characters.")).toBeInTheDocument()
    );

    spy.mockResolvedValue(
      new Response(JSON.stringify({ valid: true, errors: [] }), {
        headers: { "Content-Type": "application/json" },
      })
    );
    await userEvent.setup({ delay: null }).type(document.getElementById("title")!, "My Plan");
    await fireDebounce();

    await waitFor(() =>
      expect(screen.queryByText("Title must be at least 3 characters.")).not.toBeInTheDocument()
    );
  });

  // ── Blocking advance ───────────────────────────────────────────────────────

  it("disables Continue while server errors are present", async () => {
    mockFetch({ valid: false, errors: [{ field: "title", message: "Too short." }] });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();

    await waitFor(() => expect(screen.getByText("Too short.")).toBeInTheDocument());
    expect(screen.getByRole("button", { name: /continue/i })).toBeDisabled();
  });

  it("enables Continue when validation passes", async () => {
    mockFetch({ valid: true, errors: [] });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();

    await waitFor(() =>
      expect(screen.getByRole("button", { name: /continue/i })).not.toBeDisabled()
    );
  });

  it("calls onAdvance with form values when Continue is clicked on valid form", async () => {
    mockFetch({ valid: true, errors: [] });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fillForm();
    await fireDebounce();

    await waitFor(() =>
      expect(screen.getByRole("button", { name: /continue/i })).not.toBeDisabled()
    );

    await userEvent.setup({ delay: null }).click(screen.getByRole("button", { name: /continue/i }));
    expect(onAdvance).toHaveBeenCalledWith(expect.objectContaining({ title: "My Will" }));
  });

  it("does not call onAdvance when Continue is clicked with errors", async () => {
    mockFetch({ valid: false, errors: [{ field: "title", message: "Too short." }] });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();
    await waitFor(() => expect(screen.getByText("Too short.")).toBeInTheDocument());

    const form = screen.getByRole("form", { name: /configure commitment/i });
    form.dispatchEvent(new Event("submit", { bubbles: true }));
    expect(onAdvance).not.toHaveBeenCalled();
  });

  // ── Server-down fallback ───────────────────────────────────────────────────

  it("shows a warning and allows advance when the server is unreachable", async () => {
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new TypeError("Failed to fetch"));

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();

    await waitFor(() =>
      expect(screen.getByText(/validation service unavailable/i)).toBeInTheDocument()
    );
    expect(screen.getByRole("button", { name: /continue/i })).not.toBeDisabled();
  });

  // ── Request cancellation ───────────────────────────────────────────────────

  it("cancels the in-flight request when new input arrives", async () => {
    let abortCount = 0;
    vi.spyOn(globalThis, "fetch").mockImplementation(async (_, init) => {
      init?.signal?.addEventListener("abort", () => abortCount++);
      await new Promise((_, reject) =>
        setTimeout(() => reject(new DOMException("AbortError", "AbortError")), 200)
      );
      return new Response();
    });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    const user = userEvent.setup({ delay: null });
    const input = document.getElementById("title")!;

    await user.type(input, "A");
    await fireDebounce(); // triggers first validate
    await user.type(input, "B"); // aborts first, resets timer
    await fireDebounce(); // triggers second validate

    await waitFor(() => expect(abortCount).toBeGreaterThanOrEqual(1));
  });

  it("cancels any pending request on component unmount", async () => {
    let aborted = false;
    vi.spyOn(globalThis, "fetch").mockImplementation(async (_, init) => {
      init?.signal?.addEventListener("abort", () => (aborted = true));
      await new Promise(() => {}); // never resolves
      return new Response();
    });

    const { unmount } = render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();
    await Promise.resolve(); // let fetch start

    unmount();
    await waitFor(() => expect(aborted).toBe(true));
  });

  // ── Loading state ──────────────────────────────────────────────────────────

  it("shows Validating… label while request is in-flight", async () => {
    let resolve!: (v: Response) => void;
    vi.spyOn(globalThis, "fetch").mockReturnValue(
      new Promise<Response>((r) => (resolve = r))
    );

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /validating/i })).toBeInTheDocument()
    );

    resolve(
      new Response(JSON.stringify({ valid: true, errors: [] }), {
        headers: { "Content-Type": "application/json" },
      })
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /continue/i })).toBeInTheDocument()
    );
  });

  // ── Multiple errors at once ────────────────────────────────────────────────

  it("maps multiple field errors simultaneously", async () => {
    mockFetch({
      valid: false,
      errors: [
        { field: "title", message: "Title must be at least 3 characters." },
        { field: "beneficiary_name", message: "Beneficiary name must be at least 2 characters." },
        { field: "bank_account_number", message: "Bank account number must be 6–20 digits." },
      ],
    });

    render(<CreateCommitmentStepConfigure onAdvance={onAdvance} />);
    await fireDebounce();

    await waitFor(() =>
      expect(screen.getByText("Title must be at least 3 characters.")).toBeInTheDocument()
    );
    expect(screen.getByText("Beneficiary name must be at least 2 characters.")).toBeInTheDocument();
    expect(screen.getByText("Bank account number must be 6–20 digits.")).toBeInTheDocument();
  });
});
