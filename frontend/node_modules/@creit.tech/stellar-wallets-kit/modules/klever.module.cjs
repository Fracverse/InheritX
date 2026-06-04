'use strict';

var types = require('../types.cjs');
var utils = require('../utils.cjs');

const KLEVER_ID = 'klever';
class KleverModule {
    constructor() {
        this.moduleType = types.ModuleType.HOT_WALLET;
        this.productId = KLEVER_ID;
        this.productName = 'Klever Wallet';
        this.productUrl = 'https://klever.io/';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/klever.png';
    }
    async runChecks() {
        if (!(await this.isAvailable())) {
            throw new Error('Klever Wallet is not installed');
        }
    }
    isAvailable() {
        return new Promise(resolve => resolve(typeof window !== "undefined" && !!window.kleverWallet?.stellar));
    }
    async getAddress() {
        return this.runChecks()
            .then(() => window.kleverWallet.stellar.getAddress())
            .catch(e => { throw utils.parseError(e); });
    }
    async signTransaction(xdr, opts) {
        return this.runChecks()
            .then(() => window.kleverWallet.stellar.signTransaction(xdr, opts))
            .catch(e => { throw utils.parseError(e); });
    }
    async signAuthEntry(authEntry, opts) {
        return this.runChecks()
            .then(() => window.kleverWallet.stellar.signAuthEntry(authEntry, opts))
            .catch(e => { throw utils.parseError(e); });
    }
    async signMessage(message, opts) {
        return this.runChecks()
            .then(() => window.kleverWallet.stellar.signMessage(message, opts))
            .catch(e => { throw utils.parseError(e); });
    }
    async getNetwork() {
        return this.runChecks()
            .then(() => window.kleverWallet.stellar.getNetwork())
            .catch(e => { throw utils.parseError(e); });
    }
}

exports.KLEVER_ID = KLEVER_ID;
exports.KleverModule = KleverModule;
//# sourceMappingURL=klever.module.cjs.map
