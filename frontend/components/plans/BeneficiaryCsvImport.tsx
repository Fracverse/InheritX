"use client";

import React, { useState, useRef, useCallback } from "react";
import {
  Upload,
  Download,
  FileText,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Loader2,
} from "lucide-react";

interface CsvRow {
  rowNumber: number;
  walletAddress: string;
  name: string;
  allocationBps: number;
  email?: string;
  valid: boolean;
  errors: string[];
}

interface BeneficiaryCsvImportProps {
  onImport: (rows: CsvRow[]) => void;
  existingCount?: number;
}

function parseCsvLine(line: string): string[] {
  const result: string[] = [];
  let current = "";
  let inQuotes = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      if (inQuotes && line[i + 1] === '"') {
        current += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
      }
    } else if (ch === "," && !inQuotes) {
      result.push(current.trim());
      current = "";
    } else {
      current += ch;
    }
  }
  result.push(current.trim());
  return result;
}

function validateRow(walletAddress: string, allocationBps: number): string[] {
  const errors: string[] = [];
  if (!walletAddress || walletAddress.length < 32) {
    errors.push("Invalid Stellar address");
  }
  if (isNaN(allocationBps) || allocationBps < 0 || allocationBps > 10000) {
    errors.push("Allocation must be 0-10000 bps");
  }
  return errors;
}

