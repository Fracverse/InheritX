'use strict';

var signerExtensionApi = require('@lobstrco/signer-extension-api');
var types = require('../types.cjs');
var utils = require('../utils.cjs');

const LOBSTR_ID = 'lobstr';
class LobstrModule {
    constructor() {
        this.moduleType = types.ModuleType.HOT_WALLET;
        this.productId = LOBSTR_ID;
        this.productName = 'LOBSTR';
        this.productUrl = 'https://lobstr.co';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/lobstr.png';
    }
    async isAvailable() {
        return signerExtensionApi.isConnected();
    }
    async getAddress() {
        const runChecks = async () => {
            if (!(await signerExtensionApi.isConnected())) {
                throw new Error(`Lobstr is not connected`);
            }
        };
        return runChecks()
            .then(() => signerExtensionApi.getPublicKey())
            .then(address => ({ address }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signTransaction(xdr, opts) {
        const runChecks = async () => {
            if (!(await signerExtensionApi.isConnected())) {
                throw new Error(`Lobstr is not connected`);
            }
            if (opts?.address) {
                console.warn(`Lobstr doesn't allow specifying what public key should sign the transaction, we skip the value`);
            }
            if (opts?.networkPassphrase) {
                console.warn(`Lobstr doesn't allow specifying the network that should be used, we skip the value`);
            }
        };
        return runChecks()
            .then(() => signerExtensionApi.signTransaction(xdr))
            .then(signedTxXdr => ({ signedTxXdr }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signAuthEntry() {
        throw {
            code: -3,
            message: 'Lobstr does not support the "signAuthEntry" function',
        };
    }
    async signMessage() {
        throw {
            code: -3,
            message: 'Lobstr does not support the "signMessage" function',
        };
    }
    async getNetwork() {
        throw {
            code: -3,
            message: 'Lobstr does not support the "getNetwork" function',
        };
    }
}

exports.LOBSTR_ID = LOBSTR_ID;
exports.LobstrModule = LobstrModule;
//# sourceMappingURL=lobstr.module.cjs.map
