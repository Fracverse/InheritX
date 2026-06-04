"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.EOACode7702Tx = void 0;
const util_1 = require("@ethereumjs/util");
const EIP1559 = require("../capabilities/eip1559.js");
const EIP2718 = require("../capabilities/eip2718.js");
const EIP2930 = require("../capabilities/eip2930.js");
const EIP7702 = require("../capabilities/eip7702.js");
const Legacy = require("../capabilities/legacy.js");
const types_ts_1 = require("../types.js");
const internal_ts_1 = require("../util/internal.js");
const constructors_ts_1 = require("./constructors.js");
const access_ts_1 = require("../util/access.js");
/**
 * Typed transaction with the ability to set codes on EOA accounts
 *
 * - TransactionType: 4
 * - EIP: [EIP-7702](https://github.com/ethereum/EIPs/blob/62419ca3f45375db00b04a368ea37c0bfb05386a/EIPS/eip-7702.md)
 */
class EOACode7702Tx {
    /**
     * This constructor takes the values, validates them, assigns them and freezes the object.
     *
     * It is not recommended to use this constructor directly. Instead use
     * the static factory methods to assist in creating a Transaction object from
     * varying data types.
     */
    constructor(txData, opts = {}) {
        this.type = types_ts_1.TransactionType.EOACodeEIP7702; // 7702 tx type
        this.cache = {};
        /**
         * List of tx type defining EIPs,
         * e.g. 1559 (fee market) and 2930 (access lists)
         * for FeeMarket1559Tx objects
         */
        this.activeCapabilities = [];
        (0, internal_ts_1.sharedConstructor)(this, { ...txData, type: types_ts_1.TransactionType.EOACodeEIP7702 }, opts);
        const { chainId, accessList: rawAccessList, authorizationList: rawAuthorizationList, maxFeePerGas, maxPriorityFeePerGas, } = txData;
        const accessList = rawAccessList ?? [];
        const authorizationList = rawAuthorizationList ?? [];
        if (chainId !== undefined && (0, util_1.bytesToBigInt)((0, util_1.toBytes)(chainId)) !== this.common.chainId()) {
            throw (0, util_1.EthereumJSErrorWithoutCode)(`Common chain ID ${this.common.chainId} not matching the derived chain ID ${chainId}`);
        }
        this.chainId = this.common.chainId();
        if (!this.common.isActivatedEIP(7702)) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('EIP-7702 not enabled on Common');
        }
        this.activeCapabilities = this.activeCapabilities.concat([1559, 2718, 2930, 7702]);
        // Populate the access list fields
        this.accessList = (0, types_ts_1.isAccessList)(accessList) ? (0, access_ts_1.accessListJSONToBytes)(accessList) : accessList;
        // Verify the access list format.
        EIP2930.verifyAccessList(this);
        // Populate the authority list fields
        this.authorizationList = (0, util_1.isEOACode7702AuthorizationList)(authorizationList)
            ? authorizationList.map((item) => (0, util_1.eoaCode7702AuthorizationListJSONItemToBytes)(item))
            : authorizationList;
        // Verify the authority list format.
        EIP7702.verifyAuthorizationList(this);
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
        EIP2718.validateYParity(this);
        Legacy.validateHighS(this);
        if (this.to === undefined) {
            const msg = Legacy.errorMsg(this, `tx should have a "to" field and cannot be used to create contracts`);
            throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
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
     * The amount of gas paid for the data in this tx
     */
    getDataGas() {
        return EIP7702.getDataGas(this);
    }
    /**
     * Returns the minimum of calculated priority fee (from maxFeePerGas and baseFee) and maxPriorityFeePerGas
     * @param baseFee Base fee retrieved from block
     */
    getEffectivePriorityFee(baseFee) {
        return EIP1559.getEffectivePriorityFee(this, baseFee);
    }
    /**
     * The up front amount that an account must have for this transaction to be valid
     * @param baseFee The base fee of the block (will be set to 0 if not provided)
     */
    getUpfrontCost(baseFee = util_1.BIGINT_0) {
        return EIP1559.getUpfrontCost(this, baseFee);
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
     * EOACode7702Tx cannot create contracts
     */
    toCreationAddress() {
        throw (0, util_1.EthereumJSErrorWithoutCode)('EOACode7702Tx cannot create contracts');
    }
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
            this.authorizationList,
            this.v !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.v) : new Uint8Array(0),
            this.r !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.r) : new Uint8Array(0),
            this.s !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.s) : new Uint8Array(0),
        ];
    }
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
    serialize() {
        return EIP2718.serialize(this);
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
        return EIP2718.serialize(this, this.raw().slice(0, 10));
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
     * Use {@link EOACode7702Transaction.getMessageToSign} to get a tx hash for the purpose of signing.
     * @returns Hash of the serialized signed transaction
     */
    hash() {
        return Legacy.hash(this);
    }
    /**
     * Computes a sha3-256 hash which can be used to verify the signature
     * @returns Hash used when verifying the signature
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
     * Adds the provided signature values and returns a new transaction instance.
     * @param v - Recovery parameter
     * @param r - Signature `r` value
     * @param s - Signature `s` value
     * @returns New `EOACode7702Tx` that includes the signature
     */
    addSignature(v, r, s) {
        r = (0, util_1.toBytes)(r);
        s = (0, util_1.toBytes)(s);
        const opts = { ...this.txOptions, common: this.common };
        return (0, constructors_ts_1.createEOACode7702Tx)({
            chainId: this.chainId,
            nonce: this.nonce,
            maxPriorityFeePerGas: this.maxPriorityFeePerGas,
            maxFeePerGas: this.maxFeePerGas,
            gasLimit: this.gasLimit,
            to: this.to,
            value: this.value,
            data: this.data,
            accessList: this.accessList,
            authorizationList: this.authorizationList,
            v,
            r: (0, util_1.bytesToBigInt)(r),
            s: (0, util_1.bytesToBigInt)(s),
        }, opts);
    }
    /**
     * Returns an object with the JSON representation of the transaction
     * @returns JSON encoding of the transaction
     */
    toJSON() {
        const accessListJSON = (0, access_ts_1.accessListBytesToJSON)(this.accessList);
        const authorizationList = this.authorizationList.map((item) => (0, util_1.eoaCode7702AuthorizationListBytesItemToJSON)(item));
        const baseJSON = (0, internal_ts_1.getBaseJSON)(this);
        return {
            ...baseJSON,
            chainId: (0, util_1.bigIntToHex)(this.chainId),
            maxPriorityFeePerGas: (0, util_1.bigIntToHex)(this.maxPriorityFeePerGas),
            maxFeePerGas: (0, util_1.bigIntToHex)(this.maxFeePerGas),
            accessList: accessListJSON,
            authorizationList,
        };
    }
    /**
     * Returns the list of validation errors, if any.
     * @returns Array of validation error messages
     */
    getValidationErrors() {
        return Legacy.getValidationErrors(this);
    }
    /**
     * @returns true if the transaction has no validation issues
     */
    isValid() {
        return Legacy.isValid(this);
    }
    /**
     * Verifies the embedded signature.
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
     * Signs the transaction and returns the signed instance.
     * @param privateKey - 32-byte private key
     * @param extraEntropy - Optional entropy supplied to the signing routine
     * @returns Newly signed transaction
     */
    sign(privateKey, extraEntropy = false) {
        return Legacy.sign(this, privateKey, extraEntropy);
    }
    /**
     * Indicates whether the transaction already carries signature data.
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
     */
    errorStr() {
        let errorStr = Legacy.getSharedErrorPostfix(this);
        errorStr += ` maxFeePerGas=${this.maxFeePerGas} maxPriorityFeePerGas=${this.maxPriorityFeePerGas}`;
        return errorStr;
    }
}
exports.EOACode7702Tx = EOACode7702Tx;
//# sourceMappingURL=tx.js.map