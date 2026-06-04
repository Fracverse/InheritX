import { Capability, TransactionType } from '../types.ts';
import type { Common } from '@ethereumjs/common';
import type { Address } from '@ethereumjs/util';
import type { TxData as AllTypesTxData, TxValuesArray as AllTypesTxValuesArray, JSONTx, TransactionCache, TransactionInterface, TxOptions } from '../types.ts';
export type TxData = AllTypesTxData[typeof TransactionType.Legacy];
export type TxValuesArray = AllTypesTxValuesArray[typeof TransactionType.Legacy];
/**
 * An Ethereum non-typed (legacy) transaction
 */
export declare class LegacyTx implements TransactionInterface<typeof TransactionType.Legacy> {
    type: 0;
    readonly gasPrice: bigint;
    readonly nonce: bigint;
    readonly gasLimit: bigint;
    readonly value: bigint;
    readonly data: Uint8Array;
    readonly to?: Address;
    readonly v?: bigint;
    readonly r?: bigint;
    readonly s?: bigint;
    readonly common: Common;
    private keccakFunction;
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
     * the static factory methods to assist in creating a Transaction object from
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
     * Indicates whether the transaction already contains signature values.
     * @returns true if `v`, `r`, and `s` are populated
     */
    isSigned(): boolean;
    /**
     * Computes the effective priority fee for this legacy transaction, optionally considering a base fee.
     * @param baseFee - Optional base fee used on networks that emulate 1559-style pricing
     * @returns Priority fee portion denominated in wei
     */
    getEffectivePriorityFee(baseFee?: bigint): bigint;
    /**
     * Returns a Uint8Array Array of the raw Bytes of the legacy transaction, in order.
     *
     * Format: `[nonce, gasPrice, gasLimit, to, value, data, v, r, s]`
     *
     * For legacy txs this is also the correct format to add transactions
     * to a block with {@link createBlockFromBytesArray} (use the `serialize()` method
     * for typed txs).
     *
     * For an unsigned tx this method returns the empty Bytes values
     * for the signature parameters `v`, `r` and `s`. For an EIP-155 compliant
     * representation have a look at {@link Transaction.getMessageToSign}.
     */
    raw(): TxValuesArray;
    /**
     * Returns the serialized encoding of the legacy transaction.
     *
     * Format: `rlp([nonce, gasPrice, gasLimit, to, value, data, v, r, s])`
     *
     * For an unsigned tx this method uses the empty Uint8Array values for the
     * signature parameters `v`, `r` and `s` for encoding. For an EIP-155 compliant
     * representation for external signing use {@link Transaction.getMessageToSign}.
     */
    serialize(): Uint8Array;
    /**
     * Returns the raw unsigned tx, which can be used
     * to sign the transaction (e.g. for sending to a hardware wallet).
     *
     * Note: the raw message message format for the legacy tx is not RLP encoded
     * and you might need to do yourself with:
     *
     * ```javascript
     * import { RLP } from '@ethereumjs/rlp'
     * const message = tx.getMessageToSign()
     * const serializedMessage = RLP.encode(message)) // use this for the HW wallet input
     * ```
     * @returns Array representing the unsigned transaction fields
     */
    getMessageToSign(): Uint8Array[];
    /**
     * Returns the hashed serialized unsigned tx, which can be used
     * to sign the transaction (e.g. for sending to a hardware wallet).
     * @returns Hash of the unsigned transaction payload
     */
    getHashedMessageToSign(): Uint8Array<ArrayBufferLike>;
    /**
     * The amount of gas paid for the data in this tx
     */
    getDataGas(): bigint;
    /**
     * If the tx's `to` is to the creation address
     */
    toCreationAddress(): boolean;
    /**
     * The minimum gas limit which the tx to have to be valid.
     * This covers costs as the standard fee (21000 gas), the data fee (paid for each calldata byte),
     * the optional creation fee (if the transaction creates a contract), and if relevant the gas
     * to be paid for access lists (EIP-2930) and authority lists (EIP-7702).
     */
    getIntrinsicGas(): bigint;
    /**
     * The up front amount that an account must have for this transaction to be valid
     */
    getUpfrontCost(): bigint;
    /**
     * Computes a sha3-256 hash of the serialized tx.
     *
     * This method can only be used for signed txs (it throws otherwise).
     * Use {@link Transaction.getMessageToSign} to get a tx hash for the purpose of signing.
     * @returns Hash of the serialized signed transaction
     */
    hash(): Uint8Array;
    /**
     * Computes a sha3-256 hash which can be used to verify the signature
     * @returns Hash used when verifying the signature
     */
    getMessageToVerifySignature(): Uint8Array<ArrayBufferLike>;
    /**
     * Returns the public key of the sender
     * @returns Sender public key
     */
    getSenderPublicKey(): Uint8Array;
    /**
     * Adds a signature (or replaces an existing one) and returns a new transaction instance.
     * @param v - Recovery parameter, potentially unconverted when `convertV` is false
     * @param r - `r` value of the signature
     * @param s - `s` value of the signature
     * @param convertV - When true, converts the recovery ID into the appropriate legacy `v`
     * @returns A new `LegacyTx` that includes the provided signature
     */
    addSignature(v: bigint, r: Uint8Array | bigint, s: Uint8Array | bigint, convertV?: boolean): LegacyTx;
    /**
     * Returns an object with the JSON representation of the transaction.
     * @returns JSON encoding of the transaction
     */
    toJSON(): JSONTx;
    /**
     * Validates the transaction and returns any encountered errors.
     * @returns Array containing validation error messages
     */
    getValidationErrors(): string[];
    /**
     * Determines whether the transaction passes all validation checks.
     * @returns true if no validation errors were found
     */
    isValid(): boolean;
    /**
     * Checks whether the stored signature can be successfully verified.
     * @returns true if the signature is valid
     */
    verifySignature(): boolean;
    /**
     * Returns the recovered sender address.
     * @returns Sender {@link Address}
     */
    getSenderAddress(): Address;
    /**
     * Signs the transaction with the provided private key and returns the new signed instance.
     * @param privateKey - 32-byte private key used to sign the transaction
     * @param extraEntropy - Optional entropy passed to the signing routine
     * @returns A new signed `LegacyTx`
     */
    sign(privateKey: Uint8Array, extraEntropy?: Uint8Array | boolean): LegacyTx;
    /**
     * Return a compact error string representation of the object
     * @returns Human-readable error summary
     */
    errorStr(): string;
}
//# sourceMappingURL=tx.d.ts.map