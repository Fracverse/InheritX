"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validatePermissionedDomainDelete = void 0;
const common_1 = require("./common");
function validatePermissionedDomainDelete(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'DomainID', common_1.isString);
}
exports.validatePermissionedDomainDelete = validatePermissionedDomainDelete;
//# sourceMappingURL=permissionedDomainDelete.js.map