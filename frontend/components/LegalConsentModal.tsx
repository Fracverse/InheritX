"use client";

import React, { useState, useEffect, useCallback } from "react";
import {
  Shield,
  AlertTriangle,
  Check,
  ExternalLink,
  Globe,
} from "lucide-react";

const CONSENT_STORAGE_KEY = "inheritx_legal_consent_v1";
const CONSENT_VERSION = "1.0";

interface ConsentData {
  version: string;
  timestamp: number;
  hash: string;
  jurisdictionsAccepted: boolean;
  tosAccepted: boolean;
}

async function computeConsentHash(data: string): Promise<string> {
  const encoder = new TextEncoder();
  const buffer = encoder.encode(data);
  const hashBuffer = await crypto.subtle.digest("SHA-256", buffer as unknown as BufferSource);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
}

export function LegalConsentModal() {
  const [tosChecked, setTosChecked] = useState(false);
  const [jurisdictionChecked, setJurisdictionChecked] = useState(false);
  const [dismissed, setDismissed] = useState(true);

  useEffect(() => {
    try {
      const stored = localStorage.getItem(CONSENT_STORAGE_KEY);
      if (!stored) {
        setDismissed(false);
      } else {
        const data = JSON.parse(stored) as ConsentData;
        if (data.version !== CONSENT_VERSION) {
          setDismissed(false);
        }
      }
    } catch {
      setDismissed(false);
    }
  }, []);

  const canAccept = tosChecked && jurisdictionChecked;

  const handleAccept = useCallback(async () => {
    if (!canAccept) return;

    const payload = JSON.stringify({
      version: CONSENT_VERSION,
      timestamp: Date.now(),
      tosAccepted: true,
      jurisdictionsAccepted: true,
    });
    const hash = await computeConsentHash(payload);

    const consentData: ConsentData = {
      version: CONSENT_VERSION,
      timestamp: Date.now(),
      hash,
      jurisdictionsAccepted: true,
      tosAccepted: true,
    };

    try {
      localStorage.setItem(CONSENT_STORAGE_KEY, JSON.stringify(consentData));
    } catch {
      // storage unavailable
    }

    setDismissed(true);
  }, [canAccept]);

  if (dismissed) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-[60] bg-black/70 backdrop-blur-sm"
        aria-hidden="true"
      />

      {/* Modal */}
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="legal-consent-title"
        className="fixed inset-0 z-[70] flex items-center justify-center p-4"
        data-testid="legal-consent-modal"
      >
        <div className="bg-[#0A0F11] border border-white/[0.08] rounded-2xl p-6 max-w-lg w-full max-h-[90vh] overflow-y-auto shadow-2xl">
          {/* Header */}
          <div className="flex items-start justify-between mb-5">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-yellow-400/10 border border-yellow-400/20 flex items-center justify-center">
                <Shield size={18} className="text-yellow-400" />
              </div>
              <div>
                <h2 id="legal-consent-title" className="text-lg font-semibold text-white">
                  Legal Disclaimer
                </h2>
                <p className="text-xs text-slate-500 mt-0.5">
                  Please review and accept before continuing
                </p>
              </div>
            </div>
          </div>

          {/* Content */}
          <div className="space-y-4 mb-6">
            <div className="p-4 rounded-xl bg-yellow-400/[0.04] border border-yellow-400/15">
              <div className="flex items-start gap-2">
                <AlertTriangle size={14} className="text-yellow-400 flex-shrink-0 mt-0.5" />
                <div className="text-xs text-slate-300 leading-relaxed">
                  <p className="font-medium text-yellow-400 mb-1">Non-Custodial Smart Contract Risk</p>
                  <p>
                    InheritX operates non-custodial smart contracts on the Stellar network.
                    You retain full control of your assets at all times. However, smart contract
                    interactions carry inherent risks including but not limited to: smart contract
                    bugs, network congestion, and irreversible transactions.
                  </p>
                </div>
              </div>
            </div>

            <div className="p-4 rounded-xl bg-white/[0.02] border border-white/[0.05]">
              <div className="flex items-start gap-2">
                <Globe size={14} className="text-[#33C5E0] flex-shrink-0 mt-0.5" />
                <div className="text-xs text-slate-300 leading-relaxed">
                  <p className="font-medium text-white mb-1">Jurisdiction Notice</p>
                  <p>
                    InheritX does not provide services to users in prohibited jurisdictions.
                    By proceeding, you confirm that you are not located in, or a resident of,
                    any jurisdiction subject to comprehensive sanctions or where use of
                    decentralized finance protocols is prohibited by applicable law.
                  </p>
                </div>
              </div>
            </div>

            <div className="p-4 rounded-xl bg-white/[0.02] border border-white/[0.05]">
              <p className="text-xs text-slate-300 leading-relaxed">
                This platform is provided "as is" without warranties of any kind. InheritX
                is not a financial advisor. Yield projections are estimates only and not
                guaranteed. Please review our full{" "}
                <a
                  href="/Guidelines/terms-and-conditions"
                  className="text-[#33C5E0] hover:underline inline-flex items-center gap-0.5"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Terms of Service
                  <ExternalLink size={9} />
                </a>{" "}
                and{" "}
                <a
                  href="/privacy-policy"
                  className="text-[#33C5E0] hover:underline inline-flex items-center gap-0.5"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Privacy Policy
                  <ExternalLink size={9} />
                </a>
                .
              </p>
            </div>
          </div>

          {/* Checkboxes */}
          <div className="space-y-3 mb-6">
            <label className="flex items-start gap-3 cursor-pointer group">
              <div
                className={`w-5 h-5 rounded-md border flex items-center justify-center flex-shrink-0 mt-0.5 transition-colors ${
                  tosChecked
                    ? "bg-[#33C5E0] border-[#33C5E0]"
                    : "border-white/20 group-hover:border-white/40"
                }`}
              >
                {tosChecked && <Check size={12} className="text-black" strokeWidth={3} />}
              </div>
              <input
                type="checkbox"
                checked={tosChecked}
                onChange={(e) => setTosChecked(e.target.checked)}
                className="sr-only"
                aria-label="Accept Terms of Service"
              />
              <span className="text-xs text-slate-300 leading-relaxed">
                I have read and agree to the{" "}
                <a href="/Guidelines/terms-and-conditions" className="text-[#33C5E0] hover:underline" target="_blank" rel="noopener noreferrer">
                  Terms of Service
                </a>{" "}
                and understand the risks of non-custodial smart contract usage.
              </span>
            </label>

            <label className="flex items-start gap-3 cursor-pointer group">
              <div
                className={`w-5 h-5 rounded-md border flex items-center justify-center flex-shrink-0 mt-0.5 transition-colors ${
                  jurisdictionChecked
                    ? "bg-[#33C5E0] border-[#33C5E0]"
                    : "border-white/20 group-hover:border-white/40"
                }`}
              >
                {jurisdictionChecked && <Check size={12} className="text-black" strokeWidth={3} />}
              </div>
              <input
                type="checkbox"
                checked={jurisdictionChecked}
                onChange={(e) => setJurisdictionChecked(e.target.checked)}
                className="sr-only"
                aria-label="Confirm jurisdiction eligibility"
              />
              <span className="text-xs text-slate-300 leading-relaxed">
                I confirm that I am not located in, or a resident of, any sanctioned or
                prohibited jurisdiction, and that my use of this platform complies with
                applicable laws.
              </span>
            </label>
          </div>

          {/* Accept button */}
          <button
            onClick={handleAccept}
            disabled={!canAccept}
            className="w-full py-3 rounded-xl bg-[#33C5E0] text-black text-sm font-semibold hover:bg-[#33C5E0]/90 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            data-testid="accept-legal-btn"
          >
            Accept & Continue
          </button>

          <p className="text-[10px] text-slate-600 text-center mt-3">
            Your acknowledgement is stored locally in your browser for compliance purposes.
          </p>
        </div>
      </div>
    </>
  );
}
