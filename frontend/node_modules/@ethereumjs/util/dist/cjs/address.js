"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Address = void 0;
exports.createZeroAddress = createZeroAddress;
exports.createAddressFromBigInt = createAddressFromBigInt;
exports.createAddressFromString = createAddressFromString;
exports.createAddressFromPublicKey = createAddressFromPublicKey;
exports.createAddressFromPrivateKey = createAddressFromPrivateKey;
exports.createContractAddress = createContractAddress;
exports.createContractAddress2 = createContractAddress2;
const account_ts_1 = require("./account.js");
const bytes_ts_1 = require("./bytes.js");
const constants_ts_1 = require("./constants.js");
const errors_ts_1 = require("./errors.js");
/**
 * Handling and generating Ethereum addresses
 */
class Address {
    constructor(bytes) {
        if (bytes.length !== 20) {
            throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Invalid address length');
        }
        this.bytes = bytes;
    }
    /**
     * Is address equal to another.
     */
    equals(address) {
        return (0, bytes_ts_1.equalsBytes)(this.bytes, address.bytes);
    }
    /**
     * Is address zero.
     */
    isZero() {
        return this.equals(new Address(new Uint8Array(20)));
    }
    /**
     * True if address is in the address range defined
     * by EIP-1352
     */
    isPrecompileOrSystemAddress() {
        const address = (0, bytes_ts_1.bytesToBigInt)(this.bytes);
        const rangeMin = constants_ts_1.BIGINT_0;
        const rangeMax = BigInt('0xffff');
        return address >= rangeMin && address <= rangeMax;
    }
    /**
     * Returns hex encoding of address.
     */
    toString() {
        return (0, bytes_ts_1.bytesToHex)(this.bytes);
    }
    /**
     * Returns a new Uint8Array representation of address.
     */
    toBytes() {
        return new Uint8Array(this.bytes);
    }
}
exports.Address = Address;
/**
 * Returns the zero address.
 */
function createZeroAddress() {
    return new Address(new Uint8Array(20));
}
/**
 * Returns an Address object from a bigint address (they are stored as bigints on the stack)
 * @param value The bigint address
 */
function createAddressFromBigInt(value) {
    const bytes = (0, bytes_ts_1.bigIntToBytes)(value);
    if (bytes.length > 20) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`Invalid address, too long: ${bytes.length}`);
    }
    return new Address((0, bytes_ts_1.setLengthLeft)(bytes, 20));
}
/**
 * Returns an Address object from a hex-encoded string.
 * @param str - Hex-encoded address
 */
function createAddressFromString(str) {
    if (!(0, account_ts_1.isValidAddress)(str)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`Invalid address input=${str}`);
    }
    return new Address((0, bytes_ts_1.hexToBytes)(str));
}
/**
 * Returns an address for a given public key.
 * @param pubKey The two points of an uncompressed key
 */
function createAddressFromPublicKey(pubKey) {
    if (!(pubKey instanceof Uint8Array)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Public key should be Uint8Array');
    }
    const bytes = (0, account_ts_1.pubToAddress)(pubKey);
    return new Address(bytes);
}
/**
 * Returns an address for a given private key.
 * @param privateKey A private key must be 256 bits wide
 */
function createAddressFromPrivateKey(privateKey) {
    if (!(privateKey instanceof Uint8Array)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Private key should be Uint8Array');
    }
    const bytes = (0, account_ts_1.privateToAddress)(privateKey);
    return new Address(bytes);
}
/**
 * Generates an address for a newly created contract.
 * @param from The address which is creating this new address
 * @param nonce The nonce of the from account
 */
function createContractAddress(from, nonce) {
    if (typeof nonce !== 'bigint') {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Expected nonce to be a bigint');
    }
    return new Address((0, account_ts_1.generateAddress)(from.bytes, (0, bytes_ts_1.bigIntToBytes)(nonce)));
}
/**
 * Generates an address for a contract created using CREATE2.
 * @param from The address which is creating this new address
 * @param salt A salt
 * @param initCode The init code of the contract being created
 */
function createContractAddress2(from, salt, initCode) {
    if (!(salt instanceof Uint8Array)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Expected salt to be a Uint8Array');
    }
    if (!(initCode instanceof Uint8Array)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Expected initCode to be a Uint8Array');
    }
    return new Address((0, account_ts_1.generateAddress2)(from.bytes, salt, initCode));
}
//# sourceMappingURL=address.js.map