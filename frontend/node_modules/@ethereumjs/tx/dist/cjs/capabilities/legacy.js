"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.errorMsg = errorMsg;
exports.isSigned = isSigned;
exports.getDataGas = getDataGas;
exports.getIntrinsicGas = getIntrinsicGas;
exports.toCreationAddress = toCreationAddress;
exports.hash = hash;
exports.validateHighS = validateHighS;
exports.getSenderPublicKey = getSenderPublicKey;
exports.getEffectivePriorityFee = getEffectivePriorityFee;
exports.getValidationErrors = getValidationErrors;
exports.isValid = isValid;
exports.verifySignature = verifySignature;
exports.getSenderAddress = getSenderAddress;
exports.sign = sign;
exports.getSharedErrorPostfix = getSharedErrorPostfix;
const util_1 = require("@ethereumjs/util");
const secp256k1_js_1 = require("@noble/curves/secp256k1.js");
const sha3_js_1 = require("@noble/hashes/sha3.js");
const types_ts_1 = require("../types.js");
/**
 * Creates an error message with transaction context
 * @param tx - The transaction interface
 * @param msg - The error message
 * @returns Formatted error message with transaction context
 */
function errorMsg(tx, msg) {
    return `${msg} (${tx.errorStr()})`;
}
/**
 * Checks if a transaction is signed
 * @param tx - The transaction interface
 * @returns true if the transaction is signed
 */
function isSigned(tx) {
    const { v, r, s } = tx;
    if (v === undefined || r === undefined || s === undefined) {
        return false;
    }
    else {
        return true;
    }
}
/**
 * The amount of gas paid for the data in this tx
 */
function getDataGas(tx) {
    if (tx.cache.dataFee && tx.cache.dataFee.hardfork === tx.common.hardfork()) {
        return tx.cache.dataFee.value;
    }
    const txDataZero = tx.common.param('txDataZeroGas');
    const txDataNonZero = tx.common.param('txDataNonZeroGas');
    let cost = util_1.BIGINT_0;
    for (let i = 0; i < tx.data.length; i++) {
        tx.data[i] === 0 ? (cost += txDataZero) : (cost += txDataNonZero);
    }
    if ((tx.to === undefined || tx.to === null) && tx.common.isActivatedEIP(3860)) {
        const dataLength = BigInt(Math.ceil(tx.data.length / 32));
        const initCodeCost = tx.common.param('initCodeWordGas') * dataLength;
        cost += initCodeCost;
    }
    if (Object.isFrozen(tx)) {
        tx.cache.dataFee = {
            value: cost,
            hardfork: tx.common.hardfork(),
        };
    }
    return cost;
}
/**
 * The minimum gas limit which the tx to have to be valid.
 * This covers costs as the standard fee (21000 gas), the data fee (paid for each calldata byte),
 * the optional creation fee (if the transaction creates a contract), and if relevant the gas
 * to be paid for access lists (EIP-2930) and authority lists (EIP-7702).
 */
function getIntrinsicGas(tx) {
    const txFee = tx.common.param('txGas');
    let fee = tx.getDataGas();
    if (txFee)
        fee += txFee;
    let isContractCreation = false;
    try {
        isContractCreation = tx.toCreationAddress();
    }
    catch {
        isContractCreation = false;
    }
    if (tx.common.gteHardfork('homestead') && isContractCreation) {
        const txCreationFee = tx.common.param('txCreationGas');
        if (txCreationFee)
            fee += txCreationFee;
    }
    return fee;
}
/**
 * Checks if the transaction targets the creation address (deploys a contract).
 * @param tx - Transaction interface to inspect
 * @returns true if the transaction's `to` is undefined or empty
 */
function toCreationAddress(tx) {
    return tx.to === undefined || tx.to.bytes.length === 0;
}
/**
 * Computes the keccak_256 hash of a signed legacy transaction.
 * @param tx - Transaction to hash
 * @returns Hash of the serialized transaction
 * @throws EthereumJSErrorWithoutCode if the transaction is unsigned
 */
