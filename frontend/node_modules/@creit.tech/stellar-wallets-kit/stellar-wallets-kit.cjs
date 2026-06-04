'use strict';

var store = require('./state/store.cjs');

class StellarWalletsKit {
    get selectedModule() {
        const { selectedModuleId } = store.store.getValue();
        if (!selectedModuleId) {
            throw { code: -3, message: 'Please set the wallet first' };
        }
        const target = this.modules.find((mod) => mod.productId === selectedModuleId);
        if (!target) {
            throw { code: -3, message: 'Please set the wallet first' };
        }
        return target;
    }
    constructor(params) {
        this.modules = params.modules;
        if (params.selectedWalletId)
            this.setWallet(params.selectedWalletId);
        store.setNetwork(params.network);
        const modalTheme = params.theme || params.modalTheme;
        if (modalTheme) {
            store.setModalTheme(modalTheme);
        }
        if (params.buttonTheme) {
            store.seButtonTheme(params.buttonTheme);
        }
        this.getSupportedWallets().then((value) => {
            store.setAllowedWallets(value);
        });
    }
    /**
     * This method will return an array with all wallets supported by this kit but will let you know those the user have already installed/has access to
     * There are wallets that are by default available since they either don't need to be installed or have a fallback
     */
    async getSupportedWallets() {
        return Promise.all(this.modules.map(async (mod) => {
            const timer = new Promise(r => setTimeout(() => r(false), 500));
            return {
                id: mod.productId,
                name: mod.productName,
                type: mod.moduleType,
                icon: mod.productIcon,
                isAvailable: await Promise.race([timer, mod.isAvailable()]),
                isPlatformWrapper: await Promise.race([
                    timer, mod.isPlatformWrapper ? mod.isPlatformWrapper() : Promise.resolve(false)
                ]),
                url: mod.productUrl,
            };
        }));
    }
    setWallet(id) {
        const target = this.modules.find((mod) => mod.productId === id);
        if (!target) {
            throw new Error(`Wallet id "${id}" is not supported`);
        }
        store.setSelectedModuleId(target.productId);
    }
    async getAddress(params) {
        const { address } = await this.selectedModule.getAddress(params);
        store.setAddress(address);
        return { address };
    }
    async signTransaction(xdr, opts) {
        return this.selectedModule.signTransaction(xdr, {
            ...opts,
            networkPassphrase: opts?.networkPassphrase || store.store.getValue().selectedNetwork,
        });
    }
    async signAuthEntry(authEntry, opts) {
        return this.selectedModule.signAuthEntry(authEntry, {
            ...opts,
            networkPassphrase: opts?.networkPassphrase || store.store.getValue().selectedNetwork,
        });
    }
    async signMessage(message, opts) {
        return this.selectedModule.signMessage(message, {
            ...opts,
            networkPassphrase: opts?.networkPassphrase || store.store.getValue().selectedNetwork,
        });
    }
    async getNetwork() {
        return this.selectedModule.getNetwork();
    }
    async disconnect() {
        store.removeAddress();
    }
    // ---- Button methods
    isButtonCreated() {
        return !!this.buttonElement;
    }
    /**
     * This method allows developers to set their own buttons (for connection and disconnection) on their website
     * while letting the kit handle the logic behind opening the modal, setting and removing the address from the storage, etc
     */
    assignButtons(params) {
        const connectEl = typeof params.connectEl === 'string'
            ? document.querySelector(params.connectEl)
            : params.connectEl;
        if (!connectEl)
            throw new Error('connectEl is not available');
        connectEl.addEventListener('click', () => {
            this.openModal({
                onWalletSelected: option => {
                    store.setSelectedModuleId(option.id);
                    this.getAddress().then((r) => params.onConnect(r));
                },
            }).then();
        }, false);
        if (!params.disconnectEl)
            return;
        const disconnectEl = typeof params.disconnectEl === 'string'
            ? document.querySelector(params.disconnectEl)
            : params.disconnectEl;
        if (!disconnectEl)
            throw new Error('disconnectEl is not available');
        disconnectEl.addEventListener('click', () => {
            params.onDisconnect();
            store.removeAddress();
            if (this.selectedModule.disconnect) {
                this.selectedModule.disconnect().then();
            }
        }, false);
    }
    /**
     *
     * @param params {Object}
     * @param params.container {HTMLElement} - The container where the button should be rendered.
     * @param params.onConnect {Function} - This callback is called after the user has clicked the button and selected a wallet
     * @param params.onClosed {Function} - This callback is called if the user closes the modal without selecting any wallet.
     * @param params.onError {Function} - This callback is called if there is an error while trying to get the address once the user has selected the wallet from the modal.
     * @param params.onDisconnect {Function} - This callback is called once the user disconnects from the dropdown modal
     * @param params.horizonUrl {String} - If this url is set, the dropdown modal will show the current XLM balance of the address fetched from the wallet
     * @param params.buttonText {String} - A custom text to set inside the button.
     */
    async createButton(params) {
        if (this.buttonElement) {
            throw new Error(`Stellar Wallets Kit button is already created`);
        }
        this.buttonElement = document.createElement('stellar-wallets-button');
        if (params.buttonText) {
            this.buttonElement.setAttribute('buttonText', params.buttonText);
        }
        if (params.horizonUrl) {
            store.setHorizonUrl(params.horizonUrl);
        }
        params.container.appendChild(this.buttonElement);
        this.buttonElement.addEventListener('button-clicked', () => {
            this.openModal({
                onWalletSelected: option => {
                    store.setSelectedModuleId(option.id);
                    this.getAddress()
                        .then((r) => params.onConnect(r))
                        .catch(err => {
                        if (params.onError)
                            params.onError(err);
                    });
                },
                onClosed: (err) => {
                    if (params.onClosed)
                        params.onClosed(err);
                },
            });
        }, false);
        this.buttonElement.addEventListener('disconnect-wallet', () => {
            params.onDisconnect();
            if (this.selectedModule.disconnect) {
                this.selectedModule.disconnect();
            }
        }, false);
    }
    /**
     * Removes the button elements from the HTML and from the kit's instance.
     *
     * @param params.skipDisconnect - Set this to `true` if you want to prevent that we disconnect (for example, disconnecting WalletConnect or removing the address)
     */
    async removeButton(params) {
        if (!this.buttonElement) {
            throw new Error(`Stellar Wallets Kit button hasn't been created yet`);
        }
        if (params?.skipDisconnect !== true) {
            this.buttonElement.disconnect();
        }
        this.buttonElement.remove();
        delete this.buttonElement;
    }
    // ---- END Button methods
    // ---- Modal methods
    async openModal(params) {
        if (this.modalElement && !this.buttonElement) {
            throw new Error(`Stellar Wallets Kit modal is already open`);
        }
        else {
            this.modalElement = document.createElement('stellar-wallets-modal');
        }
        const supportedWallets = await this.getSupportedWallets();
        store.setAllowedWallets(supportedWallets);
        this.modalElement.setAttribute('showModal', '');
        if (params.modalTitle) {
            this.modalElement.setAttribute('modalTitle', params.modalTitle);
        }
        if (params.notAvailableText) {
            this.modalElement.setAttribute('notAvailableText', params.notAvailableText);
        }
        document.body.appendChild(this.modalElement);
        const listener = (event) => {
            params.onWalletSelected(event.detail);
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            this.modalElement.removeEventListener('wallet-selected', listener, false);
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            document.body.removeChild(this.modalElement);
            this.modalElement = undefined;
        };
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        this.modalElement.addEventListener('wallet-selected', listener, false);
        const errorListener = (event) => {
            if (params.onClosed) {
                params.onClosed(event.detail);
            }
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            this.modalElement.removeEventListener('wallet-selected', listener, false);
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            this.modalElement.removeEventListener('modal-closed', errorListener, false);
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            document.body.removeChild(this.modalElement);
            this.modalElement = undefined;
        };
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        this.modalElement.addEventListener('modal-closed', errorListener, false);
    }
}

exports.StellarWalletsKit = StellarWalletsKit;
//# sourceMappingURL=stellar-wallets-kit.cjs.map
