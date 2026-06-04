"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.LegacyTx = void 0;
const rlp_1 = require("@ethereumjs/rlp");
const util_1 = require("@ethereumjs/util");
const sha3_js_1 = require("@noble/hashes/sha3.js");
const Legacy = require("../capabilities/legacy.js");
const index_ts_1 = require("../index.js");
const types_ts_1 = require("../types.js");
const internal_ts_1 = require("../util/internal.js");
const constructors_ts_1 = require("./constructors.js");
function meetsEIP155(_v, chainId) {
    const v = Number(_v);
    const chainIdDoubled = Number(chainId) * 2;
    return v === chainIdDoubled + 35 || v === chainIdDoubled + 36;
}
/**
 * Validates tx's `v` value and extracts the chain id
 */
function validateVAndExtractChainID(common, _v) {
    let chainIdBigInt;
    const v = _v !== undefined ? Number(_v) : undefined;
    // Check for valid v values in the scope of a signed legacy tx
    if (v !== undefined) {
        // v is 1. not matching the EIP-155 chainId included case and...
        // v is 2. not matching the classic v=27 or v=28 case
        if (v < 37 && v !== 27 && v !== 28) {
            throw (0, util_1.EthereumJSErrorWithoutCode)(`Legacy txs need either v = 27/28 or v >= 37 (EIP-155 replay protection), got v = ${v}`);
        }
    }
    // No unsigned tx and EIP-155 activated and chain ID included
    if (v !== undefined && v !== 0 && common.gteHardfork('spuriousDragon') && v !== 27 && v !== 28) {
        if (!meetsEIP155(BigInt(v), common.chainId())) {
            throw (0, util_1.EthereumJSErrorWithoutCode)(`Incompatible EIP155-based V ${v} and chain id ${common.chainId()}. See the Common parameter of the Transaction constructor to set the chain id.`);
        }
        // Derive the original chain ID
        let numSub;
        if ((v - 35) % 2 === 0) {
            numSub = 35;
        }
        else {
            numSub = 36;
        }
        // Use derived chain ID to create a proper Common
        chainIdBigInt = BigInt(v - numSub) / util_1.BIGINT_2;
    }
    return chainIdBigInt;
}
/**
 * An Ethereum non-typed (legacy) transaction
 */
