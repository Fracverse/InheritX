"use client";

import React, { useState, useMemo } from "react";
import {
  Clock,
  ArrowUpRight,
  FileText,
  Heart,
  Wallet,
  TrendingUp,
  Shield,
  AlertTriangle,
  ChevronDown,
  Link as LinkIcon,
} from "lucide-react";

export interface PlanAuditEvent {
  id: string;
  timestamp: number;
  type: "CREATED" | "PINGED" | "BENEFICIARY_MODIFIED" | "LOAN_MINTED" | "PAYOUT_TRIGGERED" | "GUARDIAN_UPDATED" | "REGISTRY_LINKED";
  txHash: string;
  actor: string;
  details?: string;
}

const EVENT_CONFIG: Record<
  PlanAuditEvent["type"],
  { label: string; color: string; bg: string; border: string; icon: React.ReactNode }
> = {
  CREATED: {
    label: "Plan Created",
    color: "text-[#33C5E0]",
    bg: "bg-[#33C5E0]/10",
    border: "border-[#33C5E0]/20",
    icon: <FileText size={14} />,
  },
  PINGED: {
    label: "Proof of Life Ping",
    color: "text-emerald-400",
    bg: "bg-emerald-400/10",
    border: "border-emerald-400/20",
    icon: <Heart size={14} />,
  },
  BENEFICIARY_MODIFIED: {
    label: "Beneficiaries Updated",
    color: "text-violet-400",
    bg: "bg-violet-400/10",
    border: "border-violet-400/20",
    icon: <Wallet size={14} />,
  },
  LOAN_MINTED: {
    label: "Loan Minted",
    color: "text-orange-400",
    bg: "bg-orange-400/10",
    border: "border-orange-400/20",
    icon: <TrendingUp size={14} />,
  },
  PAYOUT_TRIGGERED: {
    label: "Payout Triggered",
    color: "text-yellow-400",
    bg: "bg-yellow-400/10",
    border: "border-yellow-400/20",
    icon: <ArrowUpRight size={14} />,
  },
  GUARDIAN_UPDATED: {
    label: "Guardians Updated",
    color: "text-pink-400",
    bg: "bg-pink-400/10",
    border: "border-pink-400/20",
    icon: <Shield size={14} />,
  },
  REGISTRY_LINKED: {
    label: "Registry Linked",
    color: "text-sky-400",
    bg: "bg-sky-400/10",
    border: "border-sky-400/20",
    icon: <LinkIcon size={14} />,
  },
};

const EVENT_CATEGORIES = ["All", "Lifecycle", "Beneficiaries", "Financial", "Security"] as const;

const CATEGORY_MAP: Record<string, PlanAuditEvent["type"][]> = {
  Lifecycle: ["CREATED", "PINGED"],
  Beneficiaries: ["BENEFICIARY_MODIFIED"],
  Financial: ["LOAN_MINTED", "PAYOUT_TRIGGERED"],
  Security: ["GUARDIAN_UPDATED", "REGISTRY_LINKED"],
};

