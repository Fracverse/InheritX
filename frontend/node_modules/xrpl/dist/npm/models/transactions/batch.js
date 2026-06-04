"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateBatch = exports.BatchFlags = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const common_1 = require("./common");
var BatchFlags;
(function (BatchFlags) {
    BatchFlags[BatchFlags["tfAllOrNothing"] = 65536] = "tfAllOrNothing";
    BatchFlags[BatchFlags["tfOnlyOne"] = 131072] = "tfOnlyOne";
    BatchFlags[BatchFlags["tfUntilFailure"] = 262144] = "tfUntilFailure";
    BatchFlags[BatchFlags["tfIndependent"] = 524288] = "tfIndependent";
})(BatchFlags || (exports.BatchFlags = BatchFlags = {}));
function validateBatchInnerTransaction(tx, index) {
    if (tx.TransactionType === 'Batch') {
        throw new errors_1.ValidationError(`Batch: RawTransactions[${index}] is a Batch transaction. Cannot nest Batch transactions.`);
    }
    if (!(0, utils_1.hasFlag)(tx, common_1.GlobalFlags.tfInnerBatchTxn, 'tfInnerBatchTxn')) {
        throw new errors_1.ValidationError(`Batch: RawTransactions[${index}] must contain the \`tfInnerBatchTxn\` flag.`);
    }
    (0, common_1.validateOptionalField)(tx, 'Fee', (0, common_1.isValue)('0'), {
        paramName: `RawTransactions[${index}].RawTransaction.Fee`,
        txType: 'Batch',
    });
    (0, common_1.validateOptionalField)(tx, 'SigningPubKey', (0, common_1.isValue)(''), {
        paramName: `RawTransactions[${index}].RawTransaction.SigningPubKey`,
        txType: 'Batch',
    });
    (0, common_1.validateOptionalField)(tx, 'TxnSignature', common_1.isNull, {
        paramName: `RawTransactions[${index}].RawTransaction.TxnSignature`,
        txType: 'Batch',
    });
    (0, common_1.validateOptionalField)(tx, 'Signers', common_1.isNull, {
        paramName: `RawTransactions[${index}].RawTransaction.Signers`,
        txType: 'Batch',
    });
    (0, common_1.validateOptionalField)(tx, 'LastLedgerSequence', common_1.isNull, {
        paramName: `RawTransactions[${index}].RawTransaction.LastLedgerSequence`,
        txType: 'Batch',
    });
}
function validateBatch(tx) {
    var _a;
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'RawTransactions', common_1.isArray);
    tx.RawTransactions.forEach((rawTxObj, index) => {
        if (!(0, common_1.isRecord)(rawTxObj)) {
            throw new errors_1.ValidationError(`Batch: RawTransactions[${index}] is not object.`);
        }
        (0, common_1.validateRequiredField)(rawTxObj, 'RawTransaction', common_1.isRecord, {
            paramName: `RawTransactions[${index}].RawTransaction`,
            txType: 'Batch',
        });
        const rawTx = rawTxObj.RawTransaction;
        validateBatchInnerTransaction(rawTx, index);
    });
    (0, common_1.validateOptionalField)(tx, 'BatchSigners', common_1.isArray);
    (_a = tx.BatchSigners) === null || _a === void 0 ? void 0 : _a.forEach((signerObj, index) => {
        if (!(0, common_1.isRecord)(signerObj)) {
            throw new errors_1.ValidationError(`Batch: BatchSigners[${index}] is not object.`);
        }
        const signerRecord = signerObj;
        (0, common_1.validateRequiredField)(signerRecord, 'BatchSigner', common_1.isRecord, {
            paramName: `BatchSigners[${index}].BatchSigner`,
            txType: 'Batch',
        });
        const signer = signerRecord.BatchSigner;
        (0, common_1.validateRequiredField)(signer, 'Account', common_1.isString, {
            paramName: `BatchSigners[${index}].BatchSigner.Account`,
            txType: 'Batch',
        });
        (0, common_1.validateOptionalField)(signer, 'SigningPubKey', common_1.isString, {
            paramName: `BatchSigners[${index}].BatchSigner.SigningPubKey`,
            txType: 'Batch',
        });
        (0, common_1.validateOptionalField)(signer, 'TxnSignature', common_1.isString, {
            paramName: `BatchSigners[${index}].BatchSigner.TxnSignature`,
            txType: 'Batch',
        });
        (0, common_1.validateOptionalField)(signer, 'Signers', common_1.isArray, {
            paramName: `BatchSigners[${index}].BatchSigner.Signers`,
            txType: 'Batch',
        });
    });
}
exports.validateBatch = validateBatch;
//# sourceMappingURL=batch.js.map