class LegacyTx {
    /**
     * This constructor takes the values, validates them, assigns them and freezes the object.
     *
     * It is not recommended to use this constructor directly. Instead use
     * the static factory methods to assist in creating a Transaction object from
     * varying data types.
     */
    constructor(txData, opts = {}) {
        /* Tx public data fields */
        this.type = types_ts_1.TransactionType.Legacy; // Legacy tx type
        this.cache = {};
        /**
         * List of tx type defining EIPs,
         * e.g. 1559 (fee market) and 2930 (access lists)
         * for FeeMarket1559Tx objects
         */
        this.activeCapabilities = [];
        (0, internal_ts_1.sharedConstructor)(this, txData, opts);
        this.gasPrice = (0, util_1.bytesToBigInt)((0, util_1.toBytes)(txData.gasPrice));
        (0, internal_ts_1.valueOverflowCheck)({ gasPrice: this.gasPrice });
        // Everything from BaseTransaction done here
        this.common.updateParams(opts.params ?? index_ts_1.paramsTx); // TODO should this move higher?
        const chainId = validateVAndExtractChainID(this.common, this.v);
        if (chainId !== undefined && chainId !== this.common.chainId()) {
            throw (0, util_1.EthereumJSErrorWithoutCode)(`Common chain ID ${this.common.chainId} not matching the derived chain ID ${chainId}`);
        }
        this.keccakFunction = this.common.customCrypto.keccak256 ?? sha3_js_1.keccak_256;
        if (this.gasPrice * this.gasLimit > util_1.MAX_INTEGER) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('gas limit * gasPrice cannot exceed MAX_INTEGER (2^256-1)');
        }
        if (this.common.gteHardfork('spuriousDragon')) {
            if (!this.isSigned()) {
                this.activeCapabilities.push(types_ts_1.Capability.EIP155ReplayProtection);
            }
            else {
                // EIP155 spec:
                // If block.number >= 2,675,000 and v = CHAIN_ID * 2 + 35 or v = CHAIN_ID * 2 + 36
                // then when computing the hash of a transaction for purposes of signing or recovering
                // instead of hashing only the first six elements (i.e. nonce, gasprice, startgas, to, value, data)
                // hash nine elements, with v replaced by CHAIN_ID, r = 0 and s = 0.
                // v and chain ID meet EIP-155 conditions
                if (meetsEIP155(this.v, this.common.chainId())) {
                    this.activeCapabilities.push(types_ts_1.Capability.EIP155ReplayProtection);
                }
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
     * Indicates whether the transaction already contains signature values.
     * @returns true if `v`, `r`, and `s` are populated
     */
    isSigned() {
        return Legacy.isSigned(this);
    }
    /**
     * Computes the effective priority fee for this legacy transaction, optionally considering a base fee.
     * @param baseFee - Optional base fee used on networks that emulate 1559-style pricing
     * @returns Priority fee portion denominated in wei
     */
    getEffectivePriorityFee(baseFee) {
        return Legacy.getEffectivePriorityFee(this.gasPrice, baseFee);
    }
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
    raw() {
        return [
            (0, util_1.bigIntToUnpaddedBytes)(this.nonce),
            (0, util_1.bigIntToUnpaddedBytes)(this.gasPrice),
            (0, util_1.bigIntToUnpaddedBytes)(this.gasLimit),
            this.to !== undefined ? this.to.bytes : new Uint8Array(0),
            (0, util_1.bigIntToUnpaddedBytes)(this.value),
            this.data,
            this.v !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.v) : new Uint8Array(0),
            this.r !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.r) : new Uint8Array(0),
            this.s !== undefined ? (0, util_1.bigIntToUnpaddedBytes)(this.s) : new Uint8Array(0),
        ];
    }
    /**
     * Returns the serialized encoding of the legacy transaction.
     *
     * Format: `rlp([nonce, gasPrice, gasLimit, to, value, data, v, r, s])`
     *
     * For an unsigned tx this method uses the empty Uint8Array values for the
     * signature parameters `v`, `r` and `s` for encoding. For an EIP-155 compliant
     * representation for external signing use {@link Transaction.getMessageToSign}.
     */
    serialize() {
        return rlp_1.RLP.encode(this.raw());
    }
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
    getMessageToSign() {
        const message = [
            (0, util_1.bigIntToUnpaddedBytes)(this.nonce),
            (0, util_1.bigIntToUnpaddedBytes)(this.gasPrice),
            (0, util_1.bigIntToUnpaddedBytes)(this.gasLimit),
            this.to !== undefined ? this.to.bytes : new Uint8Array(0),
            (0, util_1.bigIntToUnpaddedBytes)(this.value),
            this.data,
        ];
        if (this.supports(types_ts_1.Capability.EIP155ReplayProtection)) {
            message.push((0, util_1.bigIntToUnpaddedBytes)(this.common.chainId()));
            message.push((0, util_1.unpadBytes)((0, util_1.intToBytes)(0)));
            message.push((0, util_1.unpadBytes)((0, util_1.intToBytes)(0)));
        }
        return message;
    }
    /**
     * Returns the hashed serialized unsigned tx, which can be used
     * to sign the transaction (e.g. for sending to a hardware wallet).
     * @returns Hash of the unsigned transaction payload
     */
    getHashedMessageToSign() {
        const message = this.getMessageToSign();
        return this.keccakFunction(rlp_1.RLP.encode(message));
    }
    /**
     * The amount of gas paid for the data in this tx
     */
    getDataGas() {
        return Legacy.getDataGas(this);
    }
    // TODO figure out if this is necessary
    /**
     * If the tx's `to` is to the creation address
     */
    toCreationAddress() {
        return Legacy.toCreationAddress(this);
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
     * The up front amount that an account must have for this transaction to be valid
     */
    getUpfrontCost() {
        return this.gasLimit * this.gasPrice + this.value;
    }
    /**
     * Computes a sha3-256 hash of the serialized tx.
     *
     * This method can only be used for signed txs (it throws otherwise).
     * Use {@link Transaction.getMessageToSign} to get a tx hash for the purpose of signing.
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
        if (!this.isSigned()) {
            const msg = Legacy.errorMsg(this, 'This transaction is not signed');
            throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
        }
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
     * Adds a signature (or replaces an existing one) and returns a new transaction instance.
     * @param v - Recovery parameter, potentially unconverted when `convertV` is false
     * @param r - `r` value of the signature
     * @param s - `s` value of the signature
     * @param convertV - When true, converts the recovery ID into the appropriate legacy `v`
     * @returns A new `LegacyTx` that includes the provided signature
     */
    addSignature(v, r, s, 
    // convertV is `true` when called from `sign`
    // This is used to convert the `v` output from `ecsign` (0 or 1) to the values used for legacy txs:
    // 27 or 28 for non-EIP-155 protected txs
    // 35 or 36 + chainId * 2 for EIP-155 protected txs
    // See: https://eips.ethereum.org/EIPS/eip-155
    convertV = false) {
        r = (0, util_1.toBytes)(r);
        s = (0, util_1.toBytes)(s);
        if (convertV && this.supports(types_ts_1.Capability.EIP155ReplayProtection)) {
            v += BigInt(35) + this.common.chainId() * util_1.BIGINT_2;
        }
        else if (convertV) {
            v += BigInt(27);
        }
        const opts = { ...this.txOptions, common: this.common };
        return (0, constructors_ts_1.createLegacyTx)({
            nonce: this.nonce,
            gasPrice: this.gasPrice,
            gasLimit: this.gasLimit,
            to: this.to,
            value: this.value,
            data: this.data,
            v,
            r: (0, util_1.bytesToBigInt)(r),
            s: (0, util_1.bytesToBigInt)(s),
        }, opts);
    }
    /**
     * Returns an object with the JSON representation of the transaction.
     * @returns JSON encoding of the transaction
     */
    toJSON() {
        // TODO this is just copied. Make this execution-api compliant
        const baseJSON = (0, internal_ts_1.getBaseJSON)(this);
        baseJSON.gasPrice = (0, util_1.bigIntToHex)(this.gasPrice);
        return baseJSON;
    }
    /**
     * Validates the transaction and returns any encountered errors.
     * @returns Array containing validation error messages
     */
    getValidationErrors() {
        return Legacy.getValidationErrors(this);
    }
    /**
     * Determines whether the transaction passes all validation checks.
     * @returns true if no validation errors were found
     */
    isValid() {
        return Legacy.isValid(this);
    }
    /**
     * Checks whether the stored signature can be successfully verified.
     * @returns true if the signature is valid
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
     * Signs the transaction with the provided private key and returns the new signed instance.
     * @param privateKey - 32-byte private key used to sign the transaction
     * @param extraEntropy - Optional entropy passed to the signing routine
     * @returns A new signed `LegacyTx`
     */
    sign(privateKey, extraEntropy = false) {
        return Legacy.sign(this, privateKey, extraEntropy);
    }
    /**
     * Return a compact error string representation of the object
     * @returns Human-readable error summary
     */
    errorStr() {
        let errorStr = Legacy.getSharedErrorPostfix(this);
        errorStr += ` gasPrice=${this.gasPrice}`;
        return errorStr;
    }
}
exports.LegacyTx = LegacyTx;
//# sourceMappingURL=tx.js.map