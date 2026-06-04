/**
 * External Interfaces for other EthereumJS libraries
 */
import type { Account, Address, BinaryTreeExecutionWitness, PrefixedHexString } from '@ethereumjs/util';
export interface StorageDump {
    [key: string]: string;
}
/**
 * Object that can contain a set of storage keys associated with an account.
 */
export interface StorageRange {
    /**
     * A dictionary where the keys are hashed storage keys, and the values are
     * objects containing the preimage of the hashed key (in `key`) and the
     * storage key (in `value`). Currently, there is no way to retrieve preimages,
     * so they are always `null`.
     */
    storage: {
        [key: string]: {
            key: string | null;
            value: string;
        };
    };
    /**
     * The next (hashed) storage key after the greatest storage key
     * contained in `storage`.
     */
    nextKey: string | null;
}
export type AccountFields = Partial<Pick<Account, 'nonce' | 'balance' | 'storageRoot' | 'codeHash' | 'codeSize'>>;
export type StorageProof = {
    key: PrefixedHexString;
    proof: PrefixedHexString[];
    value: PrefixedHexString;
};
export type Proof = {
    address: PrefixedHexString;
    balance: PrefixedHexString;
    codeHash: PrefixedHexString;
    nonce: PrefixedHexString;
    storageHash: PrefixedHexString;
    accountProof: PrefixedHexString[];
    storageProof: StorageProof[];
};
/**
 * Binary tree related
 *
 * Experimental (do not implement)
 */
export type AccessEventFlags = {
    stemRead: boolean;
    stemWrite: boolean;
    chunkRead: boolean;
    chunkWrite: boolean;
    chunkFill: boolean;
};
export type BinaryTreeAccessedStateType = (typeof BinaryTreeAccessedStateType)[keyof typeof BinaryTreeAccessedStateType];
export declare const BinaryTreeAccessedStateType: {
    readonly BasicData: "basicData";
    readonly CodeHash: "codeHash";
    readonly Code: "code";
    readonly Storage: "storage";
};
export type RawBinaryTreeAccessedState = {
    address: Address;
    treeIndex: number | bigint;
    chunkIndex: number;
    chunkKey: PrefixedHexString;
};
export type BinaryTreeAccessedState = {
    type: Exclude<BinaryTreeAccessedStateType, typeof BinaryTreeAccessedStateType.Code | typeof BinaryTreeAccessedStateType.Storage>;
} | {
    type: typeof BinaryTreeAccessedStateType.Code;
    codeOffset: number;
} | {
    type: typeof BinaryTreeAccessedStateType.Storage;
    slot: bigint;
};
export type BinaryTreeAccessedStateWithAddress = BinaryTreeAccessedState & {
    address: Address;
    chunkKey: PrefixedHexString;
};
export interface BinaryTreeAccessWitnessInterface {
    accesses(): Generator<BinaryTreeAccessedStateWithAddress>;
    rawAccesses(): Generator<RawBinaryTreeAccessedState>;
    debugWitnessCost(): void;
    readAccountBasicData(address: Address): bigint;
    writeAccountBasicData(address: Address): bigint;
    readAccountCodeHash(address: Address): bigint;
    writeAccountCodeHash(address: Address): bigint;
    readAccountHeader(address: Address): bigint;
    writeAccountHeader(address: Address): bigint;
    readAccountCodeChunks(contract: Address, startPc: number, endPc: number): bigint;
    writeAccountCodeChunks(contract: Address, startPc: number, endPc: number): bigint;
    readAccountStorage(contract: Address, storageSlot: bigint): bigint;
    writeAccountStorage(contract: Address, storageSlot: bigint): bigint;
    merge(accessWitness: BinaryTreeAccessWitnessInterface): void;
    commit(): void;
    revert(): void;
}
export interface StateManagerInterface {
    getAccount(address: Address): Promise<Account | undefined>;
    putAccount(address: Address, account?: Account): Promise<void>;
    deleteAccount(address: Address): Promise<void>;
    modifyAccountFields(address: Address, accountFields: AccountFields): Promise<void>;
    putCode(address: Address, value: Uint8Array): Promise<void>;
    getCode(address: Address): Promise<Uint8Array>;
    getCodeSize(address: Address): Promise<number>;
    getStorage(address: Address, key: Uint8Array): Promise<Uint8Array>;
    putStorage(address: Address, key: Uint8Array, value: Uint8Array): Promise<void>;
    clearStorage(address: Address): Promise<void>;
    checkpoint(): Promise<void>;
    commit(): Promise<void>;
    revert(): Promise<void>;
    getStateRoot(): Promise<Uint8Array>;
    setStateRoot(stateRoot: Uint8Array, clearCache?: boolean): Promise<void>;
    hasStateRoot(root: Uint8Array): Promise<boolean>;
    dumpStorage?(address: Address): Promise<StorageDump>;
    dumpStorageRange?(address: Address, startKey: bigint, limit: number): Promise<StorageRange>;
    originalStorageCache: {
        get(address: Address, key: Uint8Array): Promise<Uint8Array>;
        clear(): void;
    };
    generateCanonicalGenesis?(initState: any): Promise<void>;
    initBinaryTreeExecutionWitness?(blockNum: bigint, executionWitness?: BinaryTreeExecutionWitness | null): void;
    verifyBinaryTreePostState?(accessWitness: BinaryTreeAccessWitnessInterface): Promise<boolean>;
    checkChunkWitnessPresent?(contract: Address, programCounter: number): Promise<boolean>;
    getAppliedKey?(address: Uint8Array): Uint8Array;
    clearCaches(): void;
    shallowCopy(downlevelCaches?: boolean): StateManagerInterface;
}
//# sourceMappingURL=interfaces.d.ts.map