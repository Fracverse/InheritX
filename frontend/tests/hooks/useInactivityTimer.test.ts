import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useInactivityTimer } from "@/app/hooks/useInactivityTimer";
import { plansAPI } from "@/app/lib/api/plans";

vi.mock("@/app/lib/api/plans");

describe("useInactivityTimer", () => {
  const mockPlanId = "plan-123";

  beforeEach(() => {
    vi.useFakeTimers({
      shouldAdvanceTime: true,
    });

    vi.setSystemTime(1_000_000_000_000);

    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  it("initializes with loading state and fetches inactivity status", async () => {
    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue({
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    });

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
  });

  it("calculates time remaining correctly", async () => {
    const now = 1_000_000_000_000;
    const tenDaysAgo = now - 10 * 24 * 60 * 60 * 1000;

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue({
      last_ping_timestamp: tenDaysAgo,
      inactivity_period_days: 180,
      days_until_claimable: 170,
      is_claimable: false,
    });

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.timerState.days).toBe(170);
  });

  it("sets isSoonWarning when timer is below threshold", async () => {
    const now = 1_000_000_000_000;
    const almostExpired = now - 179 * 24 * 60 * 60 * 1000;

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue({
      last_ping_timestamp: almostExpired,
      inactivity_period_days: 180,
      days_until_claimable: 0,
      is_claimable: false,
    });

    const { result } = renderHook(() =>
      useInactivityTimer({
        planId: mockPlanId,
        warningThresholdHours: 24,
      })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.timerState.isSoonWarning).toBe(true);
  });

  it("marks plan as claimable when time expires", async () => {
    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue({
      last_ping_timestamp: Date.now() - 200 * 24 * 60 * 60 * 1000,
      inactivity_period_days: 180,
      days_until_claimable: 0,
      is_claimable: true,
    });

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.timerState.isClaimable).toBe(true);
  });

  it("updates timer every second via client-side tick", async () => {
    const now = 1_000_000_000_000;

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue({
      last_ping_timestamp: now - 1000,
      inactivity_period_days: 180,
      days_until_claimable: 179,
      is_claimable: false,
    });

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const initial = result.current.timerState.seconds;

    await act(async () => {
      vi.advanceTimersByTime(1000);
    });

    await waitFor(() => {
      expect(result.current.timerState.seconds).toBe(
        initial - 1 >= 0 ? initial - 1 : 59
      );
    });
  });

  it("calls ping API and refetches status", async () => {
    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue({
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    });

    vi.mocked(plansAPI.pingKeepAlive).mockResolvedValue({
      id: "plan-123",
    } as any);

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.ping("signed-xdr-123");
    });

    expect(plansAPI.pingKeepAlive).toHaveBeenCalledWith(
      mockPlanId,
      "signed-xdr-123"
    );
  });

  it("handles ping error gracefully", async () => {
    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue({
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    });

    vi.mocked(plansAPI.pingKeepAlive).mockRejectedValue(
      new Error("Ping failed")
    );

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      try {
        await result.current.ping("signed-xdr-123");
      } catch {}
    });

    expect(result.current.error).toBeTruthy();
  });

  it("polls status at specified interval", async () => {
    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue({
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    });

    const { result } = renderHook(() =>
      useInactivityTimer({
        planId: mockPlanId,
        pollIntervalMs: 5000,
      })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(plansAPI.getInactivityStatus).toHaveBeenCalledTimes(1);

    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    await waitFor(() => {
      expect(plansAPI.getInactivityStatus).toHaveBeenCalledTimes(2);
    });
  });

  it("respects enabled flag", () => {
    renderHook(() =>
      useInactivityTimer({
        planId: mockPlanId,
        enabled: false,
      })
    );

    expect(plansAPI.getInactivityStatus).not.toHaveBeenCalled();
  });
});