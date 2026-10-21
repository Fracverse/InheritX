import { afterEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import OfflineSyncHandler from "@/components/ui/OfflineSyncHandler";
import { offlineQueue } from "@/lib/offline/offlineQueue";

afterEach(() => {
  offlineQueue.setOnline(false);
  offlineQueue.clear();
  Object.defineProperty(navigator, "onLine", { configurable: true, value: true });
});

describe("OfflineSyncHandler", () => {
  it("shows the offline status and queued request count", () => {
    Object.defineProperty(navigator, "onLine", { configurable: true, value: false });
    offlineQueue.setOnline(false);
    offlineQueue.enqueue({ type: "rpc", title: "Ledger lookup", execute: async () => undefined });

    render(<OfflineSyncHandler />);

    expect(screen.getByRole("status")).toHaveTextContent("You are offline");
    expect(screen.getByRole("status")).toHaveTextContent("1 requests queued");
  });

  it("opens the queue drawer and lists queued operations", () => {
    Object.defineProperty(navigator, "onLine", { configurable: true, value: false });
    offlineQueue.setOnline(false);
    offlineQueue.enqueue({ type: "rpc", title: "Ledger lookup", execute: async () => undefined });

    render(<OfflineSyncHandler />);
    fireEvent.click(screen.getByRole("button", { name: "Show queued operations" }));

    expect(screen.getByText("Ledger lookup")).toBeInTheDocument();
    expect(screen.getByText("QUEUED")).toBeInTheDocument();
  });
});
