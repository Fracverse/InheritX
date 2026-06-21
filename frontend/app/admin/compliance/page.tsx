"use client";

import React, { FormEvent, useEffect, useMemo, useState } from "react";
import { motion } from "framer-motion";
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Download,
  FileText,
  Gauge,
  RefreshCw,
  Scale,
  Search,
  ShieldAlert,
  ShieldCheck,
  SlidersHorizontal,
  XCircle,
} from "lucide-react";
import {
  AlertStatus,
  ComplianceAlert,
  RiskLevel,
  RiskOverrideRequest,
  RiskScore,
  SanctionsCheck,
  createComplianceAPI,
} from "@/app/lib/api/compliance";

const DEFAULT_ADDRESS = "GDRISK7W7YQF4LQRYR6D2AH6FZKBX6E5D3EXAMPLEADDRESS";

const severityStyles: Record<string, string> = {
  low: "border-[#48BB78]/30 bg-[#48BB78]/10 text-[#48BB78]",
  medium: "border-[#ECC94B]/30 bg-[#ECC94B]/10 text-[#ECC94B]",
  high: "border-[#F56565]/30 bg-[#F56565]/10 text-[#F56565]",
  critical: "border-[#F56565]/50 bg-[#F56565]/15 text-[#F56565]",
  open: "border-[#F56565]/30 bg-[#F56565]/10 text-[#F56565]",
  investigating: "border-[#33C5E0]/30 bg-[#33C5E0]/10 text-[#33C5E0]",
  resolved: "border-[#48BB78]/30 bg-[#48BB78]/10 text-[#48BB78]",
};

function normalizeRiskLevel(score: number): RiskLevel {
  if (score >= 85) return "critical";
  if (score >= 70) return "high";
  if (score >= 40) return "medium";
  return "low";
}

function formatDate(value?: string) {
  if (!value) return "Not recorded";

  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

function formatNumber(value?: number) {
  if (value === undefined || Number.isNaN(value)) return "-";
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 2 }).format(value);
}

function riskColor(level: RiskLevel) {
  if (level === "critical" || level === "high") return "#F56565";
  if (level === "medium") return "#ECC94B";
  return "#48BB78";
}

function StatusBadge({ value }: { value: string }) {
  return (
    <span
      className={`inline-flex items-center rounded border px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider ${
        severityStyles[value.toLowerCase()] || "border-[#4A5568] bg-[#161E22] text-[#8899A6]"
      }`}
    >
      {value}
    </span>
  );
}

function MetricTile({
  icon: Icon,
  label,
  value,
  tone,
}: {
  icon: React.ElementType;
  label: string;
  value: string;
  tone: string;
}) {
  return (
    <div className="rounded-xl border border-[#161E22] bg-[#0A0F11] p-5">
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          <p className="text-xs font-semibold uppercase tracking-wider text-[#4A5568]">{label}</p>
          <p className="text-2xl font-bold text-white">{value}</p>
        </div>
        <div className="flex h-11 w-11 items-center justify-center rounded-lg bg-[#161E22]">
          <Icon size={20} color={tone} />
        </div>
      </div>
    </div>
  );
}

