"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Blob4844Tx = exports.NetworkWrapperType = void 0;
const util_1 = require("@ethereumjs/util");
const EIP1559 = require("../capabilities/eip1559.js");
const EIP2718 = require("../capabilities/eip2718.js");
const EIP2930 = require("../capabilities/eip2930.js");
const Legacy = require("../capabilities/legacy.js");
const types_ts_1 = require("../types.js");
const access_ts_1 = require("../util/access.js");
const internal_ts_1 = require("../util/internal.js");
const constructors_ts_1 = require("./constructors.js");
exports.NetworkWrapperType = {
    EIP4844: 0,
    EIP7594: 1,
};
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
class Blob4844Tx {
    /**
     * This constructor takes the values, validates them, assigns them and freezes the object.
     *
     * It is not recommended to use this constructor directly. Instead use
     * the static constructors or factory methods to assist in creating a Transaction object from
     * varying data types.
     */
    constructor(txData, opts = {}) {
        this.type = types_ts_1.TransactionType.BlobEIP4844; // 4844 tx type
        this.cache = {};
        /**
         * List of tx type defining EIPs,
         * e.g. 1559 (fee market) and 2930 (access lists)
         * for FeeMarket1559Tx objects
         */
        this.activeCapabilities = [];
        // Check networkWrapperVersion early, before sharedConstructor, to ensure proper error ordering
        // This validation needs to happen before EIP-7825 gas limit checks
        const common = (0, internal_ts_1.getCommon)(opts.common);
        const networkWrapperVersion = txData.networkWrapperVersion !== undefined
            ? (0, util_1.bytesToInt)((0, util_1.toBytes)(txData.networkWrapperVersion))
            : undefined;
        if (networkWrapperVersion !== undefined) {
            switch (networkWrapperVersion) {
                case exports.NetworkWrapperType.EIP7594:
                    if (!common.isActivatedEIP(7594)) {
                        throw (0, util_1.EthereumJSErrorWithoutCode)('EIP-7594 not enabled on Common for EIP-7594 network wrapper version');
                    }
                    break;
                case exports.NetworkWrapperType.EIP4844:
                    if (common.isActivatedEIP(7594)) {
                        throw (0, util_1.EthereumJSErrorWithoutCode)('EIP-7594 is active on Common for EIP-4844 network wrapper version');
                    }
                    break;
                default:
                    throw (0, util_1.EthereumJSErrorWithoutCode)(`Invalid networkWrapperVersion=${networkWrapperVersion}`);
            }
        }
        (0, internal_ts_1.sharedConstructor)(this, { ...txData, type: types_ts_1.TransactionType.BlobEIP4844 }, opts);
        const { chainId, accessList: rawAccessList, maxFeePerGas, maxPriorityFeePerGas, maxFeePerBlobGas, } = txData;
        const accessList = rawAccessList ?? [];
        if (chainId !== undefined && (0, util_1.bytesToBigInt)((0, util_1.toBytes)(chainId)) !== this.common.chainId()) {
            throw (0, util_1.EthereumJSErrorWithoutCode)(`Common chain ID ${this.common.chainId} not matching the derived chain ID ${chainId}`);
        }
        this.chainId = this.common.chainId();
        if (!this.common.isActivatedEIP(1559)) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('EIP-1559 not enabled on Common');
        }
        if (!this.common.isActivatedEIP(4844)) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('EIP-4844 not enabled on Common');
        }
        this.activeCapabilities = this.activeCapabilities.concat([1559, 2718, 2930]);
        // Populate the access list fields
        this.accessList = (0, types_ts_1.isAccessList)(accessList) ? (0, access_ts_1.accessListJSONToBytes)(accessList) : accessList;
        // Verify the access list format.
        EIP2930.verifyAccessList(this);
        this.maxFeePerGas = (0, util_1.bytesToBigInt)((0, util_1.toBytes)(maxFeePerGas));
        this.maxPriorityFeePerGas = (0, util_1.bytesToBigInt)((0, util_1.toBytes)(maxPriorityFeePerGas));
        (0, internal_ts_1.valueOverflowCheck)({
            maxFeePerGas: this.maxFeePerGas,
            maxPriorityFeePerGas: this.maxPriorityFeePerGas,
        });
        (0, internal_ts_1.validateNotArray)(txData);
        if (this.gasLimit * this.maxFeePerGas > util_1.MAX_INTEGER) {
            const msg = Legacy.errorMsg(this, 'gasLimit * maxFeePerGas cannot exceed MAX_INTEGER (2^256-1)');
            throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
        }
        if (this.maxFeePerGas < this.maxPriorityFeePerGas) {
            const msg = Legacy.errorMsg(this, 'maxFeePerGas cannot be less than maxPriorityFeePerGas (The total must be the larger of the two)');
            throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
        }
        this.maxFeePerBlobGas = (0, util_1.bytesToBigInt)((0, util_1.toBytes)((maxFeePerBlobGas ?? '') === '' ? '0x' : maxFeePerBlobGas));
        this.blobVersionedHashes = (txData.blobVersionedHashes ?? []).map((vh) => (0, util_1.toType)(vh, util_1.TypeOutput.PrefixedHexString));
        EIP2718.validateYParity(this);
        Legacy.validateHighS(this);
        for (const hash of this.blobVersionedHashes) {
            if (hash.length !== 66) {
                // 66 is the length of a 32 byte hash as a PrefixedHexString
                const msg = Legacy.errorMsg(this, 'versioned hash is invalid length');
                throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
            }
            if (BigInt(parseInt(hash.slice(2, 4))) !== this.common.param('blobCommitmentVersionKzg')) {
                // We check the first "byte" of the hash (starts at position 2 since hash is a PrefixedHexString)
                const msg = Legacy.errorMsg(this, 'versioned hash does not start with KZG commitment version');
                throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
            }
        }
        // "Old" limit (superseded by EIP-7594 starting with Osaka)
        const limitBlobsPerTx = this.common.param('maxBlobGasPerBlock') / this.common.param('blobGasPerBlob');
        if (this.blobVersionedHashes.length > limitBlobsPerTx) {
            const msg = Legacy.errorMsg(this, `tx causes total blob gas of ${Number(this.common.param('blobGasPerBlob')) * this.blobVersionedHashes.length} to exceed maximum blob gas per block of ${this.common.param('maxBlobGasPerBlock')}`);
            throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
        }
        // EIP-7594 PeerDAS: Limit of 6 blobs per transaction
        if (this.common.isActivatedEIP(7594)) {
            const maxBlobsPerTx = this.common.param('maxBlobsPerTx');
            if (this.blobVersionedHashes.length > maxBlobsPerTx) {
                const msg = Legacy.errorMsg(this, `${this.blobVersionedHashes.length} blobs exceeds max ${maxBlobsPerTx} blobs per tx (EIP-7594)`);
                throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
            }
        }
        if (this.blobVersionedHashes.length === 0) {
            const msg = Legacy.errorMsg(this, `tx should contain at least one blob`);
            throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
        }
        if (this.to === undefined) {
            const msg = Legacy.errorMsg(this, `tx should have a "to" field and cannot be used to create contracts`);
            throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
        }
        // networkWrapperVersion was already validated earlier in the constructor
        this.networkWrapperVersion =
            txData.networkWrapperVersion !== undefined
                ? (0, util_1.bytesToInt)((0, util_1.toBytes)(txData.networkWrapperVersion))
                : undefined;
        this.blobs = txData.blobs?.map((blob) => (0, util_1.toType)(blob, util_1.TypeOutput.PrefixedHexString));
        if (this.networkWrapperVersion === undefined && this.blobs !== undefined) {
            if (this.common.isActivatedEIP(7594)) {
                this.networkWrapperVersion = 1;
            }
            else {
                this.networkWrapperVersion = 0;
            }
        }
        if (this.networkWrapperVersion !== undefined && this.blobs === undefined) {
            const msg = Legacy.errorMsg(this, 'tx is not allowed to be in network wrapper format if no blob list is provided');
            throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
        }
        this.kzgCommitments = txData.kzgCommitments?.map((commitment) => (0, util_1.toType)(commitment, util_1.TypeOutput.PrefixedHexString));
        this.kzgProofs = txData.kzgProofs?.map((proof) => (0, util_1.toType)(proof, util_1.TypeOutput.PrefixedHexString));
        if (this.blobs !== undefined) {
            if (this.kzgCommitments === undefined) {
                const msg = Legacy.errorMsg(this, 'kzgCommitments are mandatory if blobs are provided');
                throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
            }
            if (this.kzgProofs === undefined) {
                const msg = Legacy.errorMsg(this, 'kzgProofs are mandatory if blobs are provided');
                throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
            }
        }
        const freeze = opts?.freeze ?? true;
        if (freeze) {
            Object.freeze(this);
        }
    }
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
    supports(capability) {
        return this.activeCapabilities.includes(capability);
    }
    /**
     * Returns the minimum of calculated priority fee (from maxFeePerGas and baseFee) and maxPriorityFeePerGas
     * @param baseFee Base fee retrieved from block
     */
    getEffectivePriorityFee(baseFee) {
        return EIP1559.getEffectivePriorityFee(this, baseFee);
    }
    /**
     * The amount of gas paid for the data in this tx
     */
    getDataGas() {
        return EIP2930.getDataGas(this);
    }
    /**
     * The up front amount that an account must have for this transaction to be valid
     * @param baseFee The base fee of the block (will be set to 0 if not provided)
     */
    getUpfrontCost(baseFee = util_1.BIGINT_0) {
        return EIP1559.getUpfrontCost(this, baseFee);
    }
    /**
     * Blob4844Tx cannot create contracts
     */
    toCreationAddress() {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Blob4844Tx cannot create contracts');
    }
    /**
     * The minimum gas limit which the tx to have to be valid.
     * This covers costs as the standard fee (21000 gas), the data fee (paid for each calldata byte),
     * the optional creation fee (if the transaction creates a contract), and if relevant the gas
     * to be paid for access lists (EIP-2930) and authority lists (EIP-7702).
     */
    getIntrinsicGas() {
        return Legacy.getIntrinsicGas(this);
    }
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
    raw() {
        return [
            (0, util_1.bigIntToUnpaddedBytes)(this.chainId),
            (0, util_1.bigIntToUnpaddedBytes)(this.nonce),
            (0, util_1.bigIntToUnpaddedBytes)(this.maxPriorityFeePerGas),
            (0, util_1.bigIntToUnpaddedBytes)(this.maxFeePerGas),
            (0, util_1.bigIntToUnpaddedBytes)(this.gasLimit),
            this.to !== undefined ? this.to.bytes : new Uint8Array(0),
            (0, util_1.bigIntToUnpaddedBytes)(this.value),
            this.data,
            this.accessList,
            (0, util_1.bigIntToUnpaddedBytes)(this.maxFeePerBlobGas),
            this.blobVersionedHashes.map((hash) => (0, util_1.hexToBytes)(hash)),
            this.v !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.v) : new Uint8Array(0),
            this.r !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.r) : new Uint8Array(0),
            this.s !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.s) : new Uint8Array(0),
        ];
    }
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
    serialize() {
        return EIP2718.serialize(this);
    }
    /**
     * @returns the serialized form of a blob transaction in the network wrapper format
     * This format is used for gossipping mempool transactions over devp2p or when
     * submitting a transaction via RPC.
     */
    serializeNetworkWrapper() {
        if (this.networkWrapperVersion === undefined ||
            this.blobs === undefined ||
            this.kzgCommitments === undefined ||
            this.kzgProofs === undefined) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('cannot serialize network wrapper without networkWrapperVersion, blobs, KZG commitments and KZG proofs provided');
        }
        const networkSerialized = this.networkWrapperVersion === exports.NetworkWrapperType.EIP4844
            ? EIP2718.serialize(this, [this.raw(), this.blobs, this.kzgCommitments, this.kzgProofs])
            : EIP2718.serialize(this, [
                this.raw(),
                (0, util_1.intToUnpaddedBytes)(this.networkWrapperVersion),
                this.blobs,
                this.kzgCommitments,
                this.kzgProofs,
            ]);
        return networkSerialized;
    }
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
    getMessageToSign() {
        return EIP2718.serialize(this, this.raw().slice(0, 11));
    }
    /**
     * Returns the hashed serialized unsigned tx, which can be used
     * to sign the transaction (e.g. for sending to a hardware wallet).
     *
     * Note: in contrast to the legacy tx the raw message format is already
     * serialized and doesn't need to be RLP encoded any more.
     * @returns Keccak hash of the unsigned transaction payload
     */
    getHashedMessageToSign() {
        return EIP2718.getHashedMessageToSign(this);
    }
    /**
     * Computes a sha3-256 hash of the serialized tx.
     *
     * This method can only be used for signed txs (it throws otherwise).
     * Use {@link Blob4844Tx.getMessageToSign} to get a tx hash for the purpose of signing.
     * @returns Hash of the serialized signed transaction
     */
    hash() {
        return Legacy.hash(this);
    }
    /**
     * Returns the hashed unsigned transaction that should be used for signature verification.
     * @returns Hash of the unsigned transaction payload
     */
    getMessageToVerifySignature() {
        return this.getHashedMessageToSign();
    }
    /**
     * Returns the public key of the sender
     * @returns Sender public key
     */
    getSenderPublicKey() {
        return Legacy.getSenderPublicKey(this);
    }
    /**
     * Produces a JSON representation compliant with the execution API.
     * @returns JSON encoding of the transaction
     */
    toJSON() {
        const accessListJSON = (0, access_ts_1.accessListBytesToJSON)(this.accessList);
        const baseJSON = (0, internal_ts_1.getBaseJSON)(this);
        return {
            ...baseJSON,
            chainId: (0, util_1.bigIntToHex)(this.chainId),
            maxPriorityFeePerGas: (0, util_1.bigIntToHex)(this.maxPriorityFeePerGas),
            maxFeePerGas: (0, util_1.bigIntToHex)(this.maxFeePerGas),
            accessList: accessListJSON,
            maxFeePerBlobGas: (0, util_1.bigIntToHex)(this.maxFeePerBlobGas),
            blobVersionedHashes: this.blobVersionedHashes,
        };
    }
    /**
     * Adds signature values (and optional network wrapper fields) and returns a new transaction.
     * @param v - Recovery parameter
     * @param r - Signature `r` value
     * @param s - Signature `s` value
     * @returns New `Blob4844Tx` instance containing the signature
     */
    addSignature(v, r, s) {
        r = (0, util_1.toBytes)(r);
        s = (0, util_1.toBytes)(s);
        const opts = { ...this.txOptions, common: this.common };
        return (0, constructors_ts_1.createBlob4844Tx)({
            chainId: this.chainId,
            nonce: this.nonce,
            maxPriorityFeePerGas: this.maxPriorityFeePerGas,
            maxFeePerGas: this.maxFeePerGas,
            gasLimit: this.gasLimit,
            to: this.to,
            value: this.value,
            data: this.data,
            accessList: this.accessList,
            v,
            r: (0, util_1.bytesToBigInt)(r),
            s: (0, util_1.bytesToBigInt)(s),
            maxFeePerBlobGas: this.maxFeePerBlobGas,
            networkWrapperVersion: this.networkWrapperVersion,
            blobVersionedHashes: this.blobVersionedHashes,
            blobs: this.blobs,
            kzgCommitments: this.kzgCommitments,
            kzgProofs: this.kzgProofs,
        }, opts);
    }
    /**
     * Returns validation errors for this transaction, if any.
     * @returns Array of validation error messages
     */
    getValidationErrors() {
        return Legacy.getValidationErrors(this);
    }
    /**
     * @returns true if the transaction has no validation errors
     */
    isValid() {
        return Legacy.isValid(this);
    }
    /**
     * Verifies whether the attached signature is valid.
     * @returns true if signature verification succeeds
     */
    verifySignature() {
        return Legacy.verifySignature(this);
    }
    /**
     * Returns the recovered sender address.
     * @returns Sender {@link Address}
     */
    getSenderAddress() {
        return Legacy.getSenderAddress(this);
    }
    /**
     * Signs the transaction with the provided private key and returns the signed instance.
     * @param privateKey - 32-byte private key used for signing
     * @param extraEntropy - Optional entropy passed to the signing routine
     * @returns Newly signed transaction
     */
    sign(privateKey, extraEntropy = false) {
        return Legacy.sign(this, privateKey, extraEntropy);
    }
    /**
     * Indicates whether the transaction already carries signature values.
     * @returns true if signature parts are present
     */
    isSigned() {
        const { v, r, s } = this;
        if (v === undefined || r === undefined || s === undefined) {
            return false;
        }
        else {
            return true;
        }
    }
    /**
     * Return a compact error string representation of the object
     * @returns Human-readable error summary
     */
    errorStr() {
        let errorStr = Legacy.getSharedErrorPostfix(this);
        errorStr += ` maxFeePerGas=${this.maxFeePerGas} maxPriorityFeePerGas=${this.maxPriorityFeePerGas}`;
        return errorStr;
    }
    /**
     * @returns the number of blobs included with this transaction
     */
    numBlobs() {
        return this.blobVersionedHashes.length;
    }
}
exports.Blob4844Tx = Blob4844Tx;
//# sourceMappingURL=tx.js.map