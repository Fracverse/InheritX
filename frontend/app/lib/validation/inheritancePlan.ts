export interface DraftBeneficiary {
  address: string;
  name: string;
  allocationPercentage: number;
}

export interface InheritancePlanDraft {
  owner: string;
  tokenType: string;
  customTokenAddress: string;
  amount: string;
  earnYield: boolean;
  beneficiaries: DraftBeneficiary[];
  gracePeriodDays: number;
  timelockDays: number;
}

export interface ValidationResult {
  isValid: boolean;
  errors: Record<string, string>;
}

const TOKEN_ALIASES = new Set(["XLM", "USDC"]);
const BASE32_ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const STRKEY_LENGTH = 56;
const STRKEY_DECODED_LENGTH = 35;
const STRKEY_PAYLOAD_LENGTH = 32;
const ED25519_PUBLIC_KEY_VERSION_BYTE = 6 << 3;
const CONTRACT_VERSION_BYTE = 2 << 3;

function decodeBase32(value: string): number[] | null {
  let bits = 0;
  let buffer = 0;
  const decoded: number[] = [];

  for (const char of value) {
    const index = BASE32_ALPHABET.indexOf(char);
    if (index === -1) return null;

    buffer = (buffer << 5) | index;
    bits += 5;

    if (bits >= 8) {
      bits -= 8;
      decoded.push((buffer >> bits) & 0xff);
    }
  }

  return bits === 0 ? decoded : null;
}

function calculateCrc16Xmodem(bytes: number[]): number {
  let crc = 0;

  bytes.forEach((byte) => {
    crc ^= byte << 8;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = crc & 0x8000 ? (crc << 1) ^ 0x1021 : crc << 1;
      crc &= 0xffff;
    }
  });

  return crc;
}

function isValidStrKey(value: string, versionByte: number): boolean {
  const candidate = value.trim();
  if (candidate.length !== STRKEY_LENGTH) return false;

  const decoded = decodeBase32(candidate);
  if (!decoded || decoded.length !== STRKEY_DECODED_LENGTH) return false;
  if (decoded[0] !== versionByte) return false;

  const payload = decoded.slice(0, STRKEY_PAYLOAD_LENGTH + 1);
  const checksum = decoded.slice(STRKEY_PAYLOAD_LENGTH + 1);
  const expected = calculateCrc16Xmodem(payload);

  return checksum[0] === (expected & 0xff) && checksum[1] === (expected >> 8);
}

export function isValidStellarAccount(address: string): boolean {
  return isValidStrKey(address, ED25519_PUBLIC_KEY_VERSION_BYTE);
}

export function isValidTokenIdentifier(tokenType: string, customTokenAddress = ""): boolean {
  const token = tokenType.trim().toUpperCase();
  return TOKEN_ALIASES.has(token) || isValidStrKey(customTokenAddress, CONTRACT_VERSION_BYTE);
}

export function getSelectedTokenIdentifier(tokenType: string, customTokenAddress: string): string {
  const token = tokenType.trim().toUpperCase();
  return token === "CUSTOM" ? customTokenAddress.trim() : token;
}

export function percentageToBasisPoints(percentage: number): number {
  return Math.round(percentage * 100);
}

export function validateInheritancePlanDraft(draft: InheritancePlanDraft): ValidationResult {
  const errors: Record<string, string> = {};
  const amount = Number(draft.amount);

  if (!isValidStellarAccount(draft.owner)) {
    errors.owner = "Connect a valid Stellar wallet before creating a plan.";
  }

  if (!isValidTokenIdentifier(draft.tokenType, draft.customTokenAddress)) {
    errors.token = "Choose XLM, USDC, or enter a valid custom Stellar contract address.";
  }

  if (!Number.isFinite(amount) || amount <= 0) {
    errors.amount = "Enter an amount greater than zero.";
  }

  if (!Number.isInteger(draft.gracePeriodDays) || draft.gracePeriodDays < 1) {
    errors.gracePeriodDays = "Grace period must be at least 1 day.";
  }

  if (!Number.isInteger(draft.timelockDays) || draft.timelockDays < 0) {
    errors.timelockDays = "Timelock must be zero or more days.";
  }

  if (draft.beneficiaries.length === 0) {
    errors.beneficiaries = "Add at least one beneficiary.";
  }

  let allocationTotal = 0;
  const seenAddresses = new Set<string>();

  draft.beneficiaries.forEach((beneficiary, index) => {
    const row = index + 1;
    const address = beneficiary.address.trim();
    const allocation = Number(beneficiary.allocationPercentage);

    if (!beneficiary.name.trim()) {
      errors[`beneficiary.${index}.name`] = `Beneficiary ${row} needs a name.`;
    }

    if (!isValidStellarAccount(address)) {
      errors[`beneficiary.${index}.address`] = `Beneficiary ${row} needs a valid Stellar address.`;
    } else if (seenAddresses.has(address)) {
      errors[`beneficiary.${index}.address`] = `Beneficiary ${row} uses a duplicate address.`;
    }

    if (!Number.isFinite(allocation) || allocation <= 0 || allocation > 100) {
      errors[`beneficiary.${index}.allocationPercentage`] =
        `Beneficiary ${row} allocation must be between 1 and 100%.`;
    }

    allocationTotal += Number.isFinite(allocation) ? allocation : 0;
    seenAddresses.add(address);
  });

  if (Math.round(allocationTotal * 100) !== 10000) {
    errors.allocationTotal = "Beneficiary allocations must add up to exactly 100%.";
  }

  return {
    isValid: Object.keys(errors).length === 0,
    errors,
  };
}
