import { Address, BASE_FEE, Contract, TransactionBuilder, rpc, scValToNative } from "@stellar/stellar-sdk";
import {
  encodeCreateInheritancePlanParams,
  type CreateInheritancePlanParamsInput,
} from "./contractParams";
import {
  getInheritanceContractId,
  SOROBAN_RPC_URL,
  STELLAR_NETWORK_PASSPHRASE,
  TRANSACTION_TIMEOUT_SECONDS,
} from "./network";

/** Maps `InheritanceError` discriminants (contracts/inheritance-contract/src/lib.rs) to user-facing text. */
const INHERITANCE_ERROR_MESSAGES: Record<number, string> = {
  1: "That token isn't supported by the contract.",
  2: "The deposit amount is invalid.",
  3: "A required field is missing.",
  4: "A plan can have at most 10 beneficiaries.",
  5: "One of the claim codes is invalid.",
  6: "Beneficiary allocations must add up to exactly 100%.",
  7: "The description is too long (max 500 characters).",
  8: "One of the beneficiaries has invalid data.",
  9: "Wallet is not authorized to perform this action.",
  13: "One of the beneficiary allocations is invalid.",
  14: "Claim codes must be 6 digits (0-999999).",
  20: "The contract has no admin configured yet.",
  23: "This wallet hasn't completed KYC approval yet.",
  25: "Beneficiary priorities must be unique.",
  26: "Beneficiary priority must be greater than zero.",
  29: "Insufficient token balance to fund this plan.",
  30: "The 2% creation fee transfer failed.",
};

export class ContractSimulationError extends Error {
  constructor(
    message: string,
    public readonly cause?: unknown
  ) {
    super(message);
    this.name = "ContractSimulationError";
  }
}

/** Parses a Soroban host trap message like `HostError: ... Error(Contract, #23)` into a friendly string. */
export function describeContractError(raw: string): string {
  const match = raw.match(/Error\(Contract,\s*#(\d+)\)/);
  if (match) {
    const code = Number(match[1]);
    const known = INHERITANCE_ERROR_MESSAGES[code];
    if (known) return known;
    return `Contract rejected the transaction (error #${code}).`;
  }
  return raw;
}

function getServer(): rpc.Server {
  return new rpc.Server(SOROBAN_RPC_URL, {
    allowHttp: SOROBAN_RPC_URL.startsWith("http://"),
  });
}

export interface BuildCreateInheritancePlanTxResult {
  /** Base64-encoded, fully-assembled (simulated + footprint-resolved) unsigned transaction XDR. */
  unsignedTransactionXdr: string;
  /** Network passphrase the transaction was built and must be signed against. */
  networkPassphrase: string;
}

/**
 * Builds, simulates and assembles the `create_inheritance_plan` invocation
 * into a ready-to-sign transaction XDR. Simulation determines the Soroban
 * resource footprint/fees and surfaces contract validation errors (e.g.
 * insufficient balance, bad allocations) before the wallet ever prompts the
 * user to sign anything.
 */
export async function buildCreateInheritancePlanTransaction(
  params: CreateInheritancePlanParamsInput
): Promise<BuildCreateInheritancePlanTxResult> {
  const contractId = getInheritanceContractId();
  if (!contractId) {
    throw new ContractSimulationError(
      "NEXT_PUBLIC_INHERITANCE_CONTRACT_ID is not configured for this environment."
    );
  }

  const server = getServer();

  let sourceAccount;
  try {
    sourceAccount = await server.getAccount(params.owner);
  } catch (error) {
    throw new ContractSimulationError(
      "Couldn't load your account from the Stellar network. Make sure the wallet is funded on this network.",
      error
    );
  }

  const contract = new Contract(contractId);
  const paramsScVal = encodeCreateInheritancePlanParams(params);
  const operation = contract.call("create_inheritance_plan", paramsScVal);

  const transaction = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(TRANSACTION_TIMEOUT_SECONDS)
    .build();

  let prepared;
  try {
    prepared = await server.prepareTransaction(transaction);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new ContractSimulationError(describeContractError(message), error);
  }

  return {
    unsignedTransactionXdr: prepared.toXDR(),
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  };
}

// ─── Emergency pause (admin) ────────────────────────────────────────────────

export type ContractPauseAction = "pause" | "unpause";

/** Errors `pause` / `unpause` can return (contracts/inheritance-contract/src/errors.rs). */
const PAUSE_ERROR_MESSAGES: Record<number, string> = {
  22: "This wallet is not an admin of the inheritance contract.",
  51: "The contract is mid-operation and can't be paused right now. Try again in a moment.",
  52: "This admin wallet is blacklisted on the contract.",
};

function describePauseError(raw: string): string {
  const match = raw.match(/Error\(Contract,\s*#(\d+)\)/);
  const known = match ? PAUSE_ERROR_MESSAGES[Number(match[1])] : undefined;
  return known ?? describeContractError(raw);
}

function requireInheritanceContractId(): string {
  const contractId = getInheritanceContractId();
  if (!contractId) {
    throw new ContractSimulationError(
      "NEXT_PUBLIC_INHERITANCE_CONTRACT_ID is not configured for this environment."
    );
  }
  return contractId;
}

async function loadAccount(server: rpc.Server, address: string) {
  try {
    return await server.getAccount(address);
  } catch (error) {
    throw new ContractSimulationError(
      "Couldn't load your account from the Stellar network. Make sure the wallet is funded on this network.",
      error
    );
  }
}

/**
 * Builds, simulates and assembles `pause(admin)` or `unpause(admin)` on the
 * inheritance contract. The contract checks `admin.require_auth()` and the
 * Admin role, so a non-admin wallet is rejected here, at simulation, before
 * the wallet is ever asked to sign.
 */
export async function buildContractPauseTransaction(
  admin: string,
  action: ContractPauseAction
): Promise<BuildCreateInheritancePlanTxResult> {
  const contract = new Contract(requireInheritanceContractId());
  const server = getServer();
  const sourceAccount = await loadAccount(server, admin);

  const transaction = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(action, new Address(admin).toScVal()))
    .setTimeout(TRANSACTION_TIMEOUT_SECONDS)
    .build();

  try {
    const prepared = await server.prepareTransaction(transaction);
    return {
      unsignedTransactionXdr: prepared.toXDR(),
      networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new ContractSimulationError(describePauseError(message), error);
  }
}

/** Reads `is_paused()` by simulation, using `viewer` as the source account. */
export async function readContractPaused(viewer: string): Promise<boolean> {
  const contract = new Contract(requireInheritanceContractId());
  const server = getServer();
  const sourceAccount = await loadAccount(server, viewer);
  const transaction = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call("is_paused"))
    .setTimeout(TRANSACTION_TIMEOUT_SECONDS)
    .build();

  const simulation = await server.simulateTransaction(transaction);
  if (!rpc.Api.isSimulationSuccess(simulation) || !simulation.result) {
    throw new ContractSimulationError("Couldn't read the contract's pause state.", simulation);
  }
  return scValToNative(simulation.result.retval) === true;
}
