"use client";

import { useState } from "react";
import {
  createGeneticVerificationAPI,
  PrivacySettings,
} from "@/app/lib/api/geneticVerification";

interface Props {
  planId: string;
  settings: PrivacySettings;
  onSettingsChange: (settings: PrivacySettings) => void;
}

// Self-contained toggle rather than importing the existing
// app/asset-owner/security/components/Toggle.tsx — that component's prop
// API wasn't available to review, so wiring it blind risked a broken build.
// Worth swapping in once confirmed to take a similar checked/onChange shape.
function SettingToggle({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-4 py-4">
      <div>
        <p className="text-sm font-medium text-white">{label}</p>
        <p className="text-xs text-[#92A5A8]">{description}</p>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative h-6 w-11 flex-shrink-0 rounded-full transition-colors ${
          checked ? "bg-[#33C5E0]" : "bg-[#2A3338]"
        }`}
      >
        <span
          className={`absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-white transition-transform ${
            checked ? "translate-x-5" : "translate-x-0"
          }`}
        />
      </button>
    </div>
  );
}

export default function PrivacyControls({
  planId,
  settings,
  onSettingsChange,
}: Props) {
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const api = createGeneticVerificationAPI();

  const update = (partial: Partial<PrivacySettings>) => {
    onSettingsChange({ ...settings, ...partial });
    setSaved(false);
  };

  const handleSave = async () => {
    if (!planId) {
      setError("Missing plan ID. Open this page from one of your plans.");
      return;
    }
    setSaving(true);
    setError(null);
    try {
      const updated = await api.updatePrivacySettings(planId, settings);
      onSettingsChange(updated);
      setSaved(true);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Could not save privacy settings.",
      );
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
      <h2 className="mb-1 text-xl font-semibold text-white">
        Privacy Controls
      </h2>
      <p className="mb-2 text-sm text-[#92A5A8]">
        Choose what your genetic verification data is used for and who can
        see it.
      </p>

      <div className="divide-y divide-[#1C252A]">
        <SettingToggle
          label="Share with family"
          description="Verified relatives on your family tree can see your verification status."
          checked={settings.shareWithFamily}
          onChange={(value) => update({ shareWithFamily: value })}
        />
        <SettingToggle
          label="Allow health analysis"
          description="Permit health-condition triggers to be evaluated against your data."
          checked={settings.allowHealthAnalysis}
          onChange={(value) => update({ allowHealthAnalysis: value })}
        />
        <SettingToggle
          label="Visible to relatives"
          description="Show your profile as a potential match when relatives search the family tree."
          checked={settings.visibleToRelatives}
          onChange={(value) => update({ visibleToRelatives: value })}
        />
      </div>

      <div className="mt-4">
        <label className="mb-2 block text-sm font-medium text-white">
          Data retention period
        </label>
        <select
          value={settings.dataRetentionDays}
          onChange={(e) => update({ dataRetentionDays: Number(e.target.value) })}
          className="w-full rounded-lg border border-[#2A3338] bg-transparent px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] focus:outline-none"
        >
          <option className="bg-[#1C252A]" value={90}>
            90 days
          </option>
          <option className="bg-[#1C252A]" value={365}>
            1 year
          </option>
          <option className="bg-[#1C252A]" value={1825}>
            5 years
          </option>
          <option className="bg-[#1C252A]" value={-1}>
            Indefinite
          </option>
        </select>
      </div>

      {error && (
        <div className="mt-4 rounded-lg border border-[#F56565]/30 bg-[#F56565]/10 p-3 text-sm text-[#F56565]">
          {error}
        </div>
      )}

      <button
        onClick={handleSave}
        disabled={saving}
        className="mt-6 rounded-lg bg-[#33C5E0] px-6 py-3 font-semibold text-[#0D1417] transition-colors hover:bg-[#2AB8D3] disabled:cursor-not-allowed disabled:opacity-50"
      >
        {saving ? "Saving…" : saved ? "Saved ✓" : "Save Privacy Settings"}
      </button>
    </div>
  );
}