"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateSignerListSet = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
const MAX_SIGNERS = 32;
const HEX_WALLET_LOCATOR_REGEX = /^[0-9A-Fa-f]{64}$/u;
function validateSignerListSet(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'SignerQuorum', common_1.isNumber);
    if (tx.SignerQuorum === 0) {
        return;
    }
    (0, common_1.validateRequiredField)(tx, 'SignerEntries', common_1.isArray);
    if (tx.SignerEntries.length === 0) {
        throw new errors_1.ValidationError('SignerListSet: need at least 1 member in SignerEntries');
    }
    if (tx.SignerEntries.length > MAX_SIGNERS) {
        throw new errors_1.ValidationError(`SignerListSet: maximum of ${MAX_SIGNERS} members allowed in SignerEntries`);
    }
    for (const entry of tx.SignerEntries) {
        if (!(0, common_1.isRecord)(entry) || !(0, common_1.isRecord)(entry.SignerEntry)) {
            throw new errors_1.ValidationError('SignerListSet: SignerEntries must be an array of SignerEntry objects');
        }
        const signerEntry = entry.SignerEntry;
        const { WalletLocator } = signerEntry;
        if (WalletLocator != null &&
            (!(0, common_1.isString)(WalletLocator) ||
                !HEX_WALLET_LOCATOR_REGEX.test(WalletLocator))) {
            throw new errors_1.ValidationError(`SignerListSet: WalletLocator in SignerEntry must be a 256-bit (32-byte) hexadecimal value`);
        }
    }
}
exports.validateSignerListSet = validateSignerListSet;
//# sourceMappingURL=signerListSet.js.map