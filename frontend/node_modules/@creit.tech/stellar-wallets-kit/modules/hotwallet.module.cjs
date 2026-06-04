'use strict';

var sdk = require('@hot-wallet/sdk');
var types = require('../types.cjs');

const HOTWALLET_ID = 'hot-wallet';
class HotWalletModule {
    constructor() {
        this.moduleType = types.ModuleType.HOT_WALLET;
        this.productId = HOTWALLET_ID;
        this.productName = 'HOT Wallet';
        this.productUrl = 'https://hot-labs.org/wallet';
        this.productIcon = 'https://storage.herewallet.app/logo.png';
    }
    async isAvailable() {
        return true;
    }
    async getAddress() {
        return await sdk.HOT.request('stellar:getAddress', {});
    }
    async signTransaction(xdr, opts) {
        return await sdk.HOT.request('stellar:signTransaction', { xdr, accountToSign: opts?.address });
    }
    async signAuthEntry(authEntry, opts) {
        return await sdk.HOT.request('stellar:signAuthEntry', { authEntry, accountToSign: opts?.address });
    }
    async signMessage(message, opts) {
        return await sdk.HOT.request('stellar:signMessage', { message, accountToSign: opts?.address });
    }
    async getNetwork() {
        return { network: 'mainnet', networkPassphrase: types.WalletNetwork.PUBLIC };
    }
}

exports.HOTWALLET_ID = HOTWALLET_ID;
exports.HotWalletModule = HotWalletModule;
//# sourceMappingURL=hotwallet.module.cjs.map
