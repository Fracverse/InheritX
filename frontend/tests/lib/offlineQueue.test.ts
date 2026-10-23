import { afterEach, describe, expect, it, vi } from "vitest";
import { offlineQueue } from "@/lib/offline/offlineQueue";

afterEach(() => {
  offlineQueue.setOnline(false);
  offlineQueue.clear();
});

describe("offlineQueue", () => {
  it("queues an operation offline and replays it when connectivity returns", async () => {
    const execute = vi.fn().mockResolvedValue(undefined);
    offlineQueue.setOnline(false);
    const id = offlineQueue.enqueue({ type: "rpc", title: "Fetch ledger", execute });

    expect(execute).not.toHaveBeenCalled();
    expect(offlineQueue.getSnapshot().items[0]).toMatchObject({ id, status: "QUEUED" });

    offlineQueue.setOnline(true);
    await vi.waitFor(() => expect(execute).toHaveBeenCalledTimes(1));
    await vi.waitFor(() => expect(offlineQueue.getSnapshot().items[0].status).toBe("COMPLETED"));
    await offlineQueue.replayAll();
  });

  it("marks an operation failed after three unsuccessful attempts", async () => {
    const execute = vi.fn().mockRejectedValue(new Error("network unavailable"));
    offlineQueue.setOnline(false);
    const id = offlineQueue.enqueue({ type: "balance", title: "Refresh balance", execute });

    offlineQueue.setOnline(true);
    await vi.waitFor(() => expect(offlineQueue.getSnapshot().items.find((item) => item.id === id)?.status).toBe("FAILED"), { timeout: 5000 });

    expect(execute).toHaveBeenCalledTimes(3);
    expect(offlineQueue.getSnapshot().items.find((item) => item.id === id)).toMatchObject({
      status: "FAILED",
      retries: 3,
      error: "network unavailable",
    });
  });
});