function hash(tx) {
    var _a;
    if (!tx.isSigned()) {
        const msg = errorMsg(tx, 'Cannot call hash method if transaction is not signed');
        throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
    }
    const keccakFunction = tx.common.customCrypto.keccak256 ?? sha3_js_1.keccak_256;
    if (Object.isFrozen(tx)) {
        (_a = tx.cache).hash ?? (_a.hash = keccakFunction(tx.serialize()));
        return tx.cache.hash;
    }
    return keccakFunction(tx.serialize());
}
/**
 * EIP-2: All transaction signatures whose s-value is greater than secp256k1n/2are considered invalid.
 * Reasoning: https://ethereum.stackexchange.com/a/55728
 */
function validateHighS(tx) {
    const { s } = tx;
    if (tx.common.gteHardfork('homestead') && s !== undefined && s > util_1.SECP256K1_ORDER_DIV_2) {
        const msg = errorMsg(tx, 'Invalid Signature: s-values greater than secp256k1n/2 are considered invalid');
        throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
    }
}
/**
 * Recovers the sender's public key from the transaction signature.
 * @param tx - Transaction from which the public key should be derived
 * @returns The uncompressed sender public key
 * @throws EthereumJSErrorWithoutCode if the signature is invalid
 */
function getSenderPublicKey(tx) {
    if (tx.cache.senderPubKey !== undefined) {
        return tx.cache.senderPubKey;
    }
    const msgHash = tx.getMessageToVerifySignature();
    const { v, r, s } = tx;
    validateHighS(tx);
    try {
        const ecrecoverFunction = tx.common.customCrypto.ecrecover ?? util_1.ecrecover;
        const sender = ecrecoverFunction(msgHash, v, (0, util_1.bigIntToUnpaddedBytes)(r), (0, util_1.bigIntToUnpaddedBytes)(s), tx.supports(types_ts_1.Capability.EIP155ReplayProtection) ? tx.common.chainId() : undefined);
        if (Object.isFrozen(tx)) {
            tx.cache.senderPubKey = sender;
        }
        return sender;
    }
    catch {
        const msg = errorMsg(tx, 'Invalid Signature');
        throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
    }
}
/**
 * Calculates the effective priority fee for a legacy-style transaction.
 * @param gasPrice - Gas price specified on the transaction
 * @param baseFee - Optional base fee from the block when operating on L2s that mimic 1559 behavior
 * @returns The priority fee portion that can be paid to the block producer
 * @throws EthereumJSErrorWithoutCode if the base fee exceeds the gas price
 */
function getEffectivePriorityFee(gasPrice, baseFee) {
    if (baseFee !== undefined && baseFee > gasPrice) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Tx cannot pay baseFee');
    }
    if (baseFee === undefined) {
        return gasPrice;
    }
    return gasPrice - baseFee;
}
/**
 * Validates the transaction signature and minimum gas requirements.
 * @returns {string[]} an array of error strings
 */
function getValidationErrors(tx) {
    const errors = [];
    if (tx.isSigned() && !tx.verifySignature()) {
        errors.push('Invalid Signature');
    }
    let intrinsicGas = tx.getIntrinsicGas();
    if (tx.common.isActivatedEIP(7623)) {
        let tokens = 0;
        if (tx.common.isActivatedEIP(7976)) {
            // EIP-7976: uniform 4 tokens per byte regardless of zero/non-zero
            tokens = tx.data.length * 4;
        }
        else {
            for (let i = 0; i < tx.data.length; i++) {
                tokens += tx.data[i] === 0 ? 1 : 4;
            }
        }
        const floorCost = tx.common.param('txGas') + tx.common.param('totalCostFloorPerToken') * BigInt(tokens);
        intrinsicGas = (0, util_1.bigIntMax)(intrinsicGas, floorCost);
    }
    if (intrinsicGas > tx.gasLimit) {
        errors.push(`gasLimit is too low. The gasLimit is lower than the minimum gas limit of ${tx.getIntrinsicGas()}, the gas limit is: ${tx.gasLimit}`);
    }
    return errors;
}
/**
 * Validates the transaction signature and minimum gas requirements.
 * @returns {boolean} true if the transaction is valid, false otherwise
 */
