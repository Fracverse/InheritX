import { TransactionType } from '../types.ts';
import type { Common } from '@ethereumjs/common';
import type { Address } from '@ethereumjs/util';
import type { AccessListBytes, TxData as AllTypesTxData, TxValuesArray as AllTypesTxValuesArray, Capability, JSONTx, TransactionCache, TransactionInterface, TxOptions } from '../types.ts';
export type TxData = AllTypesTxData[typeof TransactionType.AccessListEIP2930];
export type TxValuesArray = AllTypesTxValuesArray[typeof TransactionType.AccessListEIP2930];
/**
 * Typed transaction with optional access lists
 *
 * - TransactionType: 1
 * - EIP: [EIP-2930](https://eips.ethereum.org/EIPS/eip-2930)
 */
export declare class AccessList2930Tx implements TransactionInterface<typeof TransactionType.AccessListEIP2930> {
    type: 1;
    readonly gasPrice: bigint;
    readonly nonce: bigint;
    readonly gasLimit: bigint;
    readonly value: bigint;
    readonly data: Uint8Array;
    readonly to?: Address;
    readonly accessList: AccessListBytes;
    readonly chainId: bigint;
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
    getEffectivePriorityFee(baseFee?: bigint): bigint;
    /**
     * The amount of gas paid for the data in this tx
     */
    getDataGas(): bigint;
    /**
     * The up front amount that an account must have for this transaction to be valid
     */
    getUpfrontCost(): bigint;
    /**
     * The minimum gas limit which the tx to have to be valid.
     * This covers costs as the standard fee (21000 gas), the data fee (paid for each calldata byte),
     * the optional creation fee (if the transaction creates a contract), and if relevant the gas
     * to be paid for access lists (EIP-2930) and authority lists (EIP-7702).
     */
    getIntrinsicGas(): bigint;
    /**
     * If the tx's `to` is to the creation address
     */
    toCreationAddress(): boolean;
    /**
     * Returns a Uint8Array Array of the raw Bytes of the EIP-2930 transaction, in order.
     *
     * Format: `[chainId, nonce, gasPrice, gasLimit, to, value, data, accessList,
     * signatureYParity (v), signatureR (r), signatureS (s)]`
     *
     * Use {@link AccessList2930Tx.serialize} to add a transaction to a block
     * with {@link createBlockFromBytesArray}.
     *
     * For an unsigned tx this method uses the empty Bytes values for the
     * signature parameters `v`, `r` and `s` for encoding. For an EIP-155 compliant
     * representation for external signing use {@link AccessList2930Tx.getMessageToSign}.
     */
    raw(): TxValuesArray;
    /**
     * Returns the serialized encoding of the EIP-2930 transaction.
     *
     * Format: `0x01 || rlp([chainId, nonce, gasPrice, gasLimit, to, value, data, accessList,
     * signatureYParity (v), signatureR (r), signatureS (s)])`
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
     * Use {@link Transaction.getMessageToSign} to get a tx hash for the purpose of signing.
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
     * @param v - Recovery parameter (y-parity)
     * @param r - `r` component of the signature
     * @param s - `s` component of the signature
     * @returns New `AccessList2930Tx` with the supplied signature
     */
    addSignature(v: bigint, r: Uint8Array | bigint, s: Uint8Array | bigint): AccessList2930Tx;
    /**
     * Returns an object with the JSON representation of the transaction
     * @returns JSON encoding of the transaction
     */
    toJSON(): JSONTx;
    /**
     * Runs transaction validation and returns any discovered errors.
     * @returns Array of validation error messages
     */
    getValidationErrors(): string[];
    /**
     * @returns true if the transaction has no validation errors
     */
    isValid(): boolean;
    /**
     * Checks whether the signature currently attached to the transaction is valid.
     * @returns true if signature verification succeeds
     */
    verifySignature(): boolean;
    /**
     * Returns the signer's address recovered from the signature.
     * @returns Sender {@link Address}
     */
    getSenderAddress(): Address;
    /**
     * Signs the transaction with the provided private key and returns a new instance.
     * @param privateKey - 32-byte private key
     * @param extraEntropy - Optional entropy fed into the signing algorithm
     * @returns Newly signed transaction
     */
    sign(privateKey: Uint8Array, extraEntropy?: Uint8Array | boolean): AccessList2930Tx;
    /**
     * Reports whether the transaction already contains signature values.
     * @returns true if signature parts are present
     */
    isSigned(): boolean;
    /**
     * Return a compact error string representation of the object
     */
    errorStr(): string;
}
//# sourceMappingURL=tx.d.ts.map