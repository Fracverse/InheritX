"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.combineLoanSetCounterpartySigners = exports.signLoanSetByCounterparty = void 0;
const fast_json_stable_stringify_1 = __importDefault(require("fast-json-stable-stringify"));
const ripple_binary_codec_1 = require("ripple-binary-codec");
const errors_1 = require("../errors");
const models_1 = require("../models");
const hashes_1 = require("../utils/hashes");
const utils_1 = require("./utils");
function signLoanSetByCounterparty(wallet, transaction, opts = {}) {
    const tx = (0, utils_1.getDecodedTransaction)(transaction);
    if (tx.TransactionType !== 'LoanSet') {
        throw new errors_1.ValidationError('Transaction must be a LoanSet transaction.');
    }
    if (tx.CounterpartySignature) {
        throw new errors_1.ValidationError('Transaction is already signed by the counterparty.');
    }
    if (tx.TxnSignature == null || tx.SigningPubKey == null) {
        throw new errors_1.ValidationError('Transaction must be first signed by first party.');
    }
    (0, models_1.validate)(tx);
    let multisignAddress = false;
    if (typeof opts.multisign === 'string') {
        multisignAddress = opts.multisign;
    }
    else if (opts.multisign) {
        multisignAddress = wallet.classicAddress;
    }
    if (multisignAddress) {
        tx.CounterpartySignature = {
            Signers: [
                {
                    Signer: {
                        Account: multisignAddress,
                        SigningPubKey: wallet.publicKey,
                        TxnSignature: (0, utils_1.computeSignature)(tx, wallet.privateKey, multisignAddress),
                    },
                },
            ],
        };
    }
    else {
        tx.CounterpartySignature = {
            SigningPubKey: wallet.publicKey,
            TxnSignature: (0, utils_1.computeSignature)(tx, wallet.privateKey),
        };
    }
    const serialized = (0, ripple_binary_codec_1.encode)(tx);
    return {
        tx,
        tx_blob: serialized,
        hash: (0, hashes_1.hashSignedTx)(serialized),
    };
}
exports.signLoanSetByCounterparty = signLoanSetByCounterparty;
function combineLoanSetCounterpartySigners(transactions) {
    if (transactions.length === 0) {
        throw new errors_1.ValidationError('There are 0 transactions to combine.');
    }
    const decodedTransactions = transactions.map((txOrBlob) => {
        return (0, utils_1.getDecodedTransaction)(txOrBlob);
    });
    decodedTransactions.forEach((tx) => {
        var _a;
        (0, models_1.validate)(tx);
        if (tx.TransactionType !== 'LoanSet') {
            throw new errors_1.ValidationError('Transaction must be a LoanSet transaction.');
        }
        if (((_a = tx.CounterpartySignature) === null || _a === void 0 ? void 0 : _a.Signers) == null ||
            tx.CounterpartySignature.Signers.length === 0) {
            throw new errors_1.ValidationError('CounterpartySignature must have Signers.');
        }
        if (tx.TxnSignature == null || tx.SigningPubKey == null) {
            throw new errors_1.ValidationError('Transaction must be first signed by first party.');
        }
    });
    const loanSetTransactions = decodedTransactions;
    validateLoanSetTransactionEquivalence(loanSetTransactions);
    const tx = getTransactionWithAllLoanSetCounterpartySigners(loanSetTransactions);
    return {
        tx,
        tx_blob: (0, ripple_binary_codec_1.encode)(tx),
    };
}
exports.combineLoanSetCounterpartySigners = combineLoanSetCounterpartySigners;
function validateLoanSetTransactionEquivalence(transactions) {
    const exampleTransaction = (0, fast_json_stable_stringify_1.default)(Object.assign(Object.assign({}, transactions[0]), { CounterpartySignature: Object.assign(Object.assign({}, transactions[0].CounterpartySignature), { Signers: null }) }));
    if (transactions.slice(1).some((tx) => (0, fast_json_stable_stringify_1.default)(Object.assign(Object.assign({}, tx), { CounterpartySignature: Object.assign(Object.assign({}, tx.CounterpartySignature), { Signers: null }) })) !== exampleTransaction)) {
        throw new errors_1.ValidationError('LoanSet transactions are not the same.');
    }
}
function getTransactionWithAllLoanSetCounterpartySigners(transactions) {
    const sortedSigners = transactions
        .flatMap((tx) => { var _a, _b; return (_b = (_a = tx.CounterpartySignature) === null || _a === void 0 ? void 0 : _a.Signers) !== null && _b !== void 0 ? _b : []; })
        .sort((signer1, signer2) => (0, utils_1.compareSigners)(signer1.Signer, signer2.Signer));
    return Object.assign(Object.assign({}, transactions[0]), { CounterpartySignature: { Signers: sortedSigners } });
}
//# sourceMappingURL=counterpartySigner.js.map