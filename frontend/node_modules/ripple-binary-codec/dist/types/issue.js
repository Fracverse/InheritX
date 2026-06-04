"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Issue = void 0;
const utils_1 = require("@xrplf/isomorphic/utils");
const binary_parser_1 = require("../serdes/binary-parser");
const account_id_1 = require("./account-id");
const currency_1 = require("./currency");
const serialized_type_1 = require("./serialized-type");
const hash_192_1 = require("./hash-192");
const utils_2 = require("../utils");
/**
 * Type guard for Issue Object
 */
function isIssueObject(arg) {
    const keys = Object.keys(arg).sort();
    const isXRP = keys.length === 1 && keys[0] === 'currency';
    const isIOU = keys.length === 2 && keys[0] === 'currency' && keys[1] === 'issuer';
    const isMPT = keys.length === 1 && keys[0] === 'mpt_issuance_id';
    return isXRP || isIOU || isMPT;
}
const MPT_WIDTH = 44;
const NO_ACCOUNT = account_id_1.AccountID.from('0000000000000000000000000000000000000001');
/**
 * Class for serializing/Deserializing Issue
 */
class Issue extends serialized_type_1.SerializedType {
    constructor(bytes) {
        super(bytes !== null && bytes !== void 0 ? bytes : Issue.XRP_ISSUE.bytes);
    }
    /**
     * Construct Issue from XRPIssue, IOUIssue or MPTIssue
     *
     * @param value An object representing an XRPIssue, IOUIssue or MPTIssue
     * @returns An Issue object
     */
    static from(value) {
        if (value instanceof Issue) {
            return value;
        }
        if (isIssueObject(value)) {
            if (value.currency) {
                const currency = currency_1.Currency.from(value.currency.toString()).toBytes();
                //IOU case
                if (value.issuer) {
                    const issuer = account_id_1.AccountID.from(value.issuer.toString()).toBytes();
                    return new Issue((0, utils_1.concat)([currency, issuer]));
                }
                //XRP case
                return new Issue(currency);
            }
            // MPT case
            if (value.mpt_issuance_id) {
                const mptIssuanceIdBytes = hash_192_1.Hash192.from(value.mpt_issuance_id.toString()).toBytes();
                const issuerAccount = mptIssuanceIdBytes.slice(4);
                const sequence = Number((0, utils_2.readUInt32BE)(mptIssuanceIdBytes.slice(0, 4), 0)); // sequence is in Big-endian format in mpt_issuance_id
                // Convert to Little-endian
                const sequenceBuffer = new Uint8Array(4);
                new DataView(sequenceBuffer.buffer).setUint32(0, sequence, true);
                return new Issue((0, utils_1.concat)([issuerAccount, NO_ACCOUNT.toBytes(), sequenceBuffer]));
            }
        }
        throw new Error('Invalid type to construct an Issue');
    }
    /**
     * Read Issue from a BinaryParser
     *
     * @param parser BinaryParser to read the Issue from
     *
     * @returns An Issue object
     */
    static fromParser(parser) {
        // XRP
        const currencyOrAccount = parser.read(20);
        if (new currency_1.Currency(currencyOrAccount).toJSON() === 'XRP') {
            return new Issue(currencyOrAccount);
        }
        // MPT
        const issuerAccountId = new account_id_1.AccountID(parser.read(20));
        if (NO_ACCOUNT.toHex() === issuerAccountId.toHex()) {
            const sequence = parser.read(4);
            return new Issue((0, utils_1.concat)([currencyOrAccount, NO_ACCOUNT.toBytes(), sequence]));
        }
        // IOU
        return new Issue((0, utils_1.concat)([currencyOrAccount, issuerAccountId.toBytes()]));
    }
    /**
     * Get the JSON representation of this IssueObject
     *
     * @returns the JSON interpretation of this.bytes
     */
    toJSON() {
        // If the buffer is exactly 44 bytes, treat it as an MPTIssue.
        if (this.toBytes().length === MPT_WIDTH) {
            const issuerAccount = this.toBytes().slice(0, 20);
            const sequence = new DataView(this.toBytes().slice(40).buffer).getUint32(0, true);
            // sequence part of mpt_issuance_id should be in Big-endian
            const sequenceBuffer = new Uint8Array(4);
            (0, utils_2.writeUInt32BE)(sequenceBuffer, sequence, 0);
            return {
                mpt_issuance_id: (0, utils_1.bytesToHex)((0, utils_1.concat)([sequenceBuffer, issuerAccount])),
            };
        }
        const parser = new binary_parser_1.BinaryParser(this.toString());
        const currency = currency_1.Currency.fromParser(parser);
        if (currency.toJSON() === 'XRP') {
            return { currency: currency.toJSON() };
        }
        const issuer = account_id_1.AccountID.fromParser(parser);
        return {
            currency: currency.toJSON(),
            issuer: issuer.toJSON(),
        };
    }
}
exports.Issue = Issue;
Issue.XRP_ISSUE = new Issue(new Uint8Array(20));
//# sourceMappingURL=issue.js.map