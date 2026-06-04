'use strict';

var tslib = require('tslib');
var lit = require('lit');
var styleMap_js = require('lit/directives/style-map.js');
var decorators_js = require('lit/decorators.js');
var reactiveState = require('../../state/reactive-state.cjs');
var store = require('../../state/store.cjs');
var styles = require('./styles.cjs');

exports.ModalThemeType = void 0;
(function (ModalThemeType) {
    ModalThemeType["DARK"] = "DARK";
    ModalThemeType["LIGHT"] = "LIGHT";
})(exports.ModalThemeType || (exports.ModalThemeType = {}));
const ModalThemes = {
    DARK: {
        bgColor: '#161616',
        textColor: '#a0a0a0',
        solidTextColor: '#ededed',
        headerButtonColor: '#707070',
        dividerColor: 'rgba(255, 255, 255, 0.15)',
        helpBgColor: '#1c1c1c',
        notAvailableTextColor: '#a0a0a0',
        notAvailableBgColor: '#232323',
        notAvailableBorderColor: '#343434',
    },
    LIGHT: {
        bgColor: '#fcfcfc',
        textColor: '#181818',
        solidTextColor: '#000000',
        headerButtonColor: '#8f8f8f',
        dividerColor: 'rgba(0, 0, 0, 0.15)',
        helpBgColor: '#f8f8f8',
        notAvailableTextColor: '#6f6f6f',
        notAvailableBgColor: '#f3f3f3',
        notAvailableBorderColor: '#e2e2e2',
    },
};
exports.StellarWalletsModal = class StellarWalletsModal extends lit.LitElement {
    constructor() {
        super(...arguments);
        /**
         * This value is used to tell the modal that it should not update the value
         * `showModal` at any moment, this comes handy when the state wants to be handled by another source
         */
        this.ignoreShowStatus = false;
        this.showModal = false;
        this.closingModal = false;
        this.modalTitle = 'Connect a Wallet';
        this.notAvailableText = 'Not available';
        this.allowedWallets = new reactiveState.ReactiveState(this, store.allowedWallets$);
        this.theme = new reactiveState.ReactiveState(this, store.modalTheme$);
    }
    static { this.styles = [
        lit.css `
      :host * {
        box-sizing: border-box;
      }

      .mb-0 {
        margin-bottom: 0 !important;
      }
    `,
        styles.modalDialogStyles,
        styles.modalDialogBodyStyles,
        styles.modalHelpSection,
        styles.backdropStyles,
        styles.modalAnimations,
        styles.modalWalletsSection,
    ]; }
    connectedCallback() {
        super.connectedCallback();
        const platformWrapper = this.getPlatformWrapper();
        if (platformWrapper) {
            setTimeout(() => {
                this.pickWalletOption(platformWrapper);
            }, 10);
        }
    }
    async closeModal() {
        this.closingModal = true;
        await new Promise(r => setTimeout(r, 280));
        if (!this.ignoreShowStatus) {
            this.showModal = false;
        }
        this.dispatchEvent(new CustomEvent('modal-closed', {
            detail: new Error('Modal closed'),
            bubbles: true,
            composed: true,
        }));
        this.closingModal = false;
    }
    async pickWalletOption(option) {
        if (!option.isAvailable) {
            window.open(option.url, '_blank');
            return;
        }
        this.closingModal = true;
        await new Promise(r => setTimeout(r, 280));
        try {
            const record = window.localStorage.getItem('@StellarWalletsKit/usedWalletsIds');
            let usedWalletsIds = record ? JSON.parse(record) : [];
            usedWalletsIds = [option.id, ...usedWalletsIds.filter((id) => id !== option.id)];
            window.localStorage.setItem('@StellarWalletsKit/usedWalletsIds', JSON.stringify(usedWalletsIds));
        }
        catch (e) {
            console.error(e);
        }
        this.dispatchEvent(new CustomEvent('wallet-selected', {
            detail: option,
            bubbles: true,
            composed: true,
        }));
        this.closingModal = false;
    }
    /**
     * This function gets the list of the wallets following the logic from this task https://github.com/Creit-Tech/Stellar-Wallets-Kit/issues/28
     * It follows this order:
     * 1- Wallet last used by wallet selector
     * 2- If not wallet last use, wallets detected in the browser
     * 3- Wallet ordering as defined by the developer
     *
     */
    getSortedList() {
        const sortedWallets = this.allowedWallets.value.reduce((all, current) => {
            return {
                available: current.isAvailable ? [...all.available, current] : all.available,
                unavailable: !current.isAvailable ? [...all.unavailable, current] : all.unavailable,
            };
        }, { available: [], unavailable: [] });
        let usedWalletsIds;
        try {
            const record = window.localStorage.getItem('@StellarWalletsKit/usedWalletsIds');
            usedWalletsIds = record ? JSON.parse(record) : [];
        }
        catch (e) {
            console.error(e);
            usedWalletsIds = [];
        }
        if (usedWalletsIds.length === 0) {
            return [...sortedWallets.available, ...sortedWallets.unavailable];
        }
        const usedWallets = [];
        const nonUsedWallets = [];
        for (const availableWallet of sortedWallets.available) {
            if (usedWalletsIds.find((id) => id === availableWallet.id)) {
                usedWallets.push(availableWallet);
            }
            else {
                nonUsedWallets.push(availableWallet);
            }
        }
        return [
            ...usedWallets.sort((a, b) => {
                return usedWalletsIds.indexOf(a.id) - usedWalletsIds.indexOf(b.id);
            }),
            ...nonUsedWallets,
            ...sortedWallets.unavailable,
        ];
    }
    getPlatformWrapper() {
        return this.getSortedList().find((w) => w.isPlatformWrapper);
    }
    getThemeStyles() {
        if (!this.theme.value)
            return {};
        return {
            '--modal-bg-color': this.theme.value.bgColor,
            '--modal-text-color': this.theme.value.textColor,
            '--modal-solid-text-color': this.theme.value.solidTextColor,
            '--modal-header-button-color': this.theme.value.headerButtonColor,
            '--modal-divider-color': this.theme.value.dividerColor,
            '--modal-help-bg-color': this.theme.value.helpBgColor,
            '--modal-not-available-text-color': this.theme.value.notAvailableTextColor,
            '--modal-not-available-bg-color': this.theme.value.notAvailableBgColor,
            '--modal-not-available-border-color': this.theme.value.notAvailableBorderColor,
        };
    }
    render() {
        if (this.getPlatformWrapper()) {
            return lit.html ``;
        }
        const helpSection = lit.html `
      <section class="help-container">
        <header class="help-header">
          <h2 class="help-header__modal-title dialog-text-solid">Learn more</h2>
        </header>

        <div class="help__whats_a_wallet">
          <h2 class="dialog-text-solid help__title">What is a Wallet?</h2>
          <p class="dialog-text help__text">
            Wallets are used to send, receive, and store the keys you use to sign blockchain transactions.
          </p>
        </div>

        <div class="help__whats_stellar">
          <h2 class="dialog-text-solid help__title">What is Stellar?</h2>
          <p class="dialog-text help__text">
            Stellar is a decentralized, public blockchain that gives developers the tools to create experiences that are
            more like cash than crypto.
          </p>
        </div>
      </section>
    `;
        const walletsSection = lit.html `
      <section class="wallets-container">
        <header class="wallets-header">
          <h2 class="wallets-header__modal-title dialog-text-solid">${this.modalTitle}</h2>

          <button @click=${() => this.closeModal()} class="wallets-header__button">
            <svg xmlns="http://www.w3.org/2000/svg" fill="#000000" height="20px" width="20px" viewBox="0 0 490 490">
              <polygon
                points="456.851,0 245,212.564 33.149,0 0.708,32.337 212.669,245.004 0.708,457.678 33.149,490 245,277.443 456.851,490   489.292,457.678 277.331,245.004 489.292,32.337 " />
            </svg>
          </button>
        </header>

        <ul class="wallets-body">
          ${this.getSortedList().map((item, i) => lit.html `
                <li
                  @click=${() => this.pickWalletOption(item)}
                  class=" wallets-body__item ${!item.isAvailable ? 'not-available' : ''} ${i ===
            this.allowedWallets.value.length - 1
            ? 'mb-0'
            : ''}">
                  <img src=${item.icon} alt=${item.name} />
                  <span class="dialog-text-solid">${item.name}</span>
                  ${!item.isAvailable ? lit.html `<small class="not-available">${this.notAvailableText}</small>` : ''}
                </li>
              `)}
        </ul>
      </section>
    `;
        return lit.html `
      <dialog
        style=${styleMap_js.styleMap(this.getThemeStyles())}
        class="dialog-modal ${this.closingModal ? 'closing' : ''}"
        .open=${this.showModal}>
        <section class="dialog-modal-body">
          <div class="dialog-modal-body__help">${helpSection}</div>
          <div class="dialog-modal-body__wallets">${walletsSection}</div>
        </section>
      </dialog>

      <div
        style="position: fixed; z-index: 950"
        class="backdrop ${this.closingModal ? 'closing' : ''}"
        @click=${() => this.closeModal()}></div>
    `;
    }
};
tslib.__decorate([
    decorators_js.property({ type: Boolean, reflect: true })
], exports.StellarWalletsModal.prototype, "ignoreShowStatus", void 0);
tslib.__decorate([
    decorators_js.property({ type: Boolean, reflect: true })
], exports.StellarWalletsModal.prototype, "showModal", void 0);
tslib.__decorate([
    decorators_js.state()
], exports.StellarWalletsModal.prototype, "closingModal", void 0);
tslib.__decorate([
    decorators_js.property({ type: String, reflect: true })
], exports.StellarWalletsModal.prototype, "modalTitle", void 0);
tslib.__decorate([
    decorators_js.property({ type: String, reflect: true })
], exports.StellarWalletsModal.prototype, "notAvailableText", void 0);
exports.StellarWalletsModal = tslib.__decorate([
    decorators_js.customElement('stellar-wallets-modal')
], exports.StellarWalletsModal);

exports.ModalThemes = ModalThemes;
//# sourceMappingURL=stellar-wallets-modal.cjs.map
