"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateOracleDelete = void 0;
const common_1 = require("./common");
function validateOracleDelete(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'OracleDocumentID', common_1.isNumber);
}
exports.validateOracleDelete = validateOracleDelete;
//# sourceMappingURL=oracleDelete.js.map