function formatTimestamp(ts: number): string {
  return new Date(ts).toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatRelative(ts: number): string {
  const diff = Date.now() - ts;
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function AuditEventCard({ event, isLast }: { event: PlanAuditEvent; isLast: boolean }) {
  const [showDetails, setShowDetails] = useState(false);
  const config = EVENT_CONFIG[event.type];

  return (
    <div className="relative flex gap-4">
      {/* Timeline line */}
      {!isLast && (
        <div className="absolute left-[19px] top-10 bottom-0 w-px bg-white/[0.06]" aria-hidden="true" />
      )}

      {/* Icon */}
      <div
        className={`relative z-10 w-10 h-10 rounded-xl ${config.bg} border ${config.border} flex items-center justify-center flex-shrink-0 ${config.color}`}
        aria-hidden="true"
      >
        {config.icon}
      </div>

      {/* Content */}
      <div className="flex-1 pb-6 min-w-0">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            <p className="text-sm font-medium text-white truncate">{config.label}</p>
            <div className="flex items-center gap-2 mt-1 text-[11px] text-slate-500">
              <Clock size={10} className="flex-shrink-0" />
              <span title={formatTimestamp(event.timestamp)}>
                {formatRelative(event.timestamp)}
              </span>
              <span aria-hidden="true">·</span>
              <span className="font-mono truncate max-w-[120px]">
                {event.actor.slice(0, 6)}…{event.actor.slice(-4)}
              </span>
            </div>
          </div>
          <button
            onClick={() => setShowDetails(!showDetails)}
            className="flex items-center gap-1 text-[11px] text-slate-500 hover:text-slate-300 transition-colors flex-shrink-0"
            aria-expanded={showDetails}
            aria-label={showDetails ? "Hide details" : "Show details"}
          >
            Details
            <ChevronDown
              size={12}
              className={`transition-transform ${showDetails ? "rotate-180" : ""}`}
            />
          </button>
        </div>

        {/* Expandable details */}
        {showDetails && (
          <div className="mt-3 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] space-y-2">
            <div className="flex items-center gap-2 text-[11px]">
              <span className="text-slate-500">Transaction:</span>
              <a
                href={`https://stellar.expert/explorer/testnet/tx/${event.txHash}`}
                target="_blank"
                rel="noopener noreferrer"
                className="font-mono text-[#33C5E0] hover:underline truncate max-w-[200px]"
              >
                {event.txHash.slice(0, 10)}…
              </a>
              <ArrowUpRight size={10} className="text-slate-600 flex-shrink-0" />
            </div>
            <div className="flex items-center gap-2 text-[11px]">
              <span className="text-slate-500">Actor:</span>
              <span className="font-mono text-slate-300 truncate">{event.actor}</span>
            </div>
            {event.details && (
              <div className="text-[11px] text-slate-400">{event.details}</div>
            )}
            <div className="text-[11px] text-slate-500">
              {formatTimestamp(event.timestamp)}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

interface PlanAuditTimelineProps {
  events: PlanAuditEvent[];
  planId?: string;
}

export function PlanAuditTimeline({ events, planId }: PlanAuditTimelineProps) {
  const [category, setCategory] = useState<(typeof EVENT_CATEGORIES)[number]>("All");

  const filtered = useMemo(() => {
    if (category === "All") return events;
    const types = CATEGORY_MAP[category] ?? [];
    return events.filter((e) => types.includes(e.type));
  }, [events, category]);

  const sorted = useMemo(
    () => [...filtered].sort((a, b) => b.timestamp - a.timestamp),
    [filtered]
  );

  return (
    <div className="rounded-2xl bg-[#0A0F11] border border-white/[0.06] p-6" data-testid="plan-audit-timeline">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div>
          <h3 className="text-base font-semibold text-white">Plan Activity</h3>
          {planId && (
            <p className="text-[11px] text-slate-500 font-mono mt-0.5">#{planId.slice(0, 8)}</p>
          )}
        </div>
        <span className="text-xs text-slate-500">{sorted.length} event{sorted.length !== 1 ? "s" : ""}</span>
      </div>

      {/* Category filter */}
      <div
        className="flex items-center gap-1 p-1 rounded-xl bg-white/[0.03] border border-white/[0.06] mb-6 overflow-x-auto"
        role="tablist"
        aria-label="Filter events by category"
      >
        {EVENT_CATEGORIES.map((cat) => (
          <button
            key={cat}
            role="tab"
            aria-selected={category === cat}
            onClick={() => setCategory(cat)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors whitespace-nowrap ${
              category === cat
                ? "bg-[#33C5E0] text-black"
                : "text-slate-400 hover:text-white"
            }`}
          >
            {cat}
          </button>
        ))}
      </div>

      {/* Timeline */}
      {sorted.length === 0 ? (
        <div className="py-12 text-center">
          <AlertTriangle size={24} className="text-slate-600 mx-auto mb-3" />
          <p className="text-sm text-slate-400">No events in this category</p>
          <p className="text-xs text-slate-600 mt-1">Try selecting a different filter</p>
        </div>
      ) : (
        <div className="space-y-0" role="list" aria-label="Plan activity timeline">
          {sorted.map((event, i) => (
            <AuditEventCard
              key={event.id}
              event={event}
              isLast={i === sorted.length - 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}
