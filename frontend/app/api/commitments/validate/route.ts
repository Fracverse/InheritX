import { NextRequest, NextResponse } from "next/server";

export interface CommitmentDraft {
  title?: string;
  net_amount?: number | string;
  currency_preference?: string;
  beneficiary_name?: string;
  bank_account_number?: string;
  bank_name?: string;
  inactivity_days?: number | string;
}

export interface FieldError {
  field: string;
  message: string;
}

export interface ValidateResponse {
  valid: boolean;
  errors: FieldError[];
}

const SUPPORTED_CURRENCIES = ["XLM", "USDC", "EURC", "NGN", "KES", "BRL", "PHP", "EUR", "USD"];

export async function POST(req: NextRequest): Promise<NextResponse<ValidateResponse>> {
  let body: CommitmentDraft;
  try {
    body = await req.json();
  } catch {
    return NextResponse.json({ valid: false, errors: [{ field: "_", message: "Invalid JSON body" }] }, { status: 400 });
  }

  const errors: FieldError[] = [];

  if (!body.title || String(body.title).trim().length < 3) {
    errors.push({ field: "title", message: "Title must be at least 3 characters." });
  }

  const amount = Number(body.net_amount);
  if (!body.net_amount || isNaN(amount) || amount <= 0) {
    errors.push({ field: "net_amount", message: "Amount must be a positive number." });
  }

  if (!body.currency_preference || !SUPPORTED_CURRENCIES.includes(String(body.currency_preference).toUpperCase())) {
    errors.push({ field: "currency_preference", message: `Currency must be one of: ${SUPPORTED_CURRENCIES.join(", ")}.` });
  }

  if (!body.beneficiary_name || String(body.beneficiary_name).trim().length < 2) {
    errors.push({ field: "beneficiary_name", message: "Beneficiary name must be at least 2 characters." });
  }

  if (!body.bank_account_number || !/^\d{6,20}$/.test(String(body.bank_account_number).trim())) {
    errors.push({ field: "bank_account_number", message: "Bank account number must be 6–20 digits." });
  }

  if (!body.bank_name || String(body.bank_name).trim().length < 2) {
    errors.push({ field: "bank_name", message: "Bank name must be at least 2 characters." });
  }

  const days = Number(body.inactivity_days);
  if (body.inactivity_days !== undefined && body.inactivity_days !== "") {
    if (isNaN(days) || days < 30 || days > 3650) {
      errors.push({ field: "inactivity_days", message: "Inactivity period must be between 30 and 3650 days." });
    }
  }

  return NextResponse.json({ valid: errors.length === 0, errors });
}