function isValid(tx) {
    const errors = tx.getValidationErrors();
    return errors.length === 0;
}
/**
 * Determines if the signature is valid
 */
function verifySignature(tx) {
    try {
        // Main signature verification is done in `getSenderPublicKey()`
        const publicKey = tx.getSenderPublicKey();
        return (0, util_1.unpadBytes)(publicKey).length !== 0;
    }
    catch {
        return false;
    }
}
/**
 * Returns the sender's address
 */
function getSenderAddress(tx) {
    return new util_1.Address((0, util_1.publicToAddress)(tx.getSenderPublicKey()));
}
/**
 * Signs a transaction.
 *
 * Note that the signed tx is returned as a new object,
 * use as follows:
 * ```javascript
 * const signedTx = tx.sign(privateKey)
 * ```
 */
function sign(tx, privateKey, extraEntropy = true) {
    if (privateKey.length !== 32) {
        // TODO figure out this errorMsg logic how this diverges on other txs
        const msg = errorMsg(tx, 'Private key must be 32 bytes in length.');
        throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
    }
    // TODO (Jochem, 05 nov 2024): figure out what this hack does and clean it up
    // Hack for the constellation that we have got a legacy tx after spuriousDragon with a non-EIP155 conforming signature
    // and want to recreate a signature (where EIP155 should be applied)
    // Leaving this hack lets the legacy.spec.ts -> sign(), verifySignature() test fail
    // 2021-06-23
    let hackApplied = false;
    if (tx.type === types_ts_1.TransactionType.Legacy &&
        tx.common.gteHardfork('spuriousDragon') &&
        !tx.supports(types_ts_1.Capability.EIP155ReplayProtection)) {
        ;
        tx['activeCapabilities'].push(types_ts_1.Capability.EIP155ReplayProtection);
        hackApplied = true;
    }
    const msgHash = tx.getHashedMessageToSign();
    const ecSignFunction = tx.common.customCrypto?.ecsign ?? secp256k1_js_1.secp256k1.sign;
    const signatureBytes = ecSignFunction(msgHash, privateKey, {
        extraEntropy,
        format: 'recovered',
        prehash: false,
    });
    const { recovery, r, s } = secp256k1_js_1.secp256k1.Signature.fromBytes(signatureBytes, 'recovered');
    if (recovery === undefined) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid signature recovery');
    }
    const signedTx = tx.addSignature(BigInt(recovery), r, s, true);
    // Hack part 2
    if (hackApplied) {
        const index = tx['activeCapabilities'].indexOf(types_ts_1.Capability.EIP155ReplayProtection);
        if (index > -1) {
            ;
            tx['activeCapabilities'].splice(index, 1);
        }
    }
    return signedTx;
}
// TODO maybe move this to shared methods (util.ts in features)
/**
 * Builds a compact string that summarizes common transaction fields for error messages.
 * @param tx - Transaction used to assemble the postfix
 * @returns A formatted string containing tx type, hash, nonce, value, signature status, and hardfork
 */
function getSharedErrorPostfix(tx) {
    let hash = '';
    try {
        hash = tx.isSigned() ? (0, util_1.bytesToHex)(tx.hash()) : 'not available (unsigned)';
    }
    catch {
        hash = 'error';
    }
    let isSigned = '';
    try {
        isSigned = tx.isSigned().toString();
    }
    catch {
        hash = 'error';
    }
    let hf = '';
    try {
        hf = tx.common.hardfork();
    }
    catch {
        hf = 'error';
    }
    let postfix = `tx type=${tx.type} hash=${hash} nonce=${tx.nonce} value=${tx.value} `;
    postfix += `signed=${isSigned} hf=${hf}`;
    return postfix;
}
//# sourceMappingURL=legacy.js.map