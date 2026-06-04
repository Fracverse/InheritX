"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateNFTokenCreateOffer = exports.NFTokenCreateOfferFlags = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const common_1 = require("./common");
var NFTokenCreateOfferFlags;
(function (NFTokenCreateOfferFlags) {
    NFTokenCreateOfferFlags[NFTokenCreateOfferFlags["tfSellNFToken"] = 1] = "tfSellNFToken";
})(NFTokenCreateOfferFlags || (exports.NFTokenCreateOfferFlags = NFTokenCreateOfferFlags = {}));
function validateNFTokenSellOfferCases(tx) {
    if (tx.Owner != null) {
        throw new errors_1.ValidationError('NFTokenCreateOffer: Owner must not be present for sell offers');
    }
}
function validateNFTokenBuyOfferCases(tx) {
    if (tx.Owner == null) {
        throw new errors_1.ValidationError('NFTokenCreateOffer: Owner must be present for buy offers');
    }
    if ((0, common_1.parseAmountValue)(tx.Amount) <= 0) {
        throw new errors_1.ValidationError('NFTokenCreateOffer: Amount must be greater than 0 for buy offers');
    }
}
function validateNFTokenCreateOffer(tx) {
    var _a, _b;
    (0, common_1.validateBaseTransaction)(tx);
    if (tx.Account === tx.Owner) {
        throw new errors_1.ValidationError('NFTokenCreateOffer: Owner and Account must not be equal');
    }
    if (tx.Account === tx.Destination) {
        throw new errors_1.ValidationError('NFTokenCreateOffer: Destination and Account must not be equal');
    }
    (0, common_1.validateOptionalField)(tx, 'Destination', common_1.isAccount);
    (0, common_1.validateOptionalField)(tx, 'Owner', common_1.isAccount);
    if (tx.NFTokenID == null) {
        throw new errors_1.ValidationError('NFTokenCreateOffer: missing field NFTokenID');
    }
    if (!(0, common_1.isAmount)(tx.Amount)) {
        throw new errors_1.ValidationError('NFTokenCreateOffer: invalid Amount');
    }
    const flags = ((_a = tx.Flags) !== null && _a !== void 0 ? _a : 0);
    const isTfSellNFToken = typeof flags === 'number'
        ? (0, utils_1.isFlagEnabled)(flags, NFTokenCreateOfferFlags.tfSellNFToken)
        : ((_b = flags.tfSellNFToken) !== null && _b !== void 0 ? _b : false);
    if (isTfSellNFToken) {
        validateNFTokenSellOfferCases(tx);
    }
    else {
        validateNFTokenBuyOfferCases(tx);
    }
}
exports.validateNFTokenCreateOffer = validateNFTokenCreateOffer;
//# sourceMappingURL=NFTokenCreateOffer.js.map