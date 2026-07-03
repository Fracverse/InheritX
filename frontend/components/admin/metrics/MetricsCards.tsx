import { AdminMetrics } from "@/app/lib/api/admin";

interface Props {
  metrics: AdminMetrics | null;
  loading: boolean;
}

const cards = [
  {
    key: "total_users" as keyof AdminMetrics,
    label: "Total Active Users",
    format: (v: number) => v.toLocaleString(),
    accent: "text-primary border-primary/20 bg-primary/5",
    dot: "bg-primary",
  },
  {
    key: "total_revenue" as keyof AdminMetrics,
    label: "Yield Generated",
    format: (v: number) => "$" + v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }),
    accent: "text-emerald-400 border-emerald-500/20 bg-emerald-500/5",
    dot: "bg-emerald-400",
  },
  {
    key: "total_claims" as keyof AdminMetrics,
    label: "Total Disbursed Funds",
    format: (v: number) => v.toLocaleString(),
    accent: "text-violet-400 border-violet-500/20 bg-violet-500/5",
    dot: "bg-violet-400",
  },
  {
    key: "active_plans" as keyof AdminMetrics,
    label: "Active Plans",
    format: (v: number) => v.toLocaleString(),
    accent: "text-orange-400 border-orange-500/20 bg-orange-500/5",
    dot: "bg-orange-400",
  },
];

function SkeletonCard() {
  return (
    <div className="rounded-xl border border-white/5 bg-white/[0.02] p-5 animate-pulse">
      <div className="h-3 w-24 rounded bg-white/10 mb-4" />
      <div className="h-8 w-32 rounded bg-white/10" />
    </div>
  );
}

export function MetricsCards({ metrics, loading }: Props) {
  if (loading) {
    return (
      <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
        {cards.map((_, i) => (
          <SkeletonCard key={i} />
        ))}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
      {cards.map(({ key, label, format, accent, dot }) => (
        <div key={key} className={`rounded-xl border p-5 ${accent}`}>
          <div className="flex items-center gap-2 mb-3">
            <span className={`w-2 h-2 rounded-full ${dot}`} />
            <span className="text-xs font-medium text-foreground/60 uppercase tracking-wide">{label}</span>
          </div>
          <p className="text-2xl font-semibold tabular-nums">{metrics ? format(metrics[key]) : "—"}</p>
        </div>
      ))}
    </div>
  );
}
