import { Buffer } from "buffer";
import { Address, nativeToScVal, xdr } from "@stellar/stellar-sdk";

/**
 * Encodes JS values into the exact Soroban `ScVal` shapes expected by
 * `inheritance-contract`'s `create_inheritance_plan` entrypoint
 * (contracts/inheritance-contract/src/lib.rs). These helpers intentionally
 * avoid `nativeToScVal`'s generic object/array inference for structs, enums
 * and tuples — the contract's `#[contracttype]` derive encodes:
 *   - structs as an `ScVal::Map` whose Symbol keys are sorted alphabetically
 *   - fieldless enum variants as `ScVal::Vec([Symbol(variant_name)])`
 *   - tuples as a positional `ScVal::Vec`
 * so each shape is built explicitly here to match byte-for-byte.
 */

export type DistributionMethod = "LumpSum" | "Monthly" | "Quarterly" | "Yearly";

export const DISTRIBUTION_METHODS: DistributionMethod[] = [
  "LumpSum",
  "Monthly",
  "Quarterly",
  "Yearly",
];

/** Mirrors the contract's `(full_name, email, claim_code, bank_account, allocation_bp, priority)` tuple. */
export interface ContractBeneficiaryInput {
  fullName: string;
  email: string;
  /** 6-digit numeric claim code (0-999999) the contract hashes with a salt. */
  claimCode: number;
  /** Plain-text settlement/bank reference (contract requires non-empty bytes). */
  bankAccount: string;
  /** Allocation in basis points; all beneficiaries must sum to 10000. */
  allocationBp: number;
  /** 1-indexed claim priority; must be unique across beneficiaries. */
  priority: number;
}

/** Mirrors `CreateInheritancePlanParams` in contracts/inheritance-contract/src/lib.rs. */
export interface CreateInheritancePlanParamsInput {
  owner: string;
  token: string;
  planName: string;
  description: string;
  /** Amount in the token's smallest unit (atomic/stroop-equivalent, u64). */
  totalAmountAtomic: bigint;
  distributionMethod: DistributionMethod;
  beneficiaries: ContractBeneficiaryInput[];
  isLendable: boolean;
}

function scvString(value: string): xdr.ScVal {
  return xdr.ScVal.scvString(value);
}

function scvU32(value: number): xdr.ScVal {
  if (!Number.isInteger(value) || value < 0 || value > 0xffffffff) {
    throw new RangeError(`value out of u32 range: ${value}`);
  }
  return xdr.ScVal.scvU32(value);
}

const U64_MAX = BigInt("0xffffffffffffffff");

function scvU64(value: bigint): xdr.ScVal {
  if (value < BigInt(0) || value > U64_MAX) {
    throw new RangeError(`value out of u64 range: ${value}`);
  }
  return nativeToScVal(value, { type: "u64" });
}

function scvBytesFromUtf8(value: string): xdr.ScVal {
  return xdr.ScVal.scvBytes(Buffer.from(value, "utf-8"));
}

function scvAddress(strkey: string): xdr.ScVal {
  return Address.fromString(strkey).toScVal();
}

/** Encodes a fieldless enum variant the way soroban-sdk's union-style `#[contracttype]` does. */
function scvUnitEnum(variant: string): xdr.ScVal {
  return xdr.ScVal.scvVec([xdr.ScVal.scvSymbol(variant)]);
}

export function encodeDistributionMethod(method: DistributionMethod): xdr.ScVal {
  return scvUnitEnum(method);
}

export function encodeBeneficiaryTuple(beneficiary: ContractBeneficiaryInput): xdr.ScVal {
  return xdr.ScVal.scvVec([
    scvString(beneficiary.fullName),
    scvString(beneficiary.email),
    scvU32(beneficiary.claimCode),
    scvBytesFromUtf8(beneficiary.bankAccount),
    scvU32(beneficiary.allocationBp),
    scvU32(beneficiary.priority),
  ]);
}

function mapEntry(key: string, val: xdr.ScVal): xdr.ScMapEntry {
  return new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol(key), val });
}

/**
 * Builds the single `CreateInheritancePlanParams` argument ScVal for
 * `create_inheritance_plan`. Map entries MUST be sorted alphabetically by
 * field name — that's the canonical ordering the Soroban host requires for
 * struct maps, and how the contract's generated deserializer looks them up.
 */
export function encodeCreateInheritancePlanParams(
  input: CreateInheritancePlanParamsInput
): xdr.ScVal {
  const beneficiariesVec = xdr.ScVal.scvVec(
    input.beneficiaries.map((b) => encodeBeneficiaryTuple(b))
  );

  const entries = [
    mapEntry("beneficiaries_data", beneficiariesVec),
    mapEntry("description", scvString(input.description)),
    mapEntry("distribution_method", encodeDistributionMethod(input.distributionMethod)),
    mapEntry("is_lendable", xdr.ScVal.scvBool(input.isLendable)),
    mapEntry("owner", scvAddress(input.owner)),
    mapEntry("plan_name", scvString(input.planName)),
    mapEntry("token", scvAddress(input.token)),
    mapEntry("total_amount", scvU64(input.totalAmountAtomic)),
  ];

  return xdr.ScVal.scvMap(entries);
}

/** Converts a user-entered decimal amount string into atomic units (default 7 decimals, SEP-41). */
export function toAtomicAmount(amount: string | number, decimals = 7): bigint {
  const normalized = typeof amount === "number" ? amount.toString() : amount.trim();
  if (!/^\d+(\.\d+)?$/.test(normalized)) {
    throw new RangeError(`invalid decimal amount: ${amount}`);
  }

  const [whole, fraction = ""] = normalized.split(".");
  const paddedFraction = (fraction + "0".repeat(decimals)).slice(0, decimals);
  const atomic =
    BigInt(whole || "0") * BigInt(10) ** BigInt(decimals) + BigInt(paddedFraction || "0");
  return atomic;
}
