"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createBlob4844Tx = createBlob4844Tx;
exports.createBlob4844TxFromBytesArray = createBlob4844TxFromBytesArray;
exports.createBlob4844TxFromRLP = createBlob4844TxFromRLP;
exports.createBlob4844TxFromSerializedNetworkWrapper = createBlob4844TxFromSerializedNetworkWrapper;
exports.createMinimal4844TxFromNetworkWrapper = createMinimal4844TxFromNetworkWrapper;
exports.blobTxNetworkWrapperToJSON = blobTxNetworkWrapperToJSON;
const rlp_1 = require("@ethereumjs/rlp");
const util_1 = require("@ethereumjs/util");
const params_ts_1 = require("../params.js");
const types_ts_1 = require("../types.js");
const access_ts_1 = require("../util/access.js");
const tx_ts_1 = require("./tx.js");
const internal_ts_1 = require("../util/internal.js");
const validateBlobTransactionNetworkWrapper = (networkWrapperVersion, blobVersionedHashes, blobs, commitments, kzgProofs, version, kzg) => {
    if (!(blobVersionedHashes.length === blobs.length && blobs.length === commitments.length)) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Number of blobVersionedHashes, blobs, and commitments not all equal');
    }
    if (blobVersionedHashes.length === 0) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid transaction with empty blobs');
    }
    let isValid;
    try {
        if (networkWrapperVersion === tx_ts_1.NetworkWrapperType.EIP4844) {
            isValid = kzg.verifyBlobProofBatch(blobs, commitments, kzgProofs);
        }
        else {
            const [cells, indices] = (0, util_1.blobsToCells)(kzg, blobs);
            // verifyCellKzgProofBatch expected dup commitments and indices corresponding with cells and proofs
            const dupCommitments = [];
            const dupIndices = [];
            for (let i = 0; i < blobs.length; i++) {
                for (let j = 0; j < util_1.CELLS_PER_EXT_BLOB; j++) {
                    dupCommitments.push(commitments[i]);
                    dupIndices.push(indices[j]);
                }
            }
            isValid = kzg.verifyCellKzgProofBatch(dupCommitments, dupIndices, cells, kzgProofs);
        }
    }
    catch (error) {
        throw (0, util_1.EthereumJSErrorWithoutCode)(`KZG verification of blobs fail with error=${error}`);
    }
    if (!isValid) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('KZG proof cannot be verified from blobs/commitments');
    }
    for (let x = 0; x < blobVersionedHashes.length; x++) {
        const computedVersionedHash = (0, util_1.computeVersionedHash)(commitments[x], version);
        if (computedVersionedHash !== blobVersionedHashes[x]) {
            throw (0, util_1.EthereumJSErrorWithoutCode)(`commitment for blob at index ${x} does not match versionedHash`);
        }
    }
};
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
function createBlob4844Tx(txData, opts) {
    if (opts?.common?.customCrypto?.kzg === undefined) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('A common object with customCrypto.kzg initialized required to instantiate a 4844 blob tx');
    }
    const kzg = opts.common.customCrypto.kzg;
    if (txData.blobsData !== undefined && txData.blobs !== undefined) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('cannot have both raw blobs data and encoded blobs in constructor');
    }
    if (txData.blobsData !== undefined || txData.blobs !== undefined) {
        txData.blobs ?? (txData.blobs = (0, util_1.getBlobs)(txData.blobsData.reduce((acc, cur) => acc + cur)));
        txData.kzgCommitments ?? (txData.kzgCommitments = (0, util_1.blobsToCommitments)(kzg, txData.blobs));
        txData.blobVersionedHashes ?? (txData.blobVersionedHashes = (0, util_1.commitmentsToVersionedHashes)(txData.kzgCommitments));
        if (opts.common.isActivatedEIP(7594)) {
            txData.kzgProofs ?? (txData.kzgProofs = (0, util_1.blobsToCellProofs)(kzg, txData.blobs));
        }
        else {
            txData.kzgProofs ?? (txData.kzgProofs = (0, util_1.blobsToProofs)(kzg, txData.blobs, txData.kzgCommitments));
        }
    }
    return new tx_ts_1.Blob4844Tx(txData, opts);
}
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
function createBlob4844TxFromBytesArray(values, opts = {}) {
    if (opts.common?.customCrypto?.kzg === undefined) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('A common object with customCrypto.kzg initialized required to instantiate a 4844 blob tx');
    }
    if (values.length !== 11 && values.length !== 14) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-4844 transaction. Only expecting 11 values (for unsigned tx) or 14 values (for signed tx).');
    }
    const [chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, maxFeePerBlobGas, blobVersionedHashes, v, r, s,] = values;
    (0, internal_ts_1.validateNotArray)({ chainId, v });
    (0, util_1.validateNoLeadingZeroes)({
        nonce,
        maxPriorityFeePerGas,
        maxFeePerGas,
        gasLimit,
        value,
        maxFeePerBlobGas,
        v,
        r,
        s,
    });
    return new tx_ts_1.Blob4844Tx({
        chainId: (0, util_1.bytesToBigInt)(chainId),
        nonce,
        maxPriorityFeePerGas,
        maxFeePerGas,
        gasLimit,
        to,
        value,
        data,
        accessList: accessList ?? [],
        maxFeePerBlobGas,
        blobVersionedHashes,
        v: v !== undefined ? (0, util_1.bytesToBigInt)(v) : undefined, // EIP2930 supports v's with value 0 (empty Uint8Array)
        r,
        s,
    }, opts);
}
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
function createBlob4844TxFromRLP(serialized, opts = {}) {
    if (opts.common?.customCrypto?.kzg === undefined) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('A common object with customCrypto.kzg initialized required to instantiate a 4844 blob tx');
    }
    if ((0, util_1.equalsBytes)(serialized.subarray(0, 1), (0, internal_ts_1.txTypeBytes)(types_ts_1.TransactionType.BlobEIP4844)) === false) {
        throw (0, util_1.EthereumJSErrorWithoutCode)(`Invalid serialized tx input: not an EIP-4844 transaction (wrong tx type, expected: ${types_ts_1.TransactionType.BlobEIP4844}, received: ${(0, util_1.bytesToHex)(serialized.subarray(0, 1))}`);
    }
    const values = rlp_1.RLP.decode(serialized.subarray(1));
    if (!Array.isArray(values)) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid serialized tx input: must be array');
    }
    return createBlob4844TxFromBytesArray(values, opts);
}
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
function createBlob4844TxFromSerializedNetworkWrapper(serialized, opts) {
    if (!opts || !opts.common) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('common instance required to validate versioned hashes');
    }
    if (opts.common?.customCrypto?.kzg === undefined) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('A common object with customCrypto.kzg initialized required to instantiate a 4844 blob tx');
    }
    if ((0, util_1.equalsBytes)(serialized.subarray(0, 1), (0, internal_ts_1.txTypeBytes)(types_ts_1.TransactionType.BlobEIP4844)) === false) {
        throw (0, util_1.EthereumJSErrorWithoutCode)(`Invalid serialized tx input: not an EIP-4844 transaction (wrong tx type, expected: ${types_ts_1.TransactionType.BlobEIP4844}, received: ${(0, util_1.bytesToHex)(serialized.subarray(0, 1))}`);
    }
    // Validate network wrapper
    const networkTxValues = rlp_1.RLP.decode(serialized.subarray(1));
    let txValues, blobs, kzgCommitments, kzgProofs, networkWrapperVersion;
    if (networkTxValues.length === 4) {
        ;
        [txValues, blobs, kzgCommitments, kzgProofs] = networkTxValues;
        networkWrapperVersion = Uint8Array.from([tx_ts_1.NetworkWrapperType.EIP4844]);
    }
    else if (networkTxValues.length === 5) {
        ;
        [txValues, networkWrapperVersion, blobs, kzgCommitments, kzgProofs] =
            networkTxValues;
    }
    else {
        throw Error(`Expected 4 or 5 values in the deserialized network transaction`);
    }
    // Construct the tx but don't freeze yet, we will assign blobs etc once validated
    const decodedTx = createBlob4844TxFromBytesArray(txValues, { ...opts, freeze: false });
    if (decodedTx.to === undefined) {
        throw Error('Blob4844Tx can not be send without a valid `to`');
    }
    const commonCopy = opts.common.copy();
    commonCopy.updateParams(opts.params ?? params_ts_1.paramsTx);
    const version = Number(commonCopy.param('blobCommitmentVersionKzg'));
    const blobsHex = blobs.map((blob) => (0, util_1.bytesToHex)(blob));
    const commsHex = kzgCommitments.map((com) => (0, util_1.bytesToHex)(com));
    const proofsHex = kzgProofs.map((proof) => (0, util_1.bytesToHex)(proof));
    const networkWrapperVersionInt = (0, util_1.bytesToInt)(networkWrapperVersion);
    if (networkWrapperVersionInt !== tx_ts_1.NetworkWrapperType.EIP4844 &&
        networkWrapperVersionInt !== tx_ts_1.NetworkWrapperType.EIP7594) {
        throw Error(`Invalid networkWrapperVersion=${networkWrapperVersionInt}`);
    }
    validateBlobTransactionNetworkWrapper(networkWrapperVersionInt, decodedTx.blobVersionedHashes, blobsHex, commsHex, proofsHex, version, opts.common.customCrypto.kzg);
    // set the network blob data on the tx
    decodedTx.networkWrapperVersion = networkWrapperVersionInt;
    decodedTx.blobs = blobsHex;
    decodedTx.kzgCommitments = commsHex;
    decodedTx.kzgProofs = proofsHex;
    // freeze the tx
    const freeze = opts?.freeze ?? true;
    if (freeze) {
        Object.freeze(decodedTx);
    }
    return decodedTx;
}
/**
 * Creates the minimal representation of a blob transaction from the network wrapper version.
 * The minimal representation is used when adding transactions to an execution payload/block
 * @param txData a {@link Blob4844Tx} containing optional blobs/kzg commitments
 * @param opts - dictionary of {@link TxOptions}
 * @returns the "minimal" representation of a Blob4844Tx (i.e. transaction object minus blobs and kzg commitments)
 */
