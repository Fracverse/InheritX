"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isVersionedTransaction = isVersionedTransaction;
function isVersionedTransaction(transaction) {
    return 'version' in transaction;
}
//# sourceMappingURL=transaction.js.map