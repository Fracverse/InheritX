"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type { CommitmentDraft, FieldError } from "@/app/api/commitments/validate/route";

export interface ConfigureFormValues {
  title: string;
  net_amount: string;
  currency_preference: string;
  beneficiary_name: string;
  bank_account_number: string;
  bank_name: string;
  inactivity_days: string;
}

interface Props {
  initialValues?: Partial<ConfigureFormValues>;
  onAdvance: (values: ConfigureFormValues) => void;
}

const SUPPORTED_CURRENCIES = ["XLM", "USDC", "EURC", "NGN", "KES", "BRL", "PHP", "EUR", "USD"];
const DEBOUNCE_MS = 500;

const EMPTY: ConfigureFormValues = {
  title: "",
  net_amount: "",
  currency_preference: "",
  beneficiary_name: "",
  bank_account_number: "",
  bank_name: "",
  inactivity_days: "",
};

export function CreateCommitmentStepConfigure({ initialValues, onAdvance }: Props) {
  const [values, setValues] = useState<ConfigureFormValues>({ ...EMPTY, ...initialValues });
  const [serverErrors, setServerErrors] = useState<Record<string, string>>({});
  const [validating, setValidating] = useState(false);
  const [serverDown, setServerDown] = useState(false);
  const abortRef = useRef<AbortController | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const validate = useCallback(async (draft: ConfigureFormValues) => {
    if (abortRef.current) abortRef.current.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    setValidating(true);
    setServerDown(false);

    try {
      const res = await fetch("/api/commitments/validate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          ...draft,
          net_amount: draft.net_amount === "" ? undefined : Number(draft.net_amount),
          inactivity_days: draft.inactivity_days === "" ? undefined : Number(draft.inactivity_days),
        } satisfies CommitmentDraft),
        signal: controller.signal,
      });

      const data = await res.json();
      const map: Record<string, string> = {};
      for (const e of (data.errors ?? []) as FieldError[]) {
        map[e.field] = e.message;
      }
      setServerErrors(map);
    } catch (err) {
      if ((err as Error).name !== "AbortError") {
        setServerDown(true);
      }
    } finally {
      if (!controller.signal.aborted) setValidating(false);
    }
  }, []);

  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => validate(values), DEBOUNCE_MS);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [values, validate]);

  // cancel in-flight request on unmount
  useEffect(() => () => { abortRef.current?.abort(); }, []);

  const handleChange = (field: keyof ConfigureFormValues) =>
    (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) =>
      setValues((v) => ({ ...v, [field]: e.target.value }));

  const isBlocked = validating || Object.keys(serverErrors).length > 0;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!isBlocked) onAdvance(values);
  };

  return (
    <form onSubmit={handleSubmit} noValidate aria-label="Configure commitment">
      <Field id="title" label="Title" error={serverErrors.title}>
        <input
          id="title"
          value={values.title}
          onChange={handleChange("title")}
          aria-invalid={!!serverErrors.title}
          aria-describedby={serverErrors.title ? "title-error" : undefined}
        />
      </Field>

      <Field id="net_amount" label="Amount" error={serverErrors.net_amount}>
        <input
          id="net_amount"
          type="number"
          min="0"
          step="any"
          value={values.net_amount}
          onChange={handleChange("net_amount")}
          aria-invalid={!!serverErrors.net_amount}
          aria-describedby={serverErrors.net_amount ? "net_amount-error" : undefined}
        />
      </Field>

      <Field id="currency_preference" label="Currency" error={serverErrors.currency_preference}>
        <select
          id="currency_preference"
          value={values.currency_preference}
          onChange={handleChange("currency_preference")}
          aria-invalid={!!serverErrors.currency_preference}
          aria-describedby={serverErrors.currency_preference ? "currency_preference-error" : undefined}
        >
          <option value="">Select currency</option>
          {SUPPORTED_CURRENCIES.map((c) => (
            <option key={c} value={c}>{c}</option>
          ))}
        </select>
      </Field>

      <Field id="beneficiary_name" label="Beneficiary name" error={serverErrors.beneficiary_name}>
        <input
          id="beneficiary_name"
          value={values.beneficiary_name}
          onChange={handleChange("beneficiary_name")}
          aria-invalid={!!serverErrors.beneficiary_name}
          aria-describedby={serverErrors.beneficiary_name ? "beneficiary_name-error" : undefined}
        />
      </Field>

      <Field id="bank_account_number" label="Bank account number" error={serverErrors.bank_account_number}>
        <input
          id="bank_account_number"
          value={values.bank_account_number}
          onChange={handleChange("bank_account_number")}
          aria-invalid={!!serverErrors.bank_account_number}
          aria-describedby={serverErrors.bank_account_number ? "bank_account_number-error" : undefined}
        />
      </Field>

      <Field id="bank_name" label="Bank name" error={serverErrors.bank_name}>
        <input
          id="bank_name"
          value={values.bank_name}
          onChange={handleChange("bank_name")}
          aria-invalid={!!serverErrors.bank_name}
          aria-describedby={serverErrors.bank_name ? "bank_name-error" : undefined}
        />
      </Field>

      <Field id="inactivity_days" label="Inactivity days (30–3650, optional)" error={serverErrors.inactivity_days}>
        <input
          id="inactivity_days"
          type="number"
          min="30"
          max="3650"
          value={values.inactivity_days}
          onChange={handleChange("inactivity_days")}
          aria-invalid={!!serverErrors.inactivity_days}
          aria-describedby={serverErrors.inactivity_days ? "inactivity_days-error" : undefined}
        />
      </Field>

      {serverDown && (
        <p role="alert" className="text-yellow-600 text-sm">
          Validation service unavailable — you may still proceed.
        </p>
      )}

      <button
        type="submit"
        disabled={isBlocked && !serverDown}
        aria-busy={validating}
      >
        {validating ? "Validating…" : "Continue"}
      </button>
    </form>
  );
}

// ─── Tiny accessible field wrapper ───────────────────────────────────────────

function Field({
  id,
  label,
  error,
  children,
}: {
  id: string;
  label: string;
  error?: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <label htmlFor={id}>{label}</label>
      {children}
      {error && (
        <span id={`${id}-error`} role="alert" className="text-red-600 text-sm">
          {error}
        </span>
      )}
    </div>
  );
}
