"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateMPTokenIssuanceDestroy = void 0;
const common_1 = require("./common");
function validateMPTokenIssuanceDestroy(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'MPTokenIssuanceID', common_1.isString);
}
exports.validateMPTokenIssuanceDestroy = validateMPTokenIssuanceDestroy;
//# sourceMappingURL=MPTokenIssuanceDestroy.js.map