export function BeneficiaryCsvImport({ onImport, existingCount = 0 }: BeneficiaryCsvImportProps) {
  const [rows, setRows] = useState<CsvRow[]>([]);
  const [isParsing, setIsParsing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const totalBps = rows.reduce((s, r) => s + (r.valid ? r.allocationBps : 0), 0);
  const allValid = rows.length > 0 && rows.every((r) => r.valid);
  const totalValid = totalBps === 10000;

  const handleFileSelect = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setIsParsing(true);
    setError(null);
    setRows([]);

    try {
      const text = await file.text();
      const lines = text.split(/\r?\n/).filter((l) => l.trim());
      if (lines.length < 2) {
        setError("CSV must have a header row and at least one data row");
        return;
      }

      const parsed: CsvRow[] = [];
      for (let i = 1; i < lines.length; i++) {
        const cols = parseCsvLine(lines[i]);
        const walletAddress = cols[0] || "";
        const name = cols[1] || "";
        const allocationBps = Number(cols[2]) || 0;
        const email = cols[3] || undefined;
        const errs = validateRow(walletAddress, allocationBps);

        parsed.push({
          rowNumber: i + 1,
          walletAddress,
          name,
          allocationBps,
          email,
          valid: errs.length === 0,
          errors: errs,
        });
      }
      setRows(parsed);
    } catch {
      setError("Failed to parse CSV file");
    } finally {
      setIsParsing(false);
    }
  }, []);

  const downloadTemplate = useCallback(() => {
    const csv = "wallet_address,name,allocation_bps,email\nGABC123...,Alice Johnson,5000,alice@example.com\nGDEF456...,Bob Smith,5000,bob@example.com";
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "beneficiaries-template.csv";
    a.click();
    URL.revokeObjectURL(url);
  }, []);

  const handleImport = () => {
    if (!allValid || !totalValid) return;
    onImport(rows);
    setRows([]);
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  const handleClear = () => {
    setRows([]);
    setError(null);
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  return (
    <div className="rounded-xl bg-white/[0.02] border border-white/[0.06] p-4" data-testid="beneficiary-csv-import">
      <div className="flex items-center justify-between mb-3">
        <h4 className="text-sm font-medium text-white flex items-center gap-2">
          <Upload size={14} className="text-[#33C5E0]" />
          Import Beneficiaries CSV
        </h4>
        <button
          onClick={downloadTemplate}
          className="flex items-center gap-1 text-[11px] text-[#33C5E0] hover:underline"
        >
          <Download size={11} />
          Template
        </button>
      </div>

      <p className="text-[11px] text-slate-500 mb-3">
        Upload a CSV with columns: wallet_address, name, allocation_bps, email
      </p>

      {/* File input */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".csv"
        onChange={handleFileSelect}
        className="hidden"
        id="csv-upload"
        aria-label="Upload beneficiaries CSV"
      />
      <label
        htmlFor="csv-upload"
        className="flex items-center justify-center gap-2 w-full py-3 border-2 border-dashed border-white/10 rounded-xl cursor-pointer hover:border-[#33C5E0]/30 hover:bg-[#33C5E0]/[0.02] transition-colors"
      >
        {isParsing ? (
          <Loader2 size={16} className="animate-spin text-slate-400" />
        ) : (
          <FileText size={16} className="text-slate-500" />
        )}
        <span className="text-sm text-slate-400">
          {isParsing ? "Parsing…" : "Click to select CSV file"}
        </span>
      </label>

      {/* Error */}
      {error && (
        <div className="mt-3 flex items-center gap-2 text-xs text-red-400 p-2 rounded-lg bg-red-500/[0.06]">
          <AlertTriangle size={12} />
          {error}
        </div>
      )}

      {/* Parsed rows preview */}
      {rows.length > 0 && (
        <div className="mt-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs text-slate-400">{rows.length} beneficiaries parsed</span>
            <span className={`text-xs font-medium ${totalValid ? "text-emerald-400" : "text-red-400"}`}>
              Total: {totalBps.toLocaleString()} / 10,000 bps
            </span>
          </div>

          {/* Table */}
          <div className="max-h-40 overflow-y-auto rounded-lg border border-white/[0.05]">
            <table className="w-full text-[11px]">
              <thead className="sticky top-0 bg-[#0A0F11]">
                <tr className="text-slate-500">
                  <th className="px-2 py-1.5 text-left">Row</th>
                  <th className="px-2 py-1.5 text-left">Address</th>
                  <th className="px-2 py-1.5 text-left">Name</th>
                  <th className="px-2 py-1.5 text-right">BPS</th>
                  <th className="px-2 py-1.5 text-center">Status</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr key={r.rowNumber} className={`border-t border-white/[0.03] ${r.valid ? "" : "bg-red-500/[0.04]"}`}>
                    <td className="px-2 py-1.5 text-slate-500">{r.rowNumber}</td>
                    <td className="px-2 py-1.5 font-mono text-slate-300 truncate max-w-[100px]">
                      {r.walletAddress.slice(0, 10)}…
                    </td>
                    <td className="px-2 py-1.5 text-slate-300">{r.name || "—"}</td>
                    <td className="px-2 py-1.5 text-right font-mono text-slate-300">
                      {r.allocationBps.toLocaleString()}
                    </td>
                    <td className="px-2 py-1.5 text-center">
                      {r.valid ? (
                        <CheckCircle size={12} className="text-emerald-400 mx-auto" />
                      ) : (
                        <XCircle size={12} className="text-red-400 mx-auto" />
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Validation summary */}
          {!totalValid && (
            <div className="mt-2 flex items-center gap-2 text-[11px] text-red-400">
              <AlertTriangle size={11} />
              Allocation must total exactly 10,000 bps (100.00%). Currently {totalBps.toLocaleString()} bps.
            </div>
          )}

          {/* Action buttons */}
          <div className="flex gap-2 mt-3">
            <button
              onClick={handleImport}
              disabled={!allValid || !totalValid}
              className="flex-1 py-2 rounded-lg bg-[#33C5E0] text-black text-xs font-semibold hover:bg-[#33C5E0]/90 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              data-testid="csv-import-btn"
            >
              Import {rows.filter((r) => r.valid).length} Beneficiaries
            </button>
            <button
              onClick={handleClear}
              className="px-4 py-2 rounded-lg bg-white/5 text-slate-300 border border-white/10 text-xs font-medium hover:bg-white/10 transition-colors"
            >
              Clear
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
