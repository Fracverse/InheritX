"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.combineBatchSigners = exports.signMultiBatch = void 0;
const ripple_binary_codec_1 = require("ripple-binary-codec");
const ripple_keypairs_1 = require("ripple-keypairs");
const errors_1 = require("../errors");
const models_1 = require("../models");
const batch_1 = require("../models/transactions/batch");
const hashes_1 = require("../utils/hashes");
const utils_1 = require("./utils");
function constructBatchSignerObject(batchAccount, wallet, signature, multisignAddress = false) {
    let batchSigner;
    if (multisignAddress) {
        batchSigner = {
            BatchSigner: {
                Account: batchAccount,
                Signers: [
                    {
                        Signer: {
                            Account: multisignAddress,
                            SigningPubKey: wallet.publicKey,
                            TxnSignature: signature,
                        },
                    },
                ],
            },
        };
    }
    else {
        batchSigner = {
            BatchSigner: {
                Account: batchAccount,
                SigningPubKey: wallet.publicKey,
                TxnSignature: signature,
            },
        };
    }
    return batchSigner;
}
function signMultiBatch(wallet, transaction, opts = {}) {
    var _a;
    const batchAccount = (_a = opts.batchAccount) !== null && _a !== void 0 ? _a : wallet.classicAddress;
    let multisignAddress = false;
    if (typeof opts.multisign === 'string') {
        multisignAddress = opts.multisign;
    }
    else if (opts.multisign) {
        multisignAddress = wallet.classicAddress;
    }
    if (transaction.TransactionType !== 'Batch') {
        throw new errors_1.ValidationError('Must be a Batch transaction.');
    }
    (0, models_1.validate)(transaction);
    const involvedAccounts = new Set(transaction.RawTransactions.map((raw) => raw.RawTransaction.Account));
    if (!involvedAccounts.has(batchAccount)) {
        throw new errors_1.ValidationError('Must be signing for an address submitting a transaction in the Batch.');
    }
    const fieldsToSign = {
        flags: transaction.Flags,
        txIDs: transaction.RawTransactions.map((rawTx) => (0, hashes_1.hashSignedTx)(rawTx.RawTransaction)),
    };
    const signature = (0, ripple_keypairs_1.sign)((0, ripple_binary_codec_1.encodeForSigningBatch)(fieldsToSign), wallet.privateKey);
    transaction.BatchSigners = [
        constructBatchSignerObject(batchAccount, wallet, signature, multisignAddress),
    ];
}
exports.signMultiBatch = signMultiBatch;
function combineBatchSigners(transactions) {
    if (transactions.length === 0) {
        throw new errors_1.ValidationError('There are 0 transactions to combine.');
    }
    const decodedTransactions = transactions.map((txOrBlob) => {
        return (0, utils_1.getDecodedTransaction)(txOrBlob);
    });
    decodedTransactions.forEach((tx) => {
        if (tx.TransactionType !== 'Batch') {
            throw new errors_1.ValidationError('TransactionType must be `Batch`.');
        }
        (0, batch_1.validateBatch)(tx);
        if (tx.BatchSigners == null || tx.BatchSigners.length === 0) {
            throw new errors_1.ValidationError('For combining Batch transaction signatures, all transactions must include a BatchSigners field containing an array of signatures.');
        }
        if (tx.TxnSignature != null || tx.Signers != null) {
            throw new errors_1.ValidationError('Batch transaction must be unsigned.');
        }
    });
    const batchTransactions = decodedTransactions;
    validateBatchTransactionEquivalence(batchTransactions);
    return (0, ripple_binary_codec_1.encode)(getTransactionWithAllBatchSigners(batchTransactions));
}
exports.combineBatchSigners = combineBatchSigners;
function validateBatchTransactionEquivalence(transactions) {
    const exampleTransaction = JSON.stringify({
        flags: transactions[0].Flags,
        transactionIDs: transactions[0].RawTransactions.map((rawTx) => (0, hashes_1.hashSignedTx)(rawTx.RawTransaction)),
    });
    if (transactions.slice(1).some((tx) => JSON.stringify({
        flags: tx.Flags,
        transactionIDs: tx.RawTransactions.map((rawTx) => (0, hashes_1.hashSignedTx)(rawTx.RawTransaction)),
    }) !== exampleTransaction)) {
        throw new errors_1.ValidationError('Flags and transaction hashes are not the same for all provided transactions.');
    }
}
function getTransactionWithAllBatchSigners(transactions) {
    const sortedSigners = transactions
        .flatMap((tx) => { var _a; return (_a = tx.BatchSigners) !== null && _a !== void 0 ? _a : []; })
        .filter((signer) => signer.BatchSigner.Account !== transactions[0].Account)
        .sort((signer1, signer2) => (0, utils_1.compareSigners)(signer1.BatchSigner, signer2.BatchSigner));
    return Object.assign(Object.assign({}, transactions[0]), { BatchSigners: sortedSigners });
}
//# sourceMappingURL=batchSigner.js.map