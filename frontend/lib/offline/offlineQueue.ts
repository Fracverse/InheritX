export type QueueItemType = "transaction" | "rpc" | "balance";
export type QueueItemStatus = "QUEUED" | "SYNCING" | "COMPLETED" | "FAILED";

export interface QueuedRequest {
  id: string;
  type: QueueItemType;
  title: string;
  timestamp: number;
  status: QueueItemStatus;
  retries: number;
  error?: string;
}

export interface QueueState {
  items: QueuedRequest[];
  isOnline: boolean;
  isSyncing: boolean;
  progress: { current: number; total: number };
}

interface QueueEntry {
  item: QueuedRequest;
  execute: () => Promise<unknown>;
  onFailure?: (error: Error) => void;
}

export const MAX_QUEUE_SIZE = 50;
export const MAX_RETRIES = 3;
const REPLAY_SPACING_MS = 250;
const RETRY_BASE_MS = 500;

const listeners = new Set<() => void>();
const entries: QueueEntry[] = [];
let state: QueueState = {
  items: [],
  isOnline: typeof navigator === "undefined" ? true : navigator.onLine,
  isSyncing: false,
  progress: { current: 0, total: 0 },
};
let replayPromise: Promise<void> | null = null;

function publish() {
  state = { ...state, items: entries.map(({ item }) => ({ ...item })) };
  listeners.forEach((listener) => listener());
}

function wait(ms: number) {
  return new Promise<void>((resolve) => setTimeout(resolve, ms));
}

async function executeEntry(entry: QueueEntry) {
  entry.item.status = "SYNCING";
  entry.item.error = undefined;
  publish();
  try {
    await entry.execute();
    entry.item.status = "COMPLETED";
    entry.item.retries = 0;
  } catch (error) {
    entry.item.retries += 1;
    entry.item.error = error instanceof Error ? error.message : "Operation failed";
    if (entry.item.retries >= MAX_RETRIES) {
      entry.item.status = "FAILED";
      entry.onFailure?.(error instanceof Error ? error : new Error("Operation failed"));
    } else {
      entry.item.status = "QUEUED";
      await wait(RETRY_BASE_MS * 2 ** (entry.item.retries - 1));
      if (state.isOnline) return executeEntry(entry);
    }
  }
  publish();
}

async function replayAll() {
  if (replayPromise) return replayPromise;
  const pending = entries.filter(({ item }) => item.status === "QUEUED" || item.status === "FAILED");
  if (!pending.length || !state.isOnline) return;
  replayPromise = (async () => {
    state = { ...state, isSyncing: true };
    state.progress = { current: 0, total: pending.length };
    publish();
    for (const entry of pending) {
      if (!state.isOnline) break;
      entry.item.retries = 0;
      await executeEntry(entry);
      state.progress = { ...state.progress, current: state.progress.current + 1 };
      publish();
      if (state.isOnline) await wait(REPLAY_SPACING_MS);
    }
    state = { ...state, isSyncing: false };
    publish();
  })().finally(() => {
    replayPromise = null;
  });
  return replayPromise;
}

export const offlineQueue = {
  subscribe(listener: () => void) {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },
  getSnapshot(): QueueState {
    return state;
  },
  setOnline(isOnline: boolean) {
    state = { ...state, isOnline };
    publish();
    if (isOnline && entries.some(({ item }) => item.status === "QUEUED" || item.status === "FAILED")) void replayAll();
  },
  enqueue(input: { type: QueueItemType; title: string; execute: () => Promise<unknown>; onFailure?: (error: Error) => void }) {
    if (typeof navigator !== "undefined" && state.isOnline !== navigator.onLine) {
      state = { ...state, isOnline: navigator.onLine };
    }
    if (entries.length >= MAX_QUEUE_SIZE) {
      const evictable = entries.findIndex(({ item }) => item.status !== "SYNCING" && item.type !== "transaction");
      if (evictable < 0) throw new Error("Offline queue is full; no non-critical item can be evicted.");
      const [removed] = entries.splice(evictable, 1);
      removed?.onFailure?.(new Error("Operation evicted because the offline queue is full."));
    }
    const item: QueuedRequest = {
      id: typeof crypto !== "undefined" && "randomUUID" in crypto ? crypto.randomUUID() : `${Date.now()}-${Math.random()}`,
      type: input.type,
      title: input.title,
      timestamp: Date.now(),
      status: "QUEUED",
      retries: 0,
    };
    entries.push({ item, execute: input.execute, onFailure: input.onFailure });
    publish();
    if (state.isOnline) void replayAll();
    return item.id;
  },
  async retry(id: string) {
    const entry = entries.find(({ item }) => item.id === id);
    if (!entry) return;
    entry.item.status = "QUEUED";
    entry.item.retries = 0;
    publish();
    if (state.isOnline) await replayAll();
  },
  async retryAll() {
    for (const entry of entries) {
      if (entry.item.status === "FAILED") {
        entry.item.status = "QUEUED";
        entry.item.retries = 0;
      }
    }
    publish();
    if (state.isOnline) await replayAll();
  },
  clear() {
    if (state.isSyncing) return;
    entries.forEach((entry) => entry.onFailure?.(new Error("Operation removed from offline queue.")));
    entries.length = 0;
    publish();
  },
  replayAll,
};

export function queueRpcRequest<T>(options: {
  title: string;
  type?: QueueItemType;
  request: () => Promise<T>;
}): Promise<T> {
  if (typeof navigator === "undefined" || navigator.onLine) return options.request();
  return new Promise<T>((resolve, reject) => {
    offlineQueue.enqueue({
      type: options.type ?? "rpc",
      title: options.title,
      execute: async () => resolve(await options.request()),
      onFailure: reject,
    });
  });
}
