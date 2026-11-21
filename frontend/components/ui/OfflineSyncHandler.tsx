"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import { Check, ChevronDown, ChevronUp, Database, Radio, RefreshCw, Send, Trash2, WifiOff, X } from "lucide-react";
import { offlineQueue, type QueueState } from "@/lib/offline/offlineQueue";

const SERVER_SNAPSHOT: QueueState = { items: [], isOnline: true, isSyncing: false, progress: { current: 0, total: 0 } };
const getServerSnapshot = () => SERVER_SNAPSHOT;

export default function OfflineSyncHandler() {
  const state = useSyncExternalStore(offlineQueue.subscribe, offlineQueue.getSnapshot, getServerSnapshot);
  const [expanded, setExpanded] = useState(false);
  const [restored, setRestored] = useState(false);

  useEffect(() => {
    const update = () => offlineQueue.setOnline(navigator.onLine);
    update();
    window.addEventListener("online", update);
    window.addEventListener("offline", update);
    return () => {
      window.removeEventListener("online", update);
      window.removeEventListener("offline", update);
    };
  }, []);

  useEffect(() => {
    if (!state.isOnline || state.isSyncing || state.items.length === 0) return;
    if (!state.items.every((item) => item.status === "COMPLETED")) return;
    setRestored(true);
    const timer = window.setTimeout(() => setRestored(false), 3000);
    return () => window.clearTimeout(timer);
  }, [state.isOnline, state.isSyncing, state.items]);

  if (state.isOnline && !state.isSyncing && !restored && state.items.length === 0) return null;

  const hasFailed = state.items.some((item) => item.status === "FAILED");
  const status = !state.isOnline ? "offline" : state.isSyncing ? "syncing" : restored ? "restored" : hasFailed ? "failed" : "online";
  if (state.isOnline && !state.isSyncing && !restored && state.items.every((item) => item.status === "COMPLETED")) return null;
  const palette = status === "offline"
    ? "border-amber-500/30 text-amber-300"
    : status === "syncing"
      ? "border-cyan-500/30 text-cyan-300"
      : status === "failed"
        ? "border-rose-500/30 text-rose-300"
        : "border-emerald-500/30 text-emerald-300";

  return (
    <section className={`fixed bottom-4 right-4 z-[60] w-[min(28rem,calc(100vw-2rem))] rounded-xl border bg-[#0F1621]/95 p-4 shadow-2xl backdrop-blur ${palette}`} role="status" aria-live="polite">
      <div className="flex items-center gap-3">
        {!state.isOnline ? <WifiOff size={18} /> : status === "syncing" ? <RefreshCw size={18} className="animate-spin" /> : <Check size={18} />}
        <div className="min-w-0 flex-1">
          <p className="font-semibold">{status === "offline" ? "You are offline" : status === "syncing" ? `Back online — Replaying ${state.progress.current}/${state.progress.total} queued operations…` : status === "failed" ? "Some queued operations failed." : "All requests synchronized successfully."}</p>
          {status === "offline" && (() => {
            const count = state.items.filter((item) => item.status === "QUEUED").length;
            return <p className="text-xs opacity-80">{count} {count === 1 ? "request" : "requests"} queued</p>;
          })()}
        </div>
        <button type="button" onClick={() => setExpanded((value) => !value)} aria-label={expanded ? "Collapse queued operations" : "Show queued operations"} className="rounded p-1 focus-visible:outline focus-visible:outline-2 focus-visible:outline-amber-400">
          {expanded ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
        </button>
        <button type="button" onClick={() => offlineQueue.clear()} aria-label="Clear queue" className="rounded p-1 focus-visible:outline focus-visible:outline-2 focus-visible:outline-amber-400">
          <X size={16} />
        </button>
      </div>
      {expanded && <div className="mt-4 border-t border-white/10 pt-3">
        <ul className="max-h-64 space-y-2 overflow-auto">
          {state.items.map((item) => {
            const Icon = item.type === "transaction" ? Send : item.type === "balance" ? Database : Radio;
            return <li key={item.id} className="flex items-center gap-2 rounded-lg bg-white/5 p-2 text-xs">
              <Icon size={15} className="shrink-0" />
              <span className="min-w-0 flex-1 truncate">{item.title}<span className="ml-2 opacity-60">{new Date(item.timestamp).toLocaleTimeString()}</span></span>
              <span className="shrink-0">{item.status}</span>
              {item.status === "FAILED" && <button type="button" onClick={() => void offlineQueue.retry(item.id)} className="rounded p-1" aria-label={`Retry ${item.title}`}><RefreshCw size={13} /></button>}
              {item.error && <span className="sr-only">{item.error}</span>}
            </li>;
          })}
        </ul>
        <div className="mt-3 flex justify-end gap-2">
          <button type="button" onClick={() => void offlineQueue.retryAll()} className="rounded border border-white/15 px-3 py-1.5 text-xs">Retry All</button>
          <button type="button" onClick={() => offlineQueue.clear()} className="inline-flex items-center gap-1 rounded border border-white/15 px-3 py-1.5 text-xs"><Trash2 size={12} /> Clear Queue</button>
        </div>
      </div>}
    </section>
  );
}
