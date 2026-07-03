import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useInactivityTimer } from "@/app/hooks/useInactivityTimer";
import { plansAPI } from "@/app/lib/api/plans";

// Mock the plans API
vi.mock("@/app/lib/api/plans");

describe("useInactivityTimer", () => {
  const mockPlanId = "plan-123";

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  it("initializes with loading state and fetches inactivity status", async () => {
    const mockStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(mockStatus);

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.timerState.isClaimable).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it("calculates time remaining correctly", async () => {
    const now = Date.now();
    const tenDaysAgo = now - 10 * 24 * 60 * 60 * 1000;

    const mockStatus = {
      last_ping_timestamp: tenDaysAgo,
      inactivity_period_days: 180,
      days_until_claimable: 170,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(mockStatus);

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.timerState.days).toBe(170);
    expect(result.current.timerState.isClaimable).toBe(false);
  });

  it("sets isSoonWarning when timer is below threshold", async () => {
    const now = Date.now();
    const alreadyPassed = now - (179 * 24 + 1) * 60 * 60 * 1000; // 179 days ago

    const mockStatus = {
      last_ping_timestamp: alreadyPassed,
      inactivity_period_days: 180,
      days_until_claimable: 0,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(mockStatus);

    const { result } = renderHook(() =>
      useInactivityTimer({
        planId: mockPlanId,
        warningThresholdHours: 24,
      })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // Should be in warning state
    expect(result.current.timerState.isSoonWarning).toBe(true);
  });

  it("marks plan as claimable when time expires", async () => {
    const now = Date.now();
    const longAgo = now - 200 * 24 * 60 * 60 * 1000; // 200 days ago

    const mockStatus = {
      last_ping_timestamp: longAgo,
      inactivity_period_days: 180,
      days_until_claimable: 0,
      is_claimable: true,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(mockStatus);

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.timerState.isClaimable).toBe(true);
  });

  it("updates timer every second via client-side tick", async () => {
    const now = Date.now();
    const mockStatus = {
      last_ping_timestamp: now - 1000, // 1 second ago
      inactivity_period_days: 180,
      days_until_claimable: 179,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(mockStatus);

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const initialSeconds = result.current.timerState.seconds;

    // Advance time by 1 second
    act(() => {
      vi.advanceTimersByTime(1000);
    });

    // Timer should update
    await waitFor(() => {
      expect(result.current.timerState.seconds).toBe(
        initialSeconds - 1 >= 0 ? initialSeconds - 1 : 59
      );
    });
  });

  it("calls ping API and refetches status", async () => {
    const mockStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    const mockPlan = {
      id: mockPlanId,
      user_id: "user-123",
      title: "Test Plan",
      fee: 100,
      net_amount: 1000,
      status: "active",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(mockStatus);
    vi.mocked(plansAPI.pingKeepAlive).mockResolvedValue(mockPlan);

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.ping("signed-xdr-123");
    });

    expect(vi.mocked(plansAPI.pingKeepAlive)).toHaveBeenCalledWith(
      mockPlanId,
      "signed-xdr-123"
    );
  });

  it("handles ping error gracefully", async () => {
    const mockStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    const pingError = new Error("Ping failed");
    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(mockStatus);
    vi.mocked(plansAPI.pingKeepAlive).mockRejectedValue(pingError);

    const { result } = renderHook(() =>
      useInactivityTimer({ planId: mockPlanId })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      try {
        await result.current.ping("signed-xdr-123");
      } catch {
        // Expected
      }
    });

    expect(result.current.error).toBeTruthy();
  });

  it("polls status at specified interval", async () => {
    const mockStatus = {
      last_ping_timestamp: Date.now(),
      inactivity_period_days: 180,
      days_until_claimable: 180,
      is_claimable: false,
    };

    vi.mocked(plansAPI.getInactivityStatus).mockResolvedValue(mockStatus);

    const { result } = renderHook(() =>
      useInactivityTimer({
        planId: mockPlanId,
        pollIntervalMs: 5000,
      })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // Initial call
    expect(vi.mocked(plansAPI.getInactivityStatus)).toHaveBeenCalledTimes(1);

    // Advance time to trigger poll
    act(() => {
      vi.advanceTimersByTime(5000);
    });

    await waitFor(() => {
      expect(vi.mocked(plansAPI.getInactivityStatus)).toHaveBeenCalledTimes(2);
    });
  });

  it("respects enabled flag", () => {
    const { result } = renderHook(() =>
      useInactivityTimer({
        planId: mockPlanId,
        enabled: false,
      })
    );

    expect(result.current.loading).toBe(true);
    expect(vi.mocked(plansAPI.getInactivityStatus)).not.toHaveBeenCalled();
  });
});
