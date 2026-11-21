import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAccountBalance, LOW_GAS_THRESHOLD_XLM, BALANCE_POLL_INTERVAL_MS } from "@/hooks/useAccountBalance";
import { useWallet } from "@/context/WalletContext";

vi.mock("@/context/WalletContext");

describe("useAccountBalance Hook", () => {
  const mockAddress = "GBZV2IQM4N5K6L7J8H9G0F1E2D3C4B5A6Z7Y8X9W";

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns null balance and idle state when wallet is not connected", () => {
    (useWallet as any).mockReturnValue({
      isConnected: false,
      address: null,
    });

    const { result } = renderHook(() => useAccountBalance());

    expect(result.current.xlmBalance).toBeNull();
    expect(result.current.formattedBalance).toBe("-- XLM");
    expect(result.current.isLowGas).toBe(false);
    expect(result.current.isUnfunded).toBe(false);
  });

  it("fetches and parses native XLM balance from Horizon when connected", async () => {
    (useWallet as any).mockReturnValue({
      isConnected: true,
      address: mockAddress,
    });

    const mockHorizonResponse = {
      id: mockAddress,
      balances: [
        { asset_type: "native", balance: "18.7500000" },
        { asset_type: "credit_alphanum4", asset_code: "USDC", balance: "50.0000000" },
      ],
    };

    const globalFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => mockHorizonResponse,
    });
    vi.stubGlobal("fetch", globalFetch);

    const { result } = renderHook(() => useAccountBalance());

    await act(async () => {
      await Promise.resolve();
    });

    expect(globalFetch).toHaveBeenCalledWith(
      expect.stringContaining(`/accounts/${mockAddress}`),
      expect.any(Object)
    );
    expect(result.current.xlmBalance).toBe(18.75);
    expect(result.current.formattedBalance).toBe("18.75 XLM");
    expect(result.current.isLowGas).toBe(false);
    expect(result.current.isUnfunded).toBe(false);
  });

  it("flags isLowGas=true when XLM balance is below 5 XLM", async () => {
    (useWallet as any).mockReturnValue({
      isConnected: true,
      address: mockAddress,
    });

    const mockHorizonResponse = {
      id: mockAddress,
      balances: [{ asset_type: "native", balance: "3.4200000" }],
    };

    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => mockHorizonResponse,
    }));

    const { result } = renderHook(() => useAccountBalance());

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.xlmBalance).toBe(3.42);
    expect(result.current.isLowGas).toBe(true);
  });

  it("handles 404 unfunded testnet account gracefully with 0 XLM and isLowGas=true", async () => {
    (useWallet as any).mockReturnValue({
      isConnected: true,
      address: mockAddress,
    });

    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: "Not Found",
    }));

    const { result } = renderHook(() => useAccountBalance());

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.xlmBalance).toBe(0);
    expect(result.current.formattedBalance).toBe("0.00 XLM");
    expect(result.current.isLowGas).toBe(true);
    expect(result.current.isUnfunded).toBe(true);
  });

  it("polls Horizon every 15 seconds", async () => {
    (useWallet as any).mockReturnValue({
      isConnected: true,
      address: mockAddress,
    });

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        id: mockAddress,
        balances: [{ asset_type: "native", balance: "20.0000000" }],
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    renderHook(() => useAccountBalance());

    // Initial fetch on mount
    await act(async () => {
      await Promise.resolve();
    });
    expect(fetchMock).toHaveBeenCalledTimes(1);

    // Fast-forward 15s
    await act(async () => {
      vi.advanceTimersByTime(BALANCE_POLL_INTERVAL_MS);
      await Promise.resolve();
    });

    expect(fetchMock).toHaveBeenCalledTimes(2);

    // Fast-forward another 15s
    await act(async () => {
      vi.advanceTimersByTime(BALANCE_POLL_INTERVAL_MS);
      await Promise.resolve();
    });

    expect(fetchMock).toHaveBeenCalledTimes(3);
  });

  it("refreshes immediately upon 'stellar:transaction-confirmed' event", async () => {
    (useWallet as any).mockReturnValue({
      isConnected: true,
      address: mockAddress,
    });

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        id: mockAddress,
        balances: [{ asset_type: "native", balance: "10.0000000" }],
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    renderHook(() => useAccountBalance());

    await act(async () => {
      await Promise.resolve();
    });
    expect(fetchMock).toHaveBeenCalledTimes(1);

    // Dispatch global transaction confirmation event
    await act(async () => {
      window.dispatchEvent(
        new CustomEvent("stellar:transaction-confirmed", { detail: { hash: "tx_123" } })
      );
      await Promise.resolve();
    });

    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it("refreshes immediately upon 'stellar:balance-refresh' event", async () => {
    (useWallet as any).mockReturnValue({
      isConnected: true,
      address: mockAddress,
    });

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        id: mockAddress,
        balances: [{ asset_type: "native", balance: "10.0000000" }],
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    renderHook(() => useAccountBalance());

    await act(async () => {
      await Promise.resolve();
    });
    expect(fetchMock).toHaveBeenCalledTimes(1);

    // Dispatch custom balance-refresh event
    await act(async () => {
      window.dispatchEvent(new CustomEvent("stellar:balance-refresh"));
      await Promise.resolve();
    });

    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it("runs a follow-up fetch when a refresh is requested during an in-flight request", async () => {
    (useWallet as any).mockReturnValue({ isConnected: true, address: mockAddress });

    let resolveFirst!: (response: unknown) => void;
    const firstResponse = new Promise((resolve) => {
      resolveFirst = resolve;
    });
    const fetchMock = vi.fn()
      .mockReturnValueOnce(firstResponse)
      .mockResolvedValue({
        ok: true,
        status: 200,
        json: async () => ({
          id: mockAddress,
          balances: [{ asset_type: "native", balance: "12.0000000" }],
        }),
      });
    vi.stubGlobal("fetch", fetchMock);

    renderHook(() => useAccountBalance());
    expect(fetchMock).toHaveBeenCalledTimes(1);

    await act(async () => {
      window.dispatchEvent(new CustomEvent("stellar:transaction-confirmed"));
      resolveFirst({
        ok: true,
        status: 200,
        json: async () => ({
          id: mockAddress,
          balances: [{ asset_type: "native", balance: "10.0000000" }],
        }),
      });
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(fetchMock).toHaveBeenCalledTimes(2);
  });
});
