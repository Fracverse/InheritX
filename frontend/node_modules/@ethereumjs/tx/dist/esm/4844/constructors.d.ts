import { Blob4844Tx } from './tx.ts';
import type { JSONBlobTxNetworkWrapper, TxOptions } from '../types.ts';
import type { TxData, TxValuesArray } from './tx.ts';
/**
 * Instantiate a Blob4844Tx transaction from a data dictionary.
 *
 * If blobs are provided the tx will be instantiated in the "Network Wrapper" format,
 * otherwise in the canonical form represented on-chain.
 *
 * @param txData - Transaction data object containing:
 *   - `chainId` - Chain ID (will be set automatically if not provided)
 *   - `nonce` - Transaction nonce
 *   - `maxPriorityFeePerGas` - Maximum priority fee per gas (EIP-1559)
 *   - `maxFeePerGas` - Maximum fee per gas (EIP-1559)
 *   - `gasLimit` - Gas limit for the transaction
 *   - `to` - Recipient address (optional for contract creation)
 *   - `value` - Value to transfer in wei
 *   - `data` - Transaction data
 *   - `accessList` - Access list for EIP-2930 (optional)
 *   - `maxFeePerBlobGas` - Maximum fee per blob gas (EIP-4844)
 *   - `blobVersionedHashes` - Versioned hashes for blob validation
 *   - `v`, `r`, `s` - Signature components (for signed transactions)
 *   - `blobs` - Raw blob data (optional, will derive commitments/proofs)
 *   - `blobsData` - Array of strings to construct blobs from (optional)
 *   - `kzgCommitments` - KZG commitments (optional, derived from blobs if not provided)
 *   - `kzgProofs` - KZG proofs (optional, derived from blobs if not provided)
 *   - `networkWrapperVersion` - Network wrapper version (0=EIP-4844, 1=EIP-7594)
 * @param opts - Transaction options including Common instance with KZG initialized
 * @returns A new Blob4844Tx instance
 *
 * @throws {EthereumJSErrorWithoutCode} If KZG is not initialized in Common
 * @throws {EthereumJSErrorWithoutCode} If both blobsData and blobs are provided
 *
 * Notes:
 * - Requires a Common instance with `customCrypto.kzg` initialized
 * - Cannot provide both `blobsData` and `blobs` simultaneously
 * - If `blobs` or `blobsData` is provided, `kzgCommitments`, `blobVersionedHashes`, and `kzgProofs` will be automatically derived
 * - KZG proof type depends on EIP-7594 activation: per-Blob proofs (EIP-4844) or per-Cell proofs (EIP-7594)
 */
export declare function createBlob4844Tx(txData: TxData, opts?: TxOptions): Blob4844Tx;
/**
 * Create a Blob4844Tx transaction from an array of byte encoded values ordered according to the devp2p network encoding.
 * Only canonical format supported, otherwise use `createBlob4844TxFromSerializedNetworkWrapper()`.
 *
 * @param values - Array of byte encoded values containing:
 *   - `chainId` - Chain ID as Uint8Array
 *   - `nonce` - Transaction nonce as Uint8Array
 *   - `maxPriorityFeePerGas` - Maximum priority fee per gas (EIP-1559) as Uint8Array
 *   - `maxFeePerGas` - Maximum fee per gas (EIP-1559) as Uint8Array
 *   - `gasLimit` - Gas limit for the transaction as Uint8Array
 *   - `to` - Recipient address as Uint8Array (optional for contract creation)
 *   - `value` - Value to transfer in wei as Uint8Array
 *   - `data` - Transaction data as Uint8Array
 *   - `accessList` - Access list for EIP-2930 as Uint8Array (optional)
 *   - `maxFeePerBlobGas` - Maximum fee per blob gas (EIP-4844) as Uint8Array
 *   - `blobVersionedHashes` - Versioned hashes for blob validation as Uint8Array[]
 *   - `v` - Signature recovery ID as Uint8Array (for signed transactions)
 *   - `r` - Signature r component as Uint8Array (for signed transactions)
 *   - `s` - Signature s component as Uint8Array (for signed transactions)
 * @param opts - Transaction options including Common instance with KZG initialized
 * @returns A new Blob4844Tx instance
 *
 * @throws {EthereumJSErrorWithoutCode} If KZG is not initialized in Common
 * @throws {EthereumJSErrorWithoutCode} If values array length is not 11 (unsigned) or 14 (signed)
 *
 * Format: `[chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
 * accessList, maxFeePerBlobGas, blobVersionedHashes, v, r, s]`
 *
 * Notes:
 * - Requires a Common instance with `customCrypto.kzg` initialized
 * - Supports both unsigned (11 values) and signed (14 values) transaction formats
 * - All numeric values must be provided as Uint8Array byte representations
 */
