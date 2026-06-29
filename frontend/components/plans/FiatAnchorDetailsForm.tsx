"use client";

import { useEffect, useMemo, useState } from "react";

interface FiatAnchorInfo {
  currency: string;
  anchor_provider: string;
  country: string;
  bank_name?: string;
  iban?: string;
  routing_number?: string;
  account_number?: string;
  accept_fees: boolean;
}

interface Props {
  value?: string;
  onChange: (value: string) => void;
}

const CURRENCY_OPTIONS = [
  { value: "USD", label: "USD" },
  { value: "EUR", label: "EUR" },
  { value: "NGN", label: "NGN" },
];

const COUNTRY_OPTIONS: Record<string, Array<{ value: string; label: string }>> = {
  USD: [{ value: "USA", label: "USA" }],
  EUR: [{ value: "Germany", label: "Germany" }],
  NGN: [{ value: "Nigeria", label: "Nigeria" }],
};

const ANCHOR_PROVIDERS: Record<string, Array<{ value: string; label: string }>> = {
  USA: [
    { value: "Circle", label: "Circle" },
    { value: "Stripe", label: "Stripe" },
  ],
  Germany: [
    { value: "Circle", label: "Circle" },
    { value: "BankTransfer", label: "Bank Transfer" },
  ],
  Nigeria: [
    { value: "Flutterwave", label: "Flutterwave" },
    { value: "Yellow Card", label: "Yellow Card" },
  ],
};

const DEFAULT_DETAILS: FiatAnchorInfo = {
  currency: "USD",
  anchor_provider: "",
  country: "",
  bank_name: "",
  iban: "",
  routing_number: "",
  account_number: "",
  accept_fees: false,
};

function parseFiatAnchorInfo(value?: string): FiatAnchorInfo {
  if (!value) return DEFAULT_DETAILS;

  try {
    const parsed = JSON.parse(value) as FiatAnchorInfo;
    return {
      ...DEFAULT_DETAILS,
      ...parsed,
    };
  } catch {
    return DEFAULT_DETAILS;
  }
}

function cleanDetails(details: FiatAnchorInfo): FiatAnchorInfo {
  return {
    ...details,
    bank_name: details.bank_name ?? "",
    iban: details.iban ?? "",
    routing_number: details.routing_number ?? "",
    account_number: details.account_number ?? "",
  };
}

function isValidRoutingNumber(value: string): boolean {
  return /^[0-9]{9}$/.test(value.trim());
}

function isValidNigerianAccountNumber(value: string): boolean {
  return /^[0-9]{10}$/.test(value.trim());
}

function isValidGermanIban(value: string): boolean {
  const sanitized = value.replace(/\s+/g, "");
  return /^DE[0-9]{20}$/.test(sanitized);
}

