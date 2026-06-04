'use strict';

var types = require('../types.cjs');
var utils = require('../utils.cjs');

const RABET_ID = 'rabet';
class RabetModule {
    constructor() {
        this.moduleType = types.ModuleType.HOT_WALLET;
        this.productId = RABET_ID;
        this.productName = 'Rabet';
        this.productUrl = 'https://rabet.io/';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/rabet.png';
    }
    isAvailable() {
        return new Promise(resolve => {
            // We wait 100ms before answering the call because Rabet is really slow when it comes to create the rabet window object and so this way we make sure is available
            setTimeout(() => {
                resolve(typeof window !== "undefined" && !!window.rabet);
            }, 100);
        });
    }
    async getAddress() {
        const runChecks = async () => {
            if (!(await this.isAvailable())) {
                throw new Error('Rabet is not installed');
            }
        };
        return runChecks()
            .then(() => window.rabet.connect())
            .then(({ publicKey }) => ({ address: publicKey }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signTransaction(xdr, opts) {
        const runChecks = async () => {
            if (!(await this.isAvailable())) {
                throw new Error('Rabet is not installed');
            }
            if (opts?.address &&
                opts.networkPassphrase !== types.WalletNetwork.PUBLIC &&
                opts.networkPassphrase !== types.WalletNetwork.TESTNET) {
                throw new Error(`Rabet doesn't support the network: ${opts.networkPassphrase}`);
            }
            if (opts?.address) {
                console.warn(`Rabet doesn't allow specifying the network that should be used, we skip the value`);
            }
        };
        const sign = async () => window.rabet.sign(xdr, opts?.networkPassphrase === types.WalletNetwork.PUBLIC ? exports.RabetNetwork.PUBLIC : exports.RabetNetwork.TESTNET);
        return runChecks()
            .then(sign)
            .then(result => ({ signedTxXdr: result?.xdr }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signAuthEntry() {
        throw {
            code: -3,
            message: 'Rabet does not support the "signAuthEntry" function',
        };
    }
    async signMessage() {
        throw {
            code: -3,
            message: 'Rabet does not support the "signMessage" function',
        };
    }
    async getNetwork() {
        throw {
            code: -3,
            message: 'Rabet does not support the "getNetwork" function',
        };
    }
}
exports.RabetNetwork = void 0;
(function (RabetNetwork) {
    RabetNetwork["PUBLIC"] = "mainnet";
    RabetNetwork["TESTNET"] = "testnet";
})(exports.RabetNetwork || (exports.RabetNetwork = {}));

exports.RABET_ID = RABET_ID;
exports.RabetModule = RabetModule;
//# sourceMappingURL=rabet.module.cjs.map
