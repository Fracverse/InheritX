"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateNFTokenModify = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const common_1 = require("./common");
function validateNFTokenModify(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'NFTokenID', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Owner', common_1.isAccount);
    (0, common_1.validateOptionalField)(tx, 'URI', common_1.isString);
    if (tx.URI !== undefined && typeof tx.URI === 'string') {
        if (tx.URI === '') {
            throw new errors_1.ValidationError('NFTokenModify: URI must not be empty string');
        }
        if (!(0, utils_1.isHex)(tx.URI)) {
            throw new errors_1.ValidationError('NFTokenModify: URI must be in hex format');
        }
    }
}
exports.validateNFTokenModify = validateNFTokenModify;
//# sourceMappingURL=NFTokenModify.js.map