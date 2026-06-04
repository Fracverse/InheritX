'use strict';

var albedo = require('@albedo-link/intent');
var types = require('../types.cjs');
var utils = require('../utils.cjs');

const ALBEDO_ID = 'albedo';
class AlbedoModule {
    constructor() {
        this.moduleType = types.ModuleType.HOT_WALLET;
        this.productId = ALBEDO_ID;
        this.productName = 'Albedo';
        this.productUrl = 'https://albedo.link/';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/albedo.png';
    }
    async isAvailable() {
        return true;
    }
    async getAddress() {
        return albedo
            .publicKey({})
            .then(result => ({ address: result.pubkey }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signTransaction(xdr, opts) {
        return albedo
            .tx({
            xdr,
            pubkey: opts?.address,
            network: opts?.networkPassphrase
                ? opts.networkPassphrase === types.WalletNetwork.PUBLIC
                    ? exports.AlbedoNetwork.PUBLIC
                    : exports.AlbedoNetwork.TESTNET
                : undefined,
        })
            .then(({ signed_envelope_xdr }) => ({
            signedTxXdr: signed_envelope_xdr,
            signerAddress: opts?.address,
        }))
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signAuthEntry() {
        throw {
            code: -3,
            message: 'Albedo does not support the "signAuthEntry" function',
        };
    }
    /**
     * We understand that Albedo has a method to sign a message, but that method is not compatible with SEP-0043
     */
    async signMessage() {
        throw {
            code: -3,
            message: 'Albedo does not support the "signMessage" function',
        };
    }
    async getNetwork() {
        throw {
            code: -3,
            message: 'Albedo does not support the "getNetwork" function',
        };
    }
}
exports.AlbedoNetwork = void 0;
(function (AlbedoNetwork) {
    AlbedoNetwork["PUBLIC"] = "public";
    AlbedoNetwork["TESTNET"] = "testnet";
})(exports.AlbedoNetwork || (exports.AlbedoNetwork = {}));

exports.ALBEDO_ID = ALBEDO_ID;
exports.AlbedoModule = AlbedoModule;
//# sourceMappingURL=albedo.module.cjs.map
