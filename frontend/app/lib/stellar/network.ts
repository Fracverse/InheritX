import { Networks } from "@stellar/stellar-sdk";

/**
 * Soroban RPC endpoint used to simulate/build transactions from the browser.
 * Defaults to the public Testnet RPC node.
 */
export const SOROBAN_RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ?? "https://soroban-testnet.stellar.org";

/**
 * Stellar Horizon endpoint used to query account balances and ledger data.
 * Defaults to the public Testnet Horizon node.
 */
export const STELLAR_HORIZON_URL =
  process.env.NEXT_PUBLIC_STELLAR_HORIZON_URL ?? "https://horizon-testnet.stellar.org";

/**
 * Stellar Testnet Faucet URL used to fund accounts with testnet XLM for gas.
 */
export const STELLAR_FAUCET_URL =
  process.env.NEXT_PUBLIC_STELLAR_FAUCET_URL ??
  "https://laboratory.stellar.org/#account-creator?network=testnet";

/** Network passphrase the built transaction is signed against. */
export const STELLAR_NETWORK_PASSPHRASE =
  process.env.NEXT_PUBLIC_STELLAR_NETWORK_PASSPHRASE ?? Networks.TESTNET;

/**
 * Deployed `inheritance-contract` Soroban contract id (C... strkey). Read
 * lazily (rather than snapshotted into a module-level constant) so it can be
 * configured per-test via `process.env` without requiring a module reset.
 */
export function getInheritanceContractId(): string {
  return process.env.NEXT_PUBLIC_INHERITANCE_CONTRACT_ID ?? "";
}

/**
 * Deployed `lending-contract` Soroban contract id (C... strkey). Read lazily
 * for the same per-test configurability as `getInheritanceContractId`.
 */
export function getLendingContractId(): string {
  return process.env.NEXT_PUBLIC_LENDING_CONTRACT_ID ?? "";
}

/** Stellar Expert network segment used to build explorer links. */
const STELLAR_EXPERT_NETWORK =
  STELLAR_NETWORK_PASSPHRASE === Networks.PUBLIC ? "public" : "testnet";

/** Builds a Stellar Expert explorer URL for a submitted transaction hash. */
export function stellarExpertTxUrl(hash: string): string {
  return `https://stellar.expert/explorer/${STELLAR_EXPERT_NETWORK}/tx/${hash}`;
}

/** Number of decimal places assumed for Soroban token amounts (SEP-41 default). */
export const TOKEN_DECIMALS = 7;

/** Seconds a built transaction remains valid for before it must be rebuilt/re-signed. */
export const TRANSACTION_TIMEOUT_SECONDS = 180;
