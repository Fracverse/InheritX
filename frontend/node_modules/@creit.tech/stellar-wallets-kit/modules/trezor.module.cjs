'use strict';

var TrezorConnect = require('@trezor/connect-web');
var connectPluginStellar = require('@trezor/connect-plugin-stellar');
var stellarBase = require('@stellar/stellar-base');
var rxjs = require('rxjs');
var store = require('../state/store.cjs');
var types = require('../types.cjs');
var utils = require('../utils.cjs');

const TREZOR_ID = 'TREZOR';
class TrezorModule {
    constructor(params) {
        this.TrezorConnect = 'default' in TrezorConnect ? TrezorConnect.default : TrezorConnect;
        this._isAvailable = false;
        this.moduleType = types.ModuleType.HW_WALLET;
        this.productId = TREZOR_ID;
        this.productName = 'Trezor';
        this.productUrl = 'https://www.trezor.com/';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/trezor.png';
        this.TrezorConnect.init({
            manifest: {
                appName: params.appName,
                appUrl: params.appUrl,
                email: params.email,
            },
            // More advanced options
            debug: params.debug || false,
            lazyLoad: params.lazyLoad || false,
            coreMode: params.coreMode || 'auto',
        }).then(() => {
            console.log('Trezor is ready');
            this._isAvailable = true;
        });
    }
    async disconnect() {
        store.removeMnemonicPath();
        store.removeHardwareWalletPaths();
    }
    /**
     * `TrezorConnect` needs to be started before we can use it but because users most likely
     * won't use their devices as soon as the site loads, we return `true` since it should be already started
     * once the user needs to interact with it.
     */
    async isAvailable() {
        return true;
    }
    async runChecks() {
        if (!this._isAvailable) {
            throw utils.parseError(new Error('Trezor connection has not been started yet.'));
        }
    }
    async getAddress(opts) {
        await this.runChecks();
        try {
            const mnemonicPath = opts?.path || (await rxjs.firstValueFrom(store.mnemonicPath$));
            if (!mnemonicPath) {
                const result = await this.openAccountSelector();
                return { address: result.publicKey };
            }
            else {
                const result = await this.TrezorConnect.stellarGetAddress({ path: mnemonicPath, showOnTrezor: false });
                if (!result.success) {
                    throw utils.parseError(new Error(result.payload.error));
                }
                return { address: result.payload.address };
            }
        }
        catch (e) {
            throw utils.parseError(e);
        }
    }
    /**
     * This method is used by the Wallets Kit itself, if you're a dApp developer, most likely you don't need to use this method.
     * @param page - {Number}
     */
    async getAddresses(page = 0) {
        const startIndex = page * 10;
        const bundle = new Array(10)
            .fill(undefined)
            .map((_, i) => ({
            path: `m/44'/148'/${i + startIndex}'`,
            showOnTrezor: false,
        }));
        const result = await this.TrezorConnect.stellarGetAddress({ bundle });
        if (!result.success) {
            throw utils.parseError(new Error(result.payload.error));
        }
        const results = result.payload.map((item, i) => ({
            publicKey: item.address,
            index: i + startIndex,
        }));
        store.setHardwareWalletPaths(results);
        return results;
    }
    /**
     * This method is used by the Wallets Kit itself, if you're a dApp developer, most likely you don't need to use this method.
     */
    async openAccountSelector() {
        return new Promise((resolve, reject) => {
            const el = document.createElement('stellar-accounts-selector');
            el.setAttribute('showModal', '');
            el.setAttribute('loadingAccounts', '');
            document.body.appendChild(el);
            this.getAddresses()
                .then(addressesData => {
                el.setAttribute('accounts', JSON.stringify(addressesData));
                el.removeAttribute('loadingAccounts');
            })
                .catch(err => {
                el.remove();
                reject(err);
            });
            const listener = (event) => {
                const { publicKey, index } = event.detail;
                store.setMnemonicPath(`44'/148'/${index}'`);
                resolve({ publicKey, index });
                // eslint-disable-next-line @typescript-eslint/ban-ts-comment
                // @ts-ignore
                el.removeEventListener('account-selected', listener, false);
                document.body.removeChild(el);
            };
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            el.addEventListener('account-selected', listener, false);
            const errorListener = (event) => {
                // eslint-disable-next-line @typescript-eslint/ban-ts-comment
                // @ts-ignore
                el.removeEventListener('account-selected', listener, false);
                // eslint-disable-next-line @typescript-eslint/ban-ts-comment
                // @ts-ignore
                el.removeEventListener('account-selector-closed', errorListener, false);
                document.body.removeChild(el);
                reject(event.detail);
            };
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            el.addEventListener('account-selector-closed', errorListener, false);
        });
    }
    async signTransaction(xdr, opts) {
        await this.runChecks();
        let mnemonicPath;
        let account;
        if (opts?.path) {
            mnemonicPath = opts.path;
            const result = await this.TrezorConnect.stellarGetAddress({ path: mnemonicPath, showOnTrezor: false });
            if (!result.success) {
                throw new Error(result.payload.error);
            }
            account = result.payload.address;
        }
        else if (opts?.address) {
            const paths = await rxjs.firstValueFrom(store.hardwareWalletPaths$);
            const target = paths.find(p => p.publicKey === opts.address);
            if (!target)
                throw utils.parseError(new Error('This address has not been loaded from this device'));
            mnemonicPath = `m/44'/148'/${target.index}'`;
            account = target.publicKey;
        }
        else {
            mnemonicPath = await rxjs.firstValueFrom(store.mnemonicPath$);
            if (!mnemonicPath)
                throw utils.parseError(new Error('There is no path available, please call the `getAddress` method first.'));
            const result = await this.TrezorConnect.stellarGetAddress({ path: mnemonicPath, showOnTrezor: false });
            if (!result.success) {
                throw new Error(result.payload.error);
            }
            account = result.payload.address;
        }
        const network = opts?.networkPassphrase || (await rxjs.firstValueFrom(store.selectedNetwork$));
        if (!network)
            throw utils.parseError(new Error('You need to provide or set a network passphrase'));
        const tx = new stellarBase.Transaction(xdr, network);
        const parsedTx = connectPluginStellar.transformTransaction(mnemonicPath, tx);
        const result = await this.TrezorConnect.stellarSignTransaction(parsedTx);
        if (!result.success) {
            throw utils.parseError(new Error(result.payload.error));
        }
        tx.addSignature(account, Buffer.from(result.payload.signature, 'hex').toString('base64'));
        return {
            signedTxXdr: tx.toXDR(),
            signerAddress: account,
        };
    }
    async signAuthEntry() {
        throw {
            code: -3,
            message: 'Trezor Wallets do not support the "signAuthEntry" method',
        };
    }
    async signMessage() {
        throw {
            code: -3,
            message: 'Trezor Wallets do not support the "signMessage" method',
        };
    }
    async getNetwork() {
        throw {
            code: -3,
            message: 'Trezor Wallets do not support the "getNetwork" method',
        };
    }
}

exports.TREZOR_ID = TREZOR_ID;
exports.TrezorModule = TrezorModule;
//# sourceMappingURL=trezor.module.cjs.map
