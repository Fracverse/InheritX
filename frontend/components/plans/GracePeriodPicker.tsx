"use client";

import React, { useState, useMemo, useCallback } from "react";
import { Clock, Calendar, AlertTriangle, Info, Minus, Plus } from "lucide-react";

const PRESET_DAYS = [30, 90, 180, 365] as const;
const MIN_DAYS = 1;
const MAX_DAYS = 3650;
const SECONDS_PER_DAY = 86400;

interface GracePeriodPickerProps {
  value: number;
  onChange: (days: number) => void;
  min?: number;
  max?: number;
  disabled?: boolean;
}

function formatDateFromNow(days: number): string {
  const target = new Date(Date.now() + days * SECONDS_PER_DAY * 1000);
  return target.toLocaleDateString("en-US", {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatDuration(days: number): string {
  if (days < 30) return `${days} day${days !== 1 ? "s" : ""}`;
  if (days < 365) {
    const months = Math.floor(days / 30);
    const remainingDays = days % 30;
    return remainingDays > 0
      ? `${months}mo ${remainingDays}d`
      : `${months} month${months !== 1 ? "s" : ""}`;
  }
  const years = Math.floor(days / 365);
  const remainingMonths = Math.floor((days % 365) / 30);
  return remainingMonths > 0
    ? `${years}y ${remainingMonths}mo`
    : `${years} year${years !== 1 ? "s" : ""}`;
}

function formatSeconds(days: number): string {
  const seconds = days * SECONDS_PER_DAY;
  return `${seconds.toLocaleString()} seconds`;
}

export function GracePeriodPicker({
  value,
  onChange,
  min = MIN_DAYS,
  max = MAX_DAYS,
  disabled = false,
}: GracePeriodPickerProps) {
  const [customMode, setCustomMode] = useState(false);
  const [inputValue, setInputValue] = useState(String(value));
  const [inputError, setInputError] = useState<string | null>(null);

  const validateInput = useCallback(
    (val: string): string | null => {
      const num = Number(val);
      if (val.trim() === "") return "Required";
      if (isNaN(num)) return "Must be a number";
      if (!Number.isInteger(num)) return "Must be a whole number";
      if (num < min) return `Minimum ${min} days`;
      if (num > max) return `Maximum ${max.toLocaleString()} days`;
      return null;
    },
    [min, max]
  );

  const handleInputChange = useCallback(
    (val: string) => {
      setInputValue(val);
      const err = validateInput(val);
      setInputError(err);
      if (!err) {
        onChange(Number(val));
      }
    },
    [validateInput, onChange]
  );

  const handlePreset = useCallback(
    (days: number) => {
      setCustomMode(false);
      setInputValue(String(days));
      setInputError(null);
      onChange(days);
    },
    [onChange]
  );

  const handleStepper = useCallback(
    (delta: number) => {
      const next = Math.max(min, Math.min(max, value + delta));
      onChange(next);
      setInputValue(String(next));
      setInputError(null);
      setCustomMode(true);
    },
    [value, min, max, onChange]
  );

  const handleEnableCustom = useCallback(() => {
    setCustomMode(true);
    setInputValue(String(value));
  }, [value]);

  const expiryPreview = useMemo(() => formatDateFromNow(value), [value]);

  return (
    <div className="space-y-4" data-testid="grace-period-picker">
      {/* Preset buttons */}
      <div>
        <label className="block text-xs text-slate-400 mb-2">
          Preset durations
        </label>
        <div className="grid grid-cols-4 gap-2" role="radiogroup" aria-label="Grace period presets">
          {PRESET_DAYS.map((days) => (
            <button
              key={days}
              type="button"
              role="radio"
              aria-checked={!customMode && value === days}
              onClick={() => handlePreset(days)}
              disabled={disabled}
              className={`py-2.5 rounded-xl text-xs font-medium transition-colors border ${
                !customMode && value === days
                  ? "bg-[#33C5E0]/10 text-[#33C5E0] border-[#33C5E0]/30"
                  : "bg-white/[0.03] text-slate-400 border-white/[0.06] hover:bg-white/[0.06] hover:text-white"
              } disabled:opacity-40`}
            >
              {formatDuration(days)}
            </button>
          ))}
        </div>
      </div>

      {/* Custom input */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <label htmlFor="grace-days" className="text-xs text-slate-400">
            Custom duration (days)
          </label>
          {!customMode && (
            <button
              type="button"
              onClick={handleEnableCustom}
              disabled={disabled}
              className="text-[11px] text-[#33C5E0] hover:underline disabled:opacity-40"
            >
              Customize
            </button>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => handleStepper(-1)}
            disabled={disabled || value <= min}
            className="p-2.5 rounded-lg bg-white/[0.04] border border-white/[0.06] text-slate-400 hover:text-white hover:bg-white/[0.08] transition-colors disabled:opacity-30"
            aria-label="Decrease by 1 day"
            style={{ minWidth: 44, minHeight: 44 }}
          >
            <Minus size={14} />
          </button>

          <div className="relative flex-1">
            <input
              id="grace-days"
              type="number"
              inputMode="numeric"
              value={inputValue}
              onChange={(e) => handleInputChange(e.target.value)}
              disabled={disabled}
              min={min}
              max={max}
              className={`w-full px-3 py-2.5 rounded-xl bg-white/5 border text-sm text-white text-center font-mono focus:outline-none transition-colors ${
                inputError
                  ? "border-red-500/40 focus:border-red-500/60"
                  : "border-white/10 focus:border-[#33C5E0]/40"
              } disabled:opacity-40`}
              aria-invalid={!!inputError}
              aria-describedby={inputError ? "grace-error" : "grace-preview"}
            />
            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-slate-500">
              days
            </span>
          </div>

          <button
            type="button"
            onClick={() => handleStepper(1)}
            disabled={disabled || value >= max}
            className="p-2.5 rounded-lg bg-white/[0.04] border border-white/[0.06] text-slate-400 hover:text-white hover:bg-white/[0.08] transition-colors disabled:opacity-30"
            aria-label="Increase by 1 day"
            style={{ minWidth: 44, minHeight: 44 }}
          >
            <Plus size={14} />
          </button>
        </div>

        {inputError && (
          <p id="grace-error" className="text-[11px] text-red-400 mt-1.5 flex items-center gap-1">
            <AlertTriangle size={10} />
            {inputError}
          </p>
        )}
      </div>

      {/* Preview */}
      <div
        id="grace-preview"
        className="p-3.5 rounded-xl bg-[#33C5E0]/[0.04] border border-[#33C5E0]/10 space-y-2"
      >
        <div className="flex items-center gap-2 text-xs">
          <Clock size={12} className="text-[#33C5E0] flex-shrink-0" />
          <span className="text-slate-400">Duration:</span>
          <span className="text-white font-medium">{formatDuration(value)}</span>
          <span className="text-slate-600">({formatSeconds(value)})</span>
        </div>
        <div className="flex items-start gap-2 text-xs">
          <Calendar size={12} className="text-[#33C5E0] flex-shrink-0 mt-0.5" />
          <div>
            <span className="text-slate-400">Expires: </span>
            <span className="text-white">{expiryPreview}</span>
          </div>
        </div>
        <div className="flex items-start gap-2 text-[11px] pt-1 border-t border-[#33C5E0]/10">
          <Info size={11} className="text-slate-500 flex-shrink-0 mt-0.5" />
          <p className="text-slate-500 leading-relaxed">
            If you don&apos;t ping before this date, your plan will be triggerable by beneficiaries.
            The grace period begins from your last ping.
          </p>
        </div>
      </div>
    </div>
  );
}
