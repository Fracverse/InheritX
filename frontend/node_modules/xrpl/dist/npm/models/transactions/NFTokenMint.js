"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateNFTokenMint = exports.NFTokenMintFlags = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const common_1 = require("./common");
var NFTokenMintFlags;
(function (NFTokenMintFlags) {
    NFTokenMintFlags[NFTokenMintFlags["tfBurnable"] = 1] = "tfBurnable";
    NFTokenMintFlags[NFTokenMintFlags["tfOnlyXRP"] = 2] = "tfOnlyXRP";
    NFTokenMintFlags[NFTokenMintFlags["tfTrustLine"] = 4] = "tfTrustLine";
    NFTokenMintFlags[NFTokenMintFlags["tfTransferable"] = 8] = "tfTransferable";
    NFTokenMintFlags[NFTokenMintFlags["tfMutable"] = 16] = "tfMutable";
})(NFTokenMintFlags || (exports.NFTokenMintFlags = NFTokenMintFlags = {}));
function validateNFTokenMint(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    if (tx.Account === tx.Issuer) {
        throw new errors_1.ValidationError('NFTokenMint: Issuer must not be equal to Account');
    }
    (0, common_1.validateOptionalField)(tx, 'Issuer', common_1.isAccount);
    if (typeof tx.URI === 'string' && tx.URI === '') {
        throw new errors_1.ValidationError('NFTokenMint: URI must not be empty string');
    }
    if (typeof tx.URI === 'string' && !(0, utils_1.isHex)(tx.URI)) {
        throw new errors_1.ValidationError('NFTokenMint: URI must be in hex format');
    }
    if (tx.NFTokenTaxon == null) {
        throw new errors_1.ValidationError('NFTokenMint: missing field NFTokenTaxon');
    }
    if (tx.Amount == null) {
        if (tx.Expiration != null || tx.Destination != null) {
            throw new errors_1.ValidationError('NFTokenMint: Amount is required when Expiration or Destination is present');
        }
    }
    (0, common_1.validateOptionalField)(tx, 'Amount', common_1.isAmount);
    (0, common_1.validateOptionalField)(tx, 'Expiration', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'Destination', common_1.isAccount);
}
exports.validateNFTokenMint = validateNFTokenMint;
//# sourceMappingURL=NFTokenMint.js.map