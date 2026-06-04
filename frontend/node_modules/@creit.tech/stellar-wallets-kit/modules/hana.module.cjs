'use strict';

var types = require('../types.cjs');
var utils = require('../utils.cjs');

const HANA_ID = 'hana';
class HanaModule {
    constructor() {
        this.moduleType = types.ModuleType.HOT_WALLET;
        this.productId = HANA_ID;
        this.productName = 'Hana Wallet';
        this.productUrl = 'https://hanawallet.io/';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/hana.png';
    }
    async runChecks() {
        if (!(await this.isAvailable())) {
            throw new Error('Hana Wallet is not installed');
        }
    }
    isAvailable() {
        return new Promise(resolve => resolve(typeof window !== "undefined" && !!window.hanaWallet?.stellar));
    }
    async getAddress() {
        return this.runChecks()
            .then(() => window.hanaWallet.stellar.getPublicKey())
            .then(address => ({ address }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signTransaction(xdr, opts) {
        return this.runChecks()
            .then(() => window.hanaWallet.stellar.signTransaction({
            xdr,
            accountToSign: opts?.address,
            networkPassphrase: opts?.networkPassphrase,
        }))
            .then(signedTxXdr => ({ signedTxXdr, signerAddress: opts?.address }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signAuthEntry(authEntry, opts) {
        return this.runChecks()
            .then(() => window.hanaWallet.stellar.signAuthEntry({
            xdr: authEntry,
            accountToSign: opts?.address,
        }))
            .then(signedAuthEntry => ({ signedAuthEntry, signerAddress: opts?.address }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signMessage(message, opts) {
        return this.runChecks()
            .then(() => window.hanaWallet.stellar.signMessage({
            message,
            accountToSign: opts?.address,
        }))
            .then(signedMessage => ({ signedMessage, signerAddress: opts?.address }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async getNetwork() {
        throw {
            code: -3,
            message: 'Hana does not support the "getNetwork" function',
        };
    }
}

exports.HANA_ID = HANA_ID;
exports.HanaModule = HanaModule;
//# sourceMappingURL=hana.module.cjs.map
