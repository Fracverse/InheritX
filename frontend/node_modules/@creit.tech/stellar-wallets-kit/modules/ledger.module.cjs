'use strict';

var Str = require('@ledgerhq/hw-app-str');
var TransportWebUSB = require('@ledgerhq/hw-transport-webusb');
var rxjs = require('rxjs');
var store = require('../state/store.cjs');
var types = require('../types.cjs');
var stellarBase = require('@stellar/stellar-base');
var utils = require('../utils.cjs');

const LEDGER_ID = 'LEDGER';
class LedgerModule {
    constructor() {
        this.moduleType = types.ModuleType.HW_WALLET;
        this.productId = LEDGER_ID;
        this.productName = 'Ledger';
        this.productUrl = 'https://www.ledger.com/';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/ledger.png';
    }
    async transport() {
        if (!(await TransportWebUSB.isSupported()))
            throw new Error('Ledger can not be used with this device.');
        if (!this._transport) {
            this._transport = await TransportWebUSB.create();
        }
        return this._transport;
    }
    async disconnect() {
        store.removeMnemonicPath();
        store.removeHardwareWalletPaths();
        this._transport?.close();
        this._transport = undefined;
    }
    /**
     * This always return true because in theory ledgers aren't supposed
     * to be connected at all time
     */
    async isAvailable() {
        return TransportWebUSB.isSupported();
    }
    async runChecks() {
        if (!(await this.isAvailable())) {
            throw new Error('Ledger wallets can not be used');
        }
    }
    async getAddress(opts) {
        await this.runChecks();
        try {
            const finalTransport = await this.transport();
            const str = new Str(finalTransport);
            let mnemonicPath = opts?.path || (await rxjs.firstValueFrom(store.mnemonicPath$));
            if (!mnemonicPath) {
                await this.openAccountSelector();
                mnemonicPath = await rxjs.firstValueFrom(store.mnemonicPath$);
            }
            const result = await str.getPublicKey(mnemonicPath);
            return { address: stellarBase.StrKey.encodeEd25519PublicKey(result.rawPublicKey) };
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
        const finalTransport = await this.transport();
        const str = new Str(finalTransport);
        const startIndex = page * 10;
        const results = [];
        for (let i = 0; i < 10; i++) {
            const result = await str.getPublicKey(`44'/148'/${i + startIndex}'`);
            results.push({
                publicKey: stellarBase.StrKey.encodeEd25519PublicKey(result.rawPublicKey),
                index: i + startIndex,
            });
        }
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
        const finalTransport = await this.transport();
        const str = new Str(finalTransport);
        let mnemonicPath;
        let account;
        if (opts?.path) {
            mnemonicPath = opts.path;
            const result = await str.getPublicKey(mnemonicPath);
            account = stellarBase.StrKey.encodeEd25519PublicKey(result.rawPublicKey);
        }
        else if (opts?.address) {
            const paths = await rxjs.firstValueFrom(store.hardwareWalletPaths$);
            const target = paths.find(p => p.publicKey === opts.address);
            if (!target)
                throw new Error('This address has not been loaded from this ledger');
            mnemonicPath = `44'/148'/${target.index}'`;
            account = target.publicKey;
        }
        else {
            mnemonicPath = await rxjs.firstValueFrom(store.mnemonicPath$);
            if (!mnemonicPath)
                throw new Error('There is no path available, please call the `getAddress` method first.');
            const result = await str.getPublicKey(mnemonicPath);
            account = stellarBase.StrKey.encodeEd25519PublicKey(result.rawPublicKey);
        }
        const network = opts?.networkPassphrase || (await rxjs.firstValueFrom(store.selectedNetwork$));
        if (!network)
            throw new Error('You need to provide or set a network passphrase');
        const tx = new stellarBase.Transaction(xdr, network);
        const result = opts?.nonBlindTx
            ? await str.signTransaction(mnemonicPath, tx.signatureBase())
            : await str.signHash(mnemonicPath, tx.hash());
        tx.addSignature(account, result.signature.toString('base64'));
        return {
            signedTxXdr: tx.toXDR(),
            signerAddress: account,
        };
    }
    async signAuthEntry() {
        throw {
            code: -3,
            message: 'Ledger Wallets do not support the "signAuthEntry" function',
        };
    }
    async signMessage() {
        throw {
            code: -3,
            message: 'Ledger Wallets do not support the "signMessage" function',
        };
    }
    async getNetwork() {
        throw {
            code: -3,
            message: 'Ledger Wallets do not support the "getNetwork" function',
        };
    }
}

exports.LEDGER_ID = LEDGER_ID;
exports.LedgerModule = LedgerModule;
//# sourceMappingURL=ledger.module.cjs.map