export declare function createBlob4844TxFromBytesArray(values: TxValuesArray, opts?: TxOptions): Blob4844Tx;
/**
 * Instantiate a Blob4844Tx transaction from an RLP serialized transaction.
 * Only canonical format supported, otherwise use `createBlob4844TxFromSerializedNetworkWrapper()`.
 *
 * @param serialized - RLP serialized transaction data as Uint8Array
 * @param opts - Transaction options including Common instance with KZG initialized
 * @returns A new Blob4844Tx instance
 *
 * @throws {EthereumJSErrorWithoutCode} If KZG is not initialized in Common
 * @throws {EthereumJSErrorWithoutCode} If serialized data is not a valid EIP-4844 transaction
 * @throws {EthereumJSErrorWithoutCode} If RLP decoded data is not an array
 *
 * Format: `0x03 || rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, to, value, data,
 * access_list, max_fee_per_blob_gas, blob_versioned_hashes, y_parity, r, s])`
 *
 * Notes:
 * - Requires a Common instance with `customCrypto.kzg` initialized
 * - Transaction type byte must be 0x03 (BlobEIP4844)
 * - RLP payload must decode to an array of transaction fields
 * - Delegates to `createBlob4844TxFromBytesArray` for actual construction
 */
export declare function createBlob4844TxFromRLP(serialized: Uint8Array, opts?: TxOptions): Blob4844Tx;
/**
 * Creates a Blob4844Tx transaction from the network encoding of a blob transaction wrapper.
 * This function handles the "Network Wrapper" format that includes blobs, commitments, and proofs.
 *
 * @param serialized - Serialized BlobTransactionNetworkWrapper as Uint8Array
 * @param opts - Transaction options including Common instance with KZG initialized
 * @returns A new Blob4844Tx instance with network wrapper data
 *
 * @throws {EthereumJSErrorWithoutCode} If Common instance is not provided
 * @throws {EthereumJSErrorWithoutCode} If KZG is not initialized in Common
 * @throws {EthereumJSErrorWithoutCode} If serialized data is not a valid EIP-4844 transaction
 * @throws {Error} If network wrapper has invalid number of values (not 4 or 5)
 * @throws {Error} If transaction has no valid `to` address
 * @throws {Error} If network wrapper version is invalid
 * @throws {EthereumJSErrorWithoutCode} If KZG verification fails
 * @throws {EthereumJSErrorWithoutCode} If versioned hashes don't match commitments
 *
 * Network Wrapper Formats:
 * - EIP-4844: `0x03 || rlp([tx_values, blobs, kzg_commitments, kzg_proofs])` (4 values)
 * - EIP-7594: `0x03 || rlp([tx_values, network_wrapper_version, blobs, kzg_commitments, kzg_proofs])` (5 values)
 *
 * Notes:
 * - Requires a Common instance with `customCrypto.kzg` initialized
 * - Validates KZG proofs against blobs and commitments
 * - Verifies versioned hashes match computed commitments
 * - Supports both EIP-4844 and EIP-7594 network wrapper formats
 * - Transaction is frozen by default unless `opts.freeze` is set to false
 */
export declare function createBlob4844TxFromSerializedNetworkWrapper(serialized: Uint8Array, opts?: TxOptions): Blob4844Tx;
/**
 * Creates the minimal representation of a blob transaction from the network wrapper version.
 * The minimal representation is used when adding transactions to an execution payload/block
 * @param txData a {@link Blob4844Tx} containing optional blobs/kzg commitments
 * @param opts - dictionary of {@link TxOptions}
 * @returns the "minimal" representation of a Blob4844Tx (i.e. transaction object minus blobs and kzg commitments)
 */
export declare function createMinimal4844TxFromNetworkWrapper(txData: Blob4844Tx, opts?: TxOptions): Blob4844Tx;
/**
 * Returns the EIP 4844 transaction network wrapper in JSON format similar to toJSON, including
 * blobs, commitments, and proofs fields
 * @param serialized a buffer representing a serialized BlobTransactionNetworkWrapper
 * @param opts any TxOptions defined
 * @returns JSONBlobTxNetworkWrapper with blobs, KZG commitments, and KZG proofs fields
 */
export declare function blobTxNetworkWrapperToJSON(serialized: Uint8Array, opts?: TxOptions): JSONBlobTxNetworkWrapper;
//# sourceMappingURL=constructors.d.ts.map