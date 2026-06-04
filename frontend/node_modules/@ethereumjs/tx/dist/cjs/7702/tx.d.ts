import type { Address, EOACode7702AuthorizationListBytes } from '@ethereumjs/util';
import { TransactionType } from '../types.ts';
import type { Common } from '@ethereumjs/common';
import type { AccessListBytes, TxData as AllTypesTxData, TxValuesArray as AllTypesTxValuesArray, Capability, JSONTx, TransactionCache, TransactionInterface, TxOptions } from '../types.ts';
export type TxData = AllTypesTxData[typeof TransactionType.EOACodeEIP7702];
export type TxValuesArray = AllTypesTxValuesArray[typeof TransactionType.EOACodeEIP7702];
/**
 * Typed transaction with the ability to set codes on EOA accounts
 *
 * - TransactionType: 4
 * - EIP: [EIP-7702](https://github.com/ethereum/EIPs/blob/62419ca3f45375db00b04a368ea37c0bfb05386a/EIPS/eip-7702.md)
 */
export declare class EOACode7702Tx implements TransactionInterface<typeof TransactionType.EOACodeEIP7702> {
    type: 4;
    readonly nonce: bigint;
    readonly gasLimit: bigint;
    readonly value: bigint;
    readonly data: Uint8Array;
    readonly to?: Address;
    readonly accessList: AccessListBytes;
    readonly authorizationList: EOACode7702AuthorizationListBytes;
    readonly chainId: bigint;
    readonly maxPriorityFeePerGas: bigint;
    readonly maxFeePerGas: bigint;
    readonly v?: bigint;
    readonly r?: bigint;
    readonly s?: bigint;
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
     * The amount of gas paid for the data in this tx
     */
    getDataGas(): bigint;
    /**
     * Returns the minimum of calculated priority fee (from maxFeePerGas and baseFee) and maxPriorityFeePerGas
     * @param baseFee Base fee retrieved from block
     */
    getEffectivePriorityFee(baseFee: bigint): bigint;
    /**
     * The up front amount that an account must have for this transaction to be valid
     * @param baseFee The base fee of the block (will be set to 0 if not provided)
     */
    getUpfrontCost(baseFee?: bigint): bigint;
    /**
     * The minimum gas limit which the tx to have to be valid.
     * This covers costs as the standard fee (21000 gas), the data fee (paid for each calldata byte),
     * the optional creation fee (if the transaction creates a contract), and if relevant the gas
     * to be paid for access lists (EIP-2930) and authority lists (EIP-7702).
     */
    getIntrinsicGas(): bigint;
    /**
     * EOACode7702Tx cannot create contracts
     */
    toCreationAddress(): never;
    /**
     * Returns a Uint8Array Array of the raw Bytes of the EIP-7702 transaction, in order.
     *
     * Format: `[chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
     * accessList, authorizationList, signatureYParity, signatureR, signatureS]`
     *
     * Use {@link EOACode7702Transaction.serialize} to add a transaction to a block
     * with {@link createBlockFromBytesArray}.
     *
     * For an unsigned tx this method uses the empty Bytes values for the
     * signature parameters `v`, `r` and `s` for encoding. For an EIP-155 compliant
     * representation for external signing use {@link EOACode7702Transaction.getMessageToSign}.
     */
    raw(): TxValuesArray;
    /**
     * Returns the serialized encoding of the EIP-7702 transaction.
     *
     * Format: `0x02 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
     * accessList, authorizationList, signatureYParity, signatureR, signatureS])`
     *
     * Note that in contrast to the legacy tx serialization format this is not
     * valid RLP any more due to the raw tx type preceding and concatenated to
     * the RLP encoding of the values.
     */
    serialize(): Uint8Array;
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
     * Use {@link EOACode7702Transaction.getMessageToSign} to get a tx hash for the purpose of signing.
     * @returns Hash of the serialized signed transaction
     */
    hash(): Uint8Array;
    /**
     * Computes a sha3-256 hash which can be used to verify the signature
     * @returns Hash used when verifying the signature
     */
    getMessageToVerifySignature(): Uint8Array;
    /**
     * Returns the public key of the sender
     * @returns Sender public key
     */
    getSenderPublicKey(): Uint8Array;
    /**
     * Adds the provided signature values and returns a new transaction instance.
     * @param v - Recovery parameter
     * @param r - Signature `r` value
     * @param s - Signature `s` value
     * @returns New `EOACode7702Tx` that includes the signature
     */
    addSignature(v: bigint, r: Uint8Array | bigint, s: Uint8Array | bigint): EOACode7702Tx;
    /**
     * Returns an object with the JSON representation of the transaction
     * @returns JSON encoding of the transaction
     */
    toJSON(): JSONTx;
    /**
     * Returns the list of validation errors, if any.
     * @returns Array of validation error messages
     */
    getValidationErrors(): string[];
    /**
     * @returns true if the transaction has no validation issues
     */
    isValid(): boolean;
    /**
     * Verifies the embedded signature.
     * @returns true if signature verification succeeds
     */
    verifySignature(): boolean;
    /**
     * Returns the recovered sender address.
     * @returns Sender {@link Address}
     */
    getSenderAddress(): Address;
    /**
     * Signs the transaction and returns the signed instance.
     * @param privateKey - 32-byte private key
     * @param extraEntropy - Optional entropy supplied to the signing routine
     * @returns Newly signed transaction
     */
    sign(privateKey: Uint8Array, extraEntropy?: Uint8Array | boolean): EOACode7702Tx;
    /**
     * Indicates whether the transaction already carries signature data.
     * @returns true if signature parts are present
     */
    isSigned(): boolean;
    /**
     * Return a compact error string representation of the object
     */
    errorStr(): string;
}
//# sourceMappingURL=tx.d.ts.map