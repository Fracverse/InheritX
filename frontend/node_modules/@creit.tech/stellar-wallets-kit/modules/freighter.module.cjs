'use strict';

var freighterApi = require('@stellar/freighter-api');
var types = require('../types.cjs');
var utils = require('../utils.cjs');
var buffer = require('buffer');

const FREIGHTER_ID = 'freighter';
class FreighterModule {
    constructor() {
        this.moduleType = types.ModuleType.HOT_WALLET;
        this.productId = FREIGHTER_ID;
        this.productName = 'Freighter';
        this.productUrl = 'https://freighter.app';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/freighter.png';
    }
    async runChecks() {
        if (!(await this.isAvailable())) {
            throw new Error('Freighter is not connected');
        }
    }
    async isAvailable() {
        // If these values are set it means we are loading the module from the Freighter's mobile version and so we need to
        // use WalletConnect instead.
        if (window.stellar?.provider === 'freighter' && window.stellar?.platform === 'mobile')
            return false;
        return freighterApi.isConnected()
            .then(({ isConnected, error }) => !error && isConnected)
            .catch(() => false);
    }
    async getAddress(params) {
        return this.runChecks()
            .then(async () => {
            if (params?.skipRequestAccess)
                return true;
            return freighterApi.requestAccess();
        })
            .then(() => freighterApi.getAddress())
            .then(({ address, error }) => {
            if (error)
                throw error;
            if (!address)
                throw {
                    code: -3,
                    message: 'Getting the address is not allowed, please request access first.',
                };
            return { address };
        })
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signTransaction(xdr, opts) {
        return this.runChecks()
            .then(async () => {
            const { signedTxXdr, signerAddress, error } = await freighterApi.signTransaction(xdr, {
                address: opts?.address,
                networkPassphrase: opts?.networkPassphrase,
            });
            if (error)
                throw error;
            return { signedTxXdr, signerAddress: signerAddress };
        })
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signAuthEntry(authEntry, opts) {
        return this.runChecks()
            .then(async () => {
            const { signedAuthEntry, signerAddress, error } = await freighterApi.signAuthEntry(authEntry, {
                address: opts?.address,
                networkPassphrase: opts?.networkPassphrase,
            });
            if (error || !signedAuthEntry)
                throw error;
            return { signedAuthEntry: buffer.Buffer.from(signedAuthEntry).toString('base64'), signerAddress: signerAddress };
        })
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signMessage(message, opts) {
        return this.runChecks()
            .then(async () => {
            const { signedMessage, signerAddress, error } = await freighterApi.signMessage(message, {
                address: opts?.address,
                networkPassphrase: opts?.networkPassphrase,
            });
            if (error || !signedMessage)
                throw error;
            return {
                signedMessage: typeof signedMessage === 'string'
                    ? signedMessage
                    : buffer.Buffer.from(signedMessage).toString('base64'),
                signerAddress: signerAddress
            };
        })
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async getNetwork() {
        return this.runChecks()
            .then(async () => {
            const { network, networkPassphrase, error } = await freighterApi.getNetwork();
            if (error)
                throw error;
            return { network, networkPassphrase };
        })
            .catch(e => {
            throw utils.parseError(e);
        });
    }
}

exports.FREIGHTER_ID = FREIGHTER_ID;
exports.FreighterModule = FreighterModule;
//# sourceMappingURL=freighter.module.cjs.map