function createMinimal4844TxFromNetworkWrapper(txData, opts) {
    if (opts?.common?.customCrypto?.kzg === undefined) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('A common object with customCrypto.kzg initialized required to instantiate a 4844 blob tx');
    }
    const tx = createBlob4844Tx({
        ...txData,
        ...{
            networkWrapperVersion: undefined,
            blobs: undefined,
            kzgCommitments: undefined,
            kzgProofs: undefined,
        },
    }, opts);
    return tx;
}
/**
 * Returns the EIP 4844 transaction network wrapper in JSON format similar to toJSON, including
 * blobs, commitments, and proofs fields
 * @param serialized a buffer representing a serialized BlobTransactionNetworkWrapper
 * @param opts any TxOptions defined
 * @returns JSONBlobTxNetworkWrapper with blobs, KZG commitments, and KZG proofs fields
 */
function blobTxNetworkWrapperToJSON(serialized, opts) {
    const tx = createBlob4844TxFromSerializedNetworkWrapper(serialized, opts);
    const accessListJSON = (0, access_ts_1.accessListBytesToJSON)(tx.accessList);
    const baseJSON = tx.toJSON();
    return {
        ...baseJSON,
        chainId: (0, util_1.bigIntToHex)(tx.chainId),
        maxPriorityFeePerGas: (0, util_1.bigIntToHex)(tx.maxPriorityFeePerGas),
        maxFeePerGas: (0, util_1.bigIntToHex)(tx.maxFeePerGas),
        accessList: accessListJSON,
        maxFeePerBlobGas: (0, util_1.bigIntToHex)(tx.maxFeePerBlobGas),
        blobVersionedHashes: tx.blobVersionedHashes,
        networkWrapperVersion: (0, util_1.intToHex)(tx.networkWrapperVersion),
        blobs: tx.blobs,
        kzgCommitments: tx.kzgCommitments,
        kzgProofs: tx.kzgProofs,
    };
}
//# sourceMappingURL=constructors.js.map