export default function FiatAnchorDetailsForm({
  value,
  onChange,
}: Props) {
  const [details, setDetails] = useState<FiatAnchorInfo>(() => parseFiatAnchorInfo(value));

  useEffect(() => {
    if (value) {
      setDetails(parseFiatAnchorInfo(value));
    }
  }, [value]);

  const availableCountries = COUNTRY_OPTIONS[details.currency] ?? [];
  const availableProviders = details.country ? ANCHOR_PROVIDERS[details.country] ?? [] : [];

  const validation = useMemo(() => {
    return {
      routing_number:
        details.country === "USA" && details.routing_number
          ? isValidRoutingNumber(details.routing_number)
            ? ""
            : "Routing number must be 9 digits."
          : "",
      account_number:
        details.country === "Nigeria" && details.account_number
          ? isValidNigerianAccountNumber(details.account_number)
            ? ""
            : "Account number must be 10 digits."
          : "",
      iban:
        details.country === "Germany" && details.iban
          ? isValidGermanIban(details.iban)
            ? ""
            : "Enter a valid German IBAN starting with DE."
          : "",
    };
  }, [details]);

  function handleChange(field: keyof FiatAnchorInfo, value: string | boolean) {
    const updated = cleanDetails({
      ...details,
      [field]: value,
    });

    setDetails(updated);
    onChange(JSON.stringify(updated));
  }

  function handleCurrencyChange(currency: string) {
    const allowedCountries = COUNTRY_OPTIONS[currency].map((item) => item.value);
    const selectedCountry = allowedCountries.includes(details.country)
      ? details.country
      : "";

    const updated: FiatAnchorInfo = {
      ...DEFAULT_DETAILS,
      ...details,
      currency,
      country: selectedCountry,
      anchor_provider: selectedCountry ? details.anchor_provider : "",
    };

    setDetails(updated);
    onChange(JSON.stringify(updated));
  }

  function handleCountryChange(country: string) {
    const updated: FiatAnchorInfo = {
      ...details,
      country,
      anchor_provider: "",
      bank_name: "",
      account_number: "",
      routing_number: "",
      iban: "",
    };

    setDetails(updated);
    onChange(JSON.stringify(updated));
  }

  function handleProviderChange(anchor_provider: string) {
    const updated: FiatAnchorInfo = {
      ...details,
      anchor_provider,
      bank_name: "",
      account_number: "",
      routing_number: "",
      iban: "",
    };

    setDetails(updated);
    onChange(JSON.stringify(updated));
  }

  return (
    <div className="space-y-4 rounded-lg border p-4">
      <h3 className="text-lg font-semibold">Fiat Off-Ramp Details</h3>

      <div className="grid gap-4 md:grid-cols-3">
        <div>
          <label className="block text-sm font-medium text-slate-200 mb-1">
            Currency
          </label>
          <select
            className="w-full rounded border border-slate-700 bg-slate-950 p-2 text-sm text-slate-200"
            value={details.currency}
            onChange={(e) => handleCurrencyChange(e.target.value)}
          >
            {CURRENCY_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-200 mb-1">
            Country
          </label>
          <select
            className="w-full rounded border border-slate-700 bg-slate-950 p-2 text-sm text-slate-200"
            value={details.country}
            onChange={(e) => handleCountryChange(e.target.value)}
          >
            <option value="">Select country</option>
            {availableCountries.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-200 mb-1">
            Anchor Provider
          </label>
          <select
            className="w-full rounded border border-slate-700 bg-slate-950 p-2 text-sm text-slate-200"
            value={details.anchor_provider}
            onChange={(e) => handleProviderChange(e.target.value)}
            disabled={!details.country}
          >
            <option value="">Select provider</option>
            {availableProviders.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {details.country === "Nigeria" && (
        <div className="space-y-3">
          <div>
            <label className="block text-sm font-medium text-slate-200 mb-1">
              Bank Name
            </label>
            <input
              className="w-full rounded border border-slate-700 bg-slate-950 p-2 text-sm text-slate-200"
              placeholder="Bank Name"
              value={details.bank_name}
              onChange={(e) => handleChange("bank_name", e.target.value)}
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-200 mb-1">
              Account Number
            </label>
            <input
              className="w-full rounded border border-slate-700 bg-slate-950 p-2 text-sm text-slate-200"
              placeholder="Account Number"
              value={details.account_number}
              onChange={(e) => handleChange("account_number", e.target.value)}
              aria-invalid={Boolean(validation.account_number)}
            />
            {validation.account_number ? (
              <p className="mt-1 text-xs text-red-400">
                {validation.account_number}
              </p>
            ) : null}
          </div>
        </div>
      )}

      {details.country === "USA" && (
        <div className="space-y-3">
          <div>
            <label className="block text-sm font-medium text-slate-200 mb-1">
              Account Number
            </label>
            <input
              className="w-full rounded border border-slate-700 bg-slate-950 p-2 text-sm text-slate-200"
              placeholder="Account Number"
              value={details.account_number}
              onChange={(e) => handleChange("account_number", e.target.value)}
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-200 mb-1">
              Routing Number
            </label>
            <input
              className="w-full rounded border border-slate-700 bg-slate-950 p-2 text-sm text-slate-200"
              placeholder="Routing Number"
              value={details.routing_number}
              onChange={(e) => handleChange("routing_number", e.target.value)}
              aria-invalid={Boolean(validation.routing_number)}
            />
            {validation.routing_number ? (
              <p className="mt-1 text-xs text-red-400">
                {validation.routing_number}
              </p>
            ) : null}
          </div>
        </div>
      )}

      {details.country === "Germany" && (
        <div className="space-y-3">
          <div>
            <label className="block text-sm font-medium text-slate-200 mb-1">
              IBAN
            </label>
            <input
              className="w-full rounded border border-slate-700 bg-slate-950 p-2 text-sm text-slate-200"
              placeholder="IBAN"
              value={details.iban}
              onChange={(e) => handleChange("iban", e.target.value)}
              aria-invalid={Boolean(validation.iban)}
            />
            {validation.iban ? (
              <p className="mt-1 text-xs text-red-400">
                {validation.iban}
              </p>
            ) : null}
          </div>
        </div>
      )}

      <label className="flex items-center gap-2 text-sm text-slate-200">
        <input
          type="checkbox"
          checked={details.accept_fees}
          onChange={(e) => handleChange("accept_fees", e.target.checked)}
          className="h-4 w-4 rounded border-slate-700 bg-slate-950 text-cyan-400"
        />
        <span>I accept fiat anchor fees</span>
      </label>

      {!details.accept_fees ? (
        <p className="text-xs text-slate-400">
          Beneficiaries must accept fiat anchor fees before payout metadata can be used.
        </p>
      ) : null}
    </div>
  );
}
