import type { PrefixedHexString } from '../types.ts';
type BALAddressHex = PrefixedHexString;
type BALStorageKeyBytes = Uint8Array;
type BALStorageKeyHex = PrefixedHexString;
type BALStorageValueBytes = Uint8Array;
type BALStorageValueHex = PrefixedHexString;
type BALAccessIndexNumber = number;
type BALAccessIndexHex = PrefixedHexString;
type BALBalanceBigInt = bigint;
type BALBalanceHex = PrefixedHexString;
type BALNonceBigInt = bigint;
type BALNonceHex = PrefixedHexString;
type BALByteCodeBytes = Uint8Array;
type BALByteCodeHex = PrefixedHexString;
type BALRawStorageChange = [BALAccessIndexNumber, BALStorageValueBytes];
type BALRawBalanceChange = [BALAccessIndexNumber, BALBalanceHex];
type BALRawNonceChange = [BALAccessIndexNumber, BALNonceHex];
type BALRawCodeChange = [BALAccessIndexNumber, BALByteCodeBytes];
type BALRawSlotChanges = [BALStorageKeyHex, BALRawStorageChange[]];
type BALRawAccountChanges = [
    BALAddressHex,
    BALRawSlotChanges[],
    BALStorageKeyHex[],
    BALRawBalanceChange[],
    BALRawNonceChange[],
    BALRawCodeChange[]
];
type BALRawBlockAccessList = BALRawAccountChanges[];
export type Accesses = Record<BALAddressHex, {
    nonceChanges: Map<BALAccessIndexNumber, BALNonceHex>;
    balanceChanges: Map<BALAccessIndexNumber, BALBalanceHex>;
    codeChanges: BALRawCodeChange[];
    storageChanges: Record<BALStorageKeyHex, BALRawStorageChange[]>;
    storageReads: Set<BALStorageKeyHex>;
}>;
interface BALJSONBalanceChange {
    blockAccessIndex: BALAccessIndexHex;
    postBalance: BALBalanceHex;
}
interface BALJSONNonceChange {
    blockAccessIndex: BALAccessIndexHex;
    postNonce: BALNonceHex;
}
interface BALJSONCodeChange {
    blockAccessIndex: BALAccessIndexHex;
    newCode: BALByteCodeHex;
}
interface BALJSONStorageChange {
    blockAccessIndex: BALAccessIndexHex;
    postValue: BALStorageValueHex;
}
interface BALJSONSlotChanges {
    slot: BALStorageKeyHex;
    slotChanges: BALJSONStorageChange[];
}
interface BALJSONAccountChanges {
    address: BALAddressHex;
    balanceChanges: BALJSONBalanceChange[];
    nonceChanges: BALJSONNonceChange[];
    codeChanges: BALJSONCodeChange[];
    storageChanges: BALJSONSlotChanges[];
    storageReads: BALStorageKeyHex[];
}
/** @remarks Experimental (Amsterdam): may change on patch releases. */
export type BALJSONBlockAccessList = BALJSONAccountChanges[];
export type { BALJSONAccountChanges, BALJSONStorageChange, BALJSONSlotChanges, BALJSONBalanceChange, BALJSONNonceChange, BALJSONCodeChange, };
/**
 * In-memory [EIP-7928](https://eips.ethereum.org/EIPS/eip-7928) block access list with
 * canonical RLP/JSON encoding, checkpointing, and mutation helpers used by the VM during execution.
 *
 * @remarks Experimental (Amsterdam): public API and behaviour may change on patch releases.
 * See `@ethereumjs/vm` README section `Amsterdam hardfork (experimental)` for release ↔ spec tracking.
 */
export declare class BlockLevelAccessList {
    /** Account-level access entries keyed by address. */
    accesses: Accesses;
    /** Current block access index (transaction or system phase) for new change records. */
    blockAccessIndex: number;
    private checkpoints;
    private originalBalances;
    private originalCodesAtIndex;
    constructor(accesses?: Accesses);
    /**
     * Canonical RLP encoding of the access list (`RLP.encode(raw())`).
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    serialize(): Uint8Array;
    /**
     * Header commitment `keccak256(serialize())` used as `blockAccessListHash`.
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    hash(): Uint8Array;
    checkpoint(): void;
    commit(): void;
    revert(): void;
    private cloneAccesses;
    /**
     * Canonical sorted tuple view used for RLP and validation.
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    raw(): BALRawBlockAccessList;
    addAddress(address: BALAddressHex): void;
    addStorageWrite(address: BALAddressHex, storageKey: BALStorageKeyBytes, value: BALStorageValueBytes, blockAccessIndex: BALAccessIndexNumber, originalValue?: BALStorageValueBytes): void;
    addStorageRead(address: BALAddressHex, storageKey: BALStorageKeyBytes): void;
    addBalanceChange(address: BALAddressHex, balance: BALBalanceBigInt, blockAccessIndex: BALAccessIndexNumber, originalBalance?: BALBalanceBigInt): void;
    /**
     * EIP-7928: Remove balance changes for addresses where final balance equals first balance.
     * Call this at the end of each transaction to clean up net-zero balance changes.
     */
    cleanupNetZeroBalanceChanges(): void;
    addNonceChange(address: BALAddressHex, nonce: BALNonceBigInt, blockAccessIndex: BALAccessIndexNumber): void;
    addCodeChange(address: BALAddressHex, code: BALByteCodeBytes, blockAccessIndex: BALAccessIndexNumber, originalCode?: BALByteCodeBytes): void;
    /**
     * Converts the internal representation to JSON fixture / Engine API form.
     * Inverse of {@link createBlockLevelAccessListFromJSON}.
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    toJSON(): BALJSONBlockAccessList;
    /**
     * For selfdestructed accounts, drop state changes while preserving read footprints.
     * Any `storageChanges` are converted to `storageReads`. Per EIP-7928, a positive
     * pre-transaction balance reduced to zero via `SELFDESTRUCT` keeps the balance change.
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    cleanupSelfdestructed(addresses: Array<BALAddressHex>): void;
}
/**
 * Creates an empty {@link BlockLevelAccessList}.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function createBlockLevelAccessList(): BlockLevelAccessList;
/**
 * Parses a JSON block access list (fixture / Engine API order).
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function createBlockLevelAccessListFromJSON(json: BALJSONBlockAccessList): BlockLevelAccessList;
/**
 * Parses an RLP-encoded block access list.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function createBlockLevelAccessListFromRLP(rlp: Uint8Array): BlockLevelAccessList;
//# sourceMappingURL=index.d.ts.map