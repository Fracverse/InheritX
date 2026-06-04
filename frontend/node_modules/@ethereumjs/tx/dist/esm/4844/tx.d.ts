import { TransactionType } from '../types.ts';
import type { Common } from '@ethereumjs/common';
import type { Address, PrefixedHexString } from '@ethereumjs/util';
import type { AccessListBytes, TxData as AllTypesTxData, TxValuesArray as AllTypesTxValuesArray, Capability, JSONTx, TransactionCache, TransactionInterface, TxOptions } from '../types.ts';
export type TxData = AllTypesTxData[typeof TransactionType.BlobEIP4844];
export type TxValuesArray = AllTypesTxValuesArray[typeof TransactionType.BlobEIP4844];
export declare const NetworkWrapperType: {
    readonly EIP4844: 0;
    readonly EIP7594: 1;
};
export type NetworkWrapperType = (typeof NetworkWrapperType)[keyof typeof NetworkWrapperType];
/**
 * Typed transaction with a new gas fee market mechanism for transactions that include "blobs" of data
 *
 * - TransactionType: 3
 * - EIP: [EIP-4844](https://eips.ethereum.org/EIPS/eip-4844)
 *
 * This tx type has two "modes": the plain canonical format only contains `blobVersionedHashes`.
 * If blobs are passed in the tx automatically switches to "Network Wrapper" format and the
 * `networkWrapperVersion` will be set or validated.
 */
