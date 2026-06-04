'use strict';

var xbullWalletConnect = require('@creit.tech/xbull-wallet-connect');
var types = require('../types.cjs');
var utils = require('../utils.cjs');

const XBULL_ID = 'xbull';
class xBullModule {
    constructor() {
        this.moduleType = types.ModuleType.HOT_WALLET;
        this.productId = XBULL_ID;
        this.productName = 'xBull';
        this.productUrl = 'https://xbull.app';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/xbull.png';
    }
    async isAvailable() {
        return true;
    }
    async getAddress() {
        try {
            const bridge = new xbullWalletConnect.xBullWalletConnect();
            const publicKey = await bridge.connect();
            bridge.closeConnections();
            return { address: publicKey };
        }
        catch (e) {
            throw utils.parseError(e);
        }
    }
    async signTransaction(xdr, opts) {
        try {
            const bridge = new xbullWalletConnect.xBullWalletConnect();
            const signedXdr = await bridge.sign({
                xdr,
                publicKey: opts?.address,
                network: opts?.networkPassphrase,
            });
            bridge.closeConnections();
            return { signedTxXdr: signedXdr, signerAddress: opts?.address };
        }
        catch (e) {
            throw utils.parseError(e);
        }
    }
    async signAuthEntry() {
        throw {
            code: -3,
            message: 'xBull does not support the "signAuthEntry" function',
        };
    }
    async signMessage(message, opts) {
        try {
            const bridge = new xbullWalletConnect.xBullWalletConnect();
            const result = await bridge.signMessage(message, {
                address: opts?.address,
                networkPassphrase: opts?.networkPassphrase,
            });
            bridge.closeConnections();
            return result;
        }
        catch (e) {
            throw utils.parseError(e);
        }
    }
    async getNetwork() {
        throw {
            code: -3,
            message: 'xBull does not support the "getNetwork" function',
        };
    }
}

exports.XBULL_ID = XBULL_ID;
exports.xBullModule = xBullModule;
//# sourceMappingURL=xbull.module.cjs.map
