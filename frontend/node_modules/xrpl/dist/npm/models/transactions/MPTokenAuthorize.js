"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateMPTokenAuthorize = exports.MPTokenAuthorizeFlags = void 0;
const common_1 = require("./common");
var MPTokenAuthorizeFlags;
(function (MPTokenAuthorizeFlags) {
    MPTokenAuthorizeFlags[MPTokenAuthorizeFlags["tfMPTUnauthorize"] = 1] = "tfMPTUnauthorize";
})(MPTokenAuthorizeFlags || (exports.MPTokenAuthorizeFlags = MPTokenAuthorizeFlags = {}));
function validateMPTokenAuthorize(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'MPTokenIssuanceID', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Holder', common_1.isAccount);
}
exports.validateMPTokenAuthorize = validateMPTokenAuthorize;
//# sourceMappingURL=MPTokenAuthorize.js.map