export declare class Blob4844Tx implements TransactionInterface<typeof TransactionType.BlobEIP4844> {
    type: 3;
    readonly nonce: bigint;
    readonly gasLimit: bigint;
    readonly value: bigint;
    readonly data: Uint8Array;
    readonly to?: Address;
    readonly accessList: AccessListBytes;
    readonly chainId: bigint;
    readonly maxPriorityFeePerGas: bigint;
    readonly maxFeePerGas: bigint;
    readonly maxFeePerBlobGas: bigint;
    blobVersionedHashes: PrefixedHexString[];
    readonly v?: bigint;
    readonly r?: bigint;
    readonly s?: bigint;
    /**
     * This property is set if the tx is in "Network Wrapper" format.
     *
     * Possible values:
     * - 0 (EIP-4844)
     * - 1 (EIP-4844 + EIP-7594)
     */
    networkWrapperVersion?: NetworkWrapperType;
    blobs?: PrefixedHexString[];
    kzgCommitments?: PrefixedHexString[];
    kzgProofs?: PrefixedHexString[];
    readonly common: Common;
    readonly txOptions: TxOptions;
    readonly cache: TransactionCache;
    /**
     * List of tx type defining EIPs,
     * e.g. 1559 (fee market) and 2930 (access lists)
     * for FeeMarket1559Tx objects
     */
    protected activeCapabilities: number[];
    /**
     * This constructor takes the values, validates them, assigns them and freezes the object.
     *
     * It is not recommended to use this constructor directly. Instead use
     * the static constructors or factory methods to assist in creating a Transaction object from
     * varying data types.
     */
    constructor(txData: TxData, opts?: TxOptions);
    /**
     * Checks if a tx type defining capability is active
     * on a tx, for example the EIP-1559 fee market mechanism
     * or the EIP-2930 access list feature.
     *
     * Note that this is different from the tx type itself,
     * so EIP-2930 access lists can very well be active
     * on an EIP-1559 tx for example.
     *
     * This method can be useful for feature checks if the
     * tx type is unknown (e.g. when instantiated with
     * the tx factory).
     *
     * See `Capabilities` in the `types` module for a reference
     * on all supported capabilities.
     */
    supports(capability: Capability): boolean;
    /**
     * Returns the minimum of calculated priority fee (from maxFeePerGas and baseFee) and maxPriorityFeePerGas
     * @param baseFee Base fee retrieved from block
     */
    getEffectivePriorityFee(baseFee: bigint): bigint;
    /**
     * The amount of gas paid for the data in this tx
     */
    getDataGas(): bigint;
    /**
     * The up front amount that an account must have for this transaction to be valid
     * @param baseFee The base fee of the block (will be set to 0 if not provided)
     */
    getUpfrontCost(baseFee?: bigint): bigint;
    /**
     * Blob4844Tx cannot create contracts
     */
    toCreationAddress(): never;
    /**
     * The minimum gas limit which the tx to have to be valid.
     * This covers costs as the standard fee (21000 gas), the data fee (paid for each calldata byte),
     * the optional creation fee (if the transaction creates a contract), and if relevant the gas
     * to be paid for access lists (EIP-2930) and authority lists (EIP-7702).
     */
    getIntrinsicGas(): bigint;
    /**
     * Returns a Uint8Array Array of the raw Bytes of the EIP-4844 transaction, in order.
     *
     * Format: [chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, to, value, data,
     * access_list, max_fee_per_data_gas, blob_versioned_hashes, y_parity, r, s]`.
     *
     * Use {@link Blob4844Tx.serialize} to add a transaction to a block
     * with {@link createBlockFromBytesArray}.
     *
     * For an unsigned tx this method uses the empty Bytes values for the
     * signature parameters `v`, `r` and `s` for encoding. For an EIP-155 compliant
     * representation for external signing use {@link Blob4844Tx.getMessageToSign}.
     */
    raw(): TxValuesArray;
    /**
     * Returns the serialized encoding of the EIP-4844 transaction.
     *
     * Format: `0x03 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
     * access_list, max_fee_per_data_gas, blob_versioned_hashes, y_parity, r, s])`.
     *
     * Note that in contrast to the legacy tx serialization format this is not
     * valid RLP any more due to the raw tx type preceding and concatenated to
     * the RLP encoding of the values.
     */
    serialize(): Uint8Array;
    /**
     * @returns the serialized form of a blob transaction in the network wrapper format
     * This format is used for gossipping mempool transactions over devp2p or when
     * submitting a transaction via RPC.
     */
    serializeNetworkWrapper(): Uint8Array;
    /**
     * Returns the raw serialized unsigned tx, which can be used
     * to sign the transaction (e.g. for sending to a hardware wallet).
     *
     * Note: in contrast to the legacy tx the raw message format is already
     * serialized and doesn't need to be RLP encoded any more.
     *
     * ```javascript
     * const serializedMessage = tx.getMessageToSign() // use this for the HW wallet input
     * ```
     * @returns Serialized unsigned transaction payload
     */
    getMessageToSign(): Uint8Array;
    /**
     * Returns the hashed serialized unsigned tx, which can be used
     * to sign the transaction (e.g. for sending to a hardware wallet).
     *
     * Note: in contrast to the legacy tx the raw message format is already
     * serialized and doesn't need to be RLP encoded any more.
     * @returns Keccak hash of the unsigned transaction payload
     */
    getHashedMessageToSign(): Uint8Array;
    /**
     * Computes a sha3-256 hash of the serialized tx.
     *
     * This method can only be used for signed txs (it throws otherwise).
     * Use {@link Blob4844Tx.getMessageToSign} to get a tx hash for the purpose of signing.
     * @returns Hash of the serialized signed transaction
     */
    hash(): Uint8Array;
    /**
     * Returns the hashed unsigned transaction that should be used for signature verification.
     * @returns Hash of the unsigned transaction payload
     */
    getMessageToVerifySignature(): Uint8Array;
    /**
     * Returns the public key of the sender
     * @returns Sender public key
     */
    getSenderPublicKey(): Uint8Array;
    /**
     * Produces a JSON representation compliant with the execution API.
     * @returns JSON encoding of the transaction
     */
    toJSON(): JSONTx;
    /**
     * Adds signature values (and optional network wrapper fields) and returns a new transaction.
     * @param v - Recovery parameter
     * @param r - Signature `r` value
     * @param s - Signature `s` value
     * @returns New `Blob4844Tx` instance containing the signature
     */
    addSignature(v: bigint, r: Uint8Array | bigint, s: Uint8Array | bigint): Blob4844Tx;
    /**
     * Returns validation errors for this transaction, if any.
     * @returns Array of validation error messages
     */
    getValidationErrors(): string[];
    /**
     * @returns true if the transaction has no validation errors
     */
    isValid(): boolean;
    /**
     * Verifies whether the attached signature is valid.
     * @returns true if signature verification succeeds
     */
    verifySignature(): boolean;
    /**
     * Returns the recovered sender address.
     * @returns Sender {@link Address}
     */
    getSenderAddress(): Address;
    /**
     * Signs the transaction with the provided private key and returns the signed instance.
     * @param privateKey - 32-byte private key used for signing
     * @param extraEntropy - Optional entropy passed to the signing routine
     * @returns Newly signed transaction
     */
    sign(privateKey: Uint8Array, extraEntropy?: Uint8Array | boolean): Blob4844Tx;
    /**
     * Indicates whether the transaction already carries signature values.
     * @returns true if signature parts are present
     */
    isSigned(): boolean;
    /**
     * Return a compact error string representation of the object
     * @returns Human-readable error summary
     */
    errorStr(): string;
    /**
     * @returns the number of blobs included with this transaction
     */
    numBlobs(): number;
}
//# sourceMappingURL=tx.d.ts.map