function exportFile(filename: string, content: string, type: string) {
  const blob = new Blob([content], { type });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function alertsToCsv(alerts: ComplianceAlert[]) {
  const header = [
    "id",
    "type",
    "address",
    "severity",
    "status",
    "reason",
    "amount",
    "asset_code",
    "event_count",
    "created_at",
  ];

  const rows = alerts.map((alert) =>
    header
      .map((key) => {
        const value = alert[key as keyof ComplianceAlert];
        return `"${String(value ?? "").replaceAll('"', '""')}"`;
      })
      .join(","),
  );

  return [header.join(","), ...rows].join("\n");
}

export default function ComplianceDashboardPage() {
  const api = useMemo(() => createComplianceAPI(), []);
  const [velocityAlerts, setVelocityAlerts] = useState<ComplianceAlert[]>([]);
  const [volumeAlerts, setVolumeAlerts] = useState<ComplianceAlert[]>([]);
  const [riskScore, setRiskScore] = useState<RiskScore | null>(null);
  const [sanctionsCheck, setSanctionsCheck] = useState<SanctionsCheck | null>(null);
  const [alertStatuses, setAlertStatuses] = useState<Record<string, AlertStatus>>({});
  const [address, setAddress] = useState(DEFAULT_ADDRESS);
  const [overrideScore, setOverrideScore] = useState(35);
  const [overrideJustification, setOverrideJustification] = useState("");
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [screening, setScreening] = useState(false);
  const [submittingOverride, setSubmittingOverride] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [overrideMessage, setOverrideMessage] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<string | null>(null);

  const allAlerts = useMemo(() => {
    return [...velocityAlerts, ...volumeAlerts].map((alert) => ({
      ...alert,
      status: alertStatuses[alert.id] || alert.status || "open",
    }));
  }, [alertStatuses, velocityAlerts, volumeAlerts]);

  const flaggedTransactions = useMemo(
    () =>
      allAlerts.filter(
        (alert) =>
          alert.type === "flagged_transaction" ||
          alert.severity === "critical" ||
          alert.severity === "high",
      ),
    [allAlerts],
  );

  const openAlerts = allAlerts.filter((alert) => alert.status !== "resolved");

  const loadDashboard = async (isRefresh = false) => {
    setError(null);
    if (isRefresh) {
      setRefreshing(true);
    } else {
      setLoading(true);
    }

    try {
      const [velocity, volume] = await Promise.all([
        api.getVelocityAlerts(),
        api.getVolumeAlerts(),
      ]);
      setVelocityAlerts(velocity);
      setVolumeAlerts(volume);
      setLastUpdated(new Date().toISOString());
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unable to load compliance alerts");
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  useEffect(() => {
    loadDashboard();
    const interval = window.setInterval(() => loadDashboard(true), 60_000);
    return () => window.clearInterval(interval);
  }, []);

  const handleScreening = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!address.trim()) return;

    setScreening(true);
    setError(null);
    setOverrideMessage(null);

    try {
      const [score, sanctions] = await Promise.all([
        api.getRiskScore(address.trim()),
        api.checkSanctions(address.trim()),
      ]);
      setRiskScore(score);
      setSanctionsCheck(sanctions);
      setOverrideScore(score.score);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unable to screen this address");
    } finally {
      setScreening(false);
    }
  };

  const handleOverride = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!address.trim() || !overrideJustification.trim()) return;

    const payload: RiskOverrideRequest = {
      address: address.trim(),
      score: overrideScore,
      level: normalizeRiskLevel(overrideScore),
      justification: overrideJustification.trim(),
    };

    setSubmittingOverride(true);
    setError(null);
    setOverrideMessage(null);

    try {
      const override = await api.overrideRisk(payload);
      setRiskScore((current) => ({
        address: current?.address || payload.address,
        score: override.score,
        level: override.level,
        factors: current?.factors || [],
        last_evaluated_at: new Date().toISOString(),
        override,
      }));
      setOverrideJustification("");
      setOverrideMessage("Risk override recorded for audit review.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unable to submit risk override");
    } finally {
      setSubmittingOverride(false);
    }
  };

  const updateAlertStatus = (alertId: string, status: AlertStatus) => {
    setAlertStatuses((current) => ({ ...current, [alertId]: status }));
  };

  const exportJsonReport = () => {
    exportFile(
      "inheritx-compliance-report.json",
      JSON.stringify(
        {
          generated_at: new Date().toISOString(),
          alerts: allAlerts,
          risk_score: riskScore,
          sanctions_check: sanctionsCheck,
        },
        null,
        2,
      ),
      "application/json",
    );
  };

  const exportCsvReport = () => {
    exportFile("inheritx-compliance-alerts.csv", alertsToCsv(allAlerts), "text/csv");
  };

  return (
    <div className="w-full min-w-0 space-y-8 pb-10">
      <motion.div
        initial={{ opacity: 0, y: -12 }}
        animate={{ opacity: 1, y: 0 }}
        className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between"
      >
        <div className="space-y-2">
          <div className="flex items-center gap-3">
            <div className="flex h-11 w-11 items-center justify-center rounded-xl bg-[#33C5E0]/10 text-[#33C5E0]">
              <ShieldAlert size={22} />
            </div>
            <h1 className="text-2xl font-bold tracking-tight text-white md:text-3xl">
              Compliance & Risk
            </h1>
          </div>
          <p className="max-w-2xl text-sm font-medium text-[#8899A6] md:text-base">
            Monitor suspicious activity, screen wallets, override risk decisions, and export audit reports.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-3">
          <span className="rounded-lg border border-[#161E22] bg-[#0A0F11] px-3 py-2 text-xs font-semibold text-[#8899A6]">
            {lastUpdated ? `Updated ${formatDate(lastUpdated)}` : "Awaiting refresh"}
          </span>
          <button
            type="button"
            onClick={() => loadDashboard(true)}
            className="inline-flex items-center gap-2 rounded-lg border border-[#161E22] bg-[#0A0F11] px-4 py-2 text-sm font-semibold text-white hover:border-[#33C5E0]/50"
          >
            <RefreshCw size={16} className={refreshing ? "animate-spin" : ""} />
            Refresh
          </button>
        </div>
      </motion.div>

      {error && (
        <div className="flex items-start gap-3 rounded-xl border border-[#F56565]/30 bg-[#F56565]/10 p-4 text-sm text-[#F56565]">
          <AlertTriangle size={18} className="mt-0.5 shrink-0" />
          <p>{error}</p>
        </div>
      )}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricTile icon={Activity} label="Velocity Alerts" value={String(velocityAlerts.length)} tone="#33C5E0" />
        <MetricTile icon={Gauge} label="Volume Alerts" value={String(volumeAlerts.length)} tone="#ECC94B" />
        <MetricTile icon={AlertTriangle} label="Flagged Transactions" value={String(flaggedTransactions.length)} tone="#F56565" />
        <MetricTile icon={CheckCircle2} label="Open Cases" value={String(openAlerts.length)} tone="#48BB78" />
      </div>

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.5fr)_minmax(360px,0.8fr)]">
        <section className="rounded-xl border border-[#161E22] bg-[#0A0F11]">
          <div className="flex flex-col gap-3 border-b border-[#161E22] p-5 md:flex-row md:items-center md:justify-between">
            <div>
              <h2 className="text-lg font-bold text-white">Compliance Alerts</h2>
              <p className="text-sm text-[#8899A6]">Velocity, volume, and high-risk transaction events.</p>
            </div>
            <StatusBadge value={loading ? "investigating" : "open"} />
          </div>

          <div className="overflow-x-auto">
            <table className="w-full min-w-[900px] border-collapse">
              <thead>
                <tr className="border-b border-[#161E22]">
                  <th className="px-5 py-4 text-left text-[10px] font-bold uppercase tracking-wider text-[#4A5568]">Alert</th>
                  <th className="px-5 py-4 text-left text-[10px] font-bold uppercase tracking-wider text-[#4A5568]">Address</th>
                  <th className="px-5 py-4 text-left text-[10px] font-bold uppercase tracking-wider text-[#4A5568]">Signal</th>
                  <th className="px-5 py-4 text-left text-[10px] font-bold uppercase tracking-wider text-[#4A5568]">Created</th>
                  <th className="px-5 py-4 text-left text-[10px] font-bold uppercase tracking-wider text-[#4A5568]">Resolution</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[#161E22]">
                {loading ? (
                  <tr>
                    <td colSpan={5} className="px-5 py-14 text-center text-sm text-[#8899A6]">
                      Loading compliance alerts...
                    </td>
                  </tr>
                ) : allAlerts.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-5 py-14 text-center">
                      <div className="flex flex-col items-center gap-3 text-[#8899A6]">
                        <ShieldCheck size={34} />
                        <p className="text-sm font-medium">No compliance alerts returned.</p>
                      </div>
                    </td>
                  </tr>
                ) : (
                  allAlerts.map((alert) => (
                    <tr key={alert.id} className="hover:bg-[#161E22]/30">
                      <td className="px-5 py-4 align-top">
                        <div className="space-y-2">
                          <div className="flex items-center gap-2">
                            <StatusBadge value={alert.severity} />
                            <span className="text-xs font-semibold uppercase text-[#8899A6]">{alert.type.replaceAll("_", " ")}</span>
                          </div>
                          <p className="max-w-md text-sm font-semibold text-white">{alert.reason}</p>
                          {alert.transaction_hash && (
                            <p className="font-mono text-xs text-[#4A5568]">{alert.transaction_hash}</p>
                          )}
                        </div>
                      </td>
                      <td className="px-5 py-4 align-top">
                        <button
                          type="button"
                          onClick={() => setAddress(alert.address)}
                          className="max-w-[220px] truncate font-mono text-xs text-[#33C5E0] hover:underline"
                        >
                          {alert.address}
                        </button>
                      </td>
                      <td className="px-5 py-4 align-top">
                        <div className="space-y-1 text-sm">
                          <p className="font-semibold text-white">
                            {alert.amount ? `${formatNumber(alert.amount)} ${alert.asset_code || ""}` : `${formatNumber(alert.event_count)} events`}
                          </p>
                          <p className="text-xs text-[#8899A6]">
                            Threshold {formatNumber(alert.threshold)}
                            {alert.window_minutes ? ` / ${alert.window_minutes}m` : ""}
                          </p>
                        </div>
                      </td>
                      <td className="px-5 py-4 align-top text-sm text-[#8899A6]">{formatDate(alert.created_at)}</td>
                      <td className="px-5 py-4 align-top">
                        <select
                          value={alert.status}
                          onChange={(event) => updateAlertStatus(alert.id, event.target.value as AlertStatus)}
                          className="rounded-lg border border-[#161E22] bg-[#060B0D] px-3 py-2 text-sm font-semibold text-white outline-none focus:border-[#33C5E0]/60"
                        >
                          <option value="open">Open</option>
                          <option value="investigating">Investigating</option>
                          <option value="resolved">Resolved</option>
                        </select>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </section>

        <div className="space-y-6">
          <section className="rounded-xl border border-[#161E22] bg-[#0A0F11] p-5">
            <div className="mb-5 flex items-center gap-3">
              <Search size={20} className="text-[#33C5E0]" />
              <div>
                <h2 className="text-lg font-bold text-white">Risk & Sanctions Screening</h2>
                <p className="text-sm text-[#8899A6]">Check a wallet before approval or payout.</p>
              </div>
            </div>

            <form onSubmit={handleScreening} className="space-y-4">
              <input
                value={address}
                onChange={(event) => setAddress(event.target.value)}
                className="w-full rounded-xl border border-[#161E22] bg-[#060B0D] px-4 py-3 font-mono text-sm text-white outline-none placeholder:text-[#4A5568] focus:border-[#33C5E0]/60"
                placeholder="Enter wallet address"
              />
              <button
                type="submit"
                disabled={screening}
                className="inline-flex w-full items-center justify-center gap-2 rounded-lg bg-[#33C5E0] px-4 py-3 text-sm font-bold text-[#060B0D] disabled:cursor-not-allowed disabled:opacity-60"
              >
                <Search size={16} />
                {screening ? "Screening..." : "Run Screening"}
              </button>
            </form>

            {riskScore && (
              <div className="mt-5 rounded-xl border border-[#161E22] bg-[#060B0D] p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-xs font-semibold uppercase tracking-wider text-[#4A5568]">Risk Score</p>
                    <p className="text-3xl font-bold text-white">{riskScore.score}/100</p>
                  </div>
                  <div className="text-right">
                    <StatusBadge value={riskScore.level} />
                    <div
                      className="mt-3 h-2 w-28 rounded-full bg-[#161E22]"
                      aria-label={`Risk score ${riskScore.score}`}
                    >
                      <div
                        className="h-2 rounded-full"
                        style={{ width: `${Math.min(riskScore.score, 100)}%`, backgroundColor: riskColor(riskScore.level) }}
                      />
                    </div>
                  </div>
                </div>

                <div className="mt-4 space-y-3">
                  {riskScore.factors.length === 0 ? (
                    <p className="text-sm text-[#8899A6]">No risk factors returned for this address.</p>
                  ) : (
                    riskScore.factors.map((factor) => (
                      <div key={`${factor.label}-${factor.score}`} className="rounded-lg border border-[#161E22] p-3">
                        <div className="flex items-center justify-between gap-3">
                          <p className="text-sm font-semibold text-white">{factor.label}</p>
                          <span className="text-xs font-bold text-[#8899A6]">{factor.score}</span>
                        </div>
                        <p className="mt-1 text-xs text-[#8899A6]">{factor.description}</p>
                      </div>
                    ))
                  )}
                </div>
              </div>
            )}

            {sanctionsCheck && (
              <div className="mt-5 rounded-xl border border-[#161E22] bg-[#060B0D] p-4">
                <div className="flex items-center justify-between gap-3">
                  <div className="flex items-center gap-3">
                    {sanctionsCheck.is_flagged ? (
                      <XCircle className="text-[#F56565]" size={22} />
                    ) : (
                      <CheckCircle2 className="text-[#48BB78]" size={22} />
                    )}
                    <div>
                      <p className="text-sm font-bold text-white">
                        {sanctionsCheck.is_flagged ? "Address flagged" : "No sanctions match"}
                      </p>
                      <p className="text-xs text-[#8899A6]">{formatDate(sanctionsCheck.checked_at)}</p>
                    </div>
                  </div>
                  <StatusBadge value={sanctionsCheck.status} />
                </div>
                <p className="mt-3 text-sm text-[#8899A6]">{sanctionsCheck.recommendation}</p>
                {sanctionsCheck.lists.length > 0 && (
                  <div className="mt-3 flex flex-wrap gap-2">
                    {sanctionsCheck.lists.map((list) => (
                      <span key={list} className="rounded border border-[#F56565]/30 bg-[#F56565]/10 px-2 py-1 text-xs font-semibold text-[#F56565]">
                        {list}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            )}
          </section>

          <section className="rounded-xl border border-[#161E22] bg-[#0A0F11] p-5">
            <div className="mb-5 flex items-center gap-3">
              <SlidersHorizontal size={20} className="text-[#ECC94B]" />
              <div>
                <h2 className="text-lg font-bold text-white">Risk Override</h2>
                <p className="text-sm text-[#8899A6]">Adjust a score with an auditable justification.</p>
              </div>
            </div>

            <form onSubmit={handleOverride} className="space-y-4">
              <div>
                <label className="mb-2 block text-xs font-bold uppercase tracking-wider text-[#4A5568]">
                  Override score
                </label>
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={overrideScore}
                  onChange={(event) => setOverrideScore(Number(event.target.value))}
                  className="w-full accent-[#33C5E0]"
                />
                <div className="mt-2 flex items-center justify-between text-sm">
                  <span className="font-bold text-white">{overrideScore}/100</span>
                  <StatusBadge value={normalizeRiskLevel(overrideScore)} />
                </div>
              </div>

              <textarea
                value={overrideJustification}
                onChange={(event) => setOverrideJustification(event.target.value)}
                rows={4}
                className="w-full resize-none rounded-xl border border-[#161E22] bg-[#060B0D] px-4 py-3 text-sm text-white outline-none placeholder:text-[#4A5568] focus:border-[#33C5E0]/60"
                placeholder="Justification for audit trail"
              />
              <button
                type="submit"
                disabled={submittingOverride || !overrideJustification.trim()}
                className="inline-flex w-full items-center justify-center gap-2 rounded-lg bg-white px-4 py-3 text-sm font-bold text-[#060B0D] disabled:cursor-not-allowed disabled:opacity-60"
              >
                <Scale size={16} />
                {submittingOverride ? "Submitting..." : "Submit Override"}
              </button>
            </form>

            {overrideMessage && (
              <p className="mt-3 rounded-lg border border-[#48BB78]/30 bg-[#48BB78]/10 px-3 py-2 text-sm text-[#48BB78]">
                {overrideMessage}
              </p>
            )}
          </section>
        </div>
      </div>

      <section className="rounded-xl border border-[#161E22] bg-[#0A0F11] p-5">
        <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="flex items-center gap-3">
            <FileText size={20} className="text-[#33C5E0]" />
            <div>
              <h2 className="text-lg font-bold text-white">Compliance Reports</h2>
              <p className="text-sm text-[#8899A6]">Export current alerts, risk scores, sanctions results, and resolution state.</p>
            </div>
          </div>
          <div className="flex flex-wrap gap-3">
            <button
              type="button"
              onClick={exportCsvReport}
              className="inline-flex items-center gap-2 rounded-lg border border-[#161E22] bg-[#060B0D] px-4 py-2 text-sm font-semibold text-white hover:border-[#33C5E0]/50"
            >
              <Download size={16} />
              CSV
            </button>
            <button
              type="button"
              onClick={exportJsonReport}
              className="inline-flex items-center gap-2 rounded-lg border border-[#161E22] bg-[#060B0D] px-4 py-2 text-sm font-semibold text-white hover:border-[#33C5E0]/50"
            >
              <Download size={16} />
              JSON
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}
