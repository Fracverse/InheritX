"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateAMMClawback = exports.AMMClawbackFlags = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
var AMMClawbackFlags;
(function (AMMClawbackFlags) {
    AMMClawbackFlags[AMMClawbackFlags["tfClawTwoAssets"] = 1] = "tfClawTwoAssets";
})(AMMClawbackFlags || (exports.AMMClawbackFlags = AMMClawbackFlags = {}));
function validateAMMClawback(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'Holder', common_1.isAccount);
    (0, common_1.validateRequiredField)(tx, 'Asset', common_1.isIssuedCurrency);
    const asset = tx.Asset;
    if (tx.Holder === asset.issuer) {
        throw new errors_1.ValidationError('AMMClawback: Holder and Asset.issuer must be distinct');
    }
    if (tx.Account !== asset.issuer) {
        throw new errors_1.ValidationError('AMMClawback: Account must be the same as Asset.issuer');
    }
    (0, common_1.validateRequiredField)(tx, 'Asset2', common_1.isIssuedCurrency);
    (0, common_1.validateOptionalField)(tx, 'Amount', common_1.isIssuedCurrencyAmount);
    if (tx.Amount != null) {
        if (tx.Amount.currency !== asset.currency) {
            throw new errors_1.ValidationError('AMMClawback: Amount.currency must match Asset.currency');
        }
        if (tx.Amount.issuer !== asset.issuer) {
            throw new errors_1.ValidationError('AMMClawback: Amount.issuer must match Amount.issuer');
        }
    }
}
exports.validateAMMClawback = validateAMMClawback;
//# sourceMappingURL=AMMClawback.js.map