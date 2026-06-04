import { LitElement, html, css } from 'lit';
import { property, state, customElement } from 'lit/decorators.js';
import { modalDialogStyles, modalDialogBodyStyles, modalHelpSection, backdropStyles, modalAnimations, modalWalletsSection } from './styles.mjs';

var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i = decorators.length - 1, decorator; i >= 0; i--)
    if (decorator = decorators[i])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp(target, key, result);
  return result;
};
let StellarSelectorModal = class extends LitElement {
  constructor() {
    super(...arguments);
    this.showModal = false;
    this.loadingAccounts = false;
    this.closingModal = false;
    this.modalTitle = "Pick your account";
    this.accounts = [];
  }
  connectedCallback() {
    super.connectedCallback();
  }
  async pickAccount(option) {
    this.closingModal = true;
    await new Promise((r) => setTimeout(r, 280));
    this.dispatchEvent(
      new CustomEvent("account-selected", {
        detail: option,
        bubbles: true,
        composed: true
      })
    );
    this.closingModal = false;
  }
  async closeModal() {
    this.closingModal = true;
    await new Promise((r) => setTimeout(r, 280));
    this.showModal = false;
    this.dispatchEvent(
      new CustomEvent("account-selector-closed", {
        detail: new Error("Account selector closed"),
        bubbles: true,
        composed: true
      })
    );
    this.closingModal = false;
  }
  render() {
    const loadingIcon = html`
      <svg width="24" height="24" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <style>
          .spinner_qM83 {
            animation: spinner_8HQG 1.05s infinite;
            fill: white;
          }
          .spinner_oXPr {
            animation-delay: 0.1s;
          }
          .spinner_ZTLf {
            animation-delay: 0.2s;
          }
          @keyframes spinner_8HQG {
            0%,
            57.14% {
              animation-timing-function: cubic-bezier(0.33, 0.66, 0.66, 1);
              transform: translate(0);
            }
            28.57% {
              animation-timing-function: cubic-bezier(0.33, 0, 0.66, 0.33);
              transform: translateY(-6px);
            }
            100% {
              transform: translate(0);
            }
          }
        </style>
        <circle class="spinner_qM83" cx="4" cy="12" r="3" />
        <circle class="spinner_qM83 spinner_oXPr" cx="12" cy="12" r="3" />
        <circle class="spinner_qM83 spinner_ZTLf" cx="20" cy="12" r="3" />
      </svg>
    `;
    const accountsList = html`
      <ul class="wallets-body">
        ${this.accounts.map(
      ({ publicKey, index }) => html`
            <li @click="${() => this.pickAccount({ publicKey, index })}" class="wallets-body__item">
              <span style="margin-right: 1rem;" class="dialog-text-solid">
                ${publicKey.slice(0, 4)}....${publicKey.slice(-6)}
              </span>
              <span class="dialog-text">(44'/148'/${index}')</span>
            </li>
          `
    )}
      </ul>
    `;
    return html`
      <dialog style="" class="dialog-modal ${this.closingModal ? "closing" : ""}" .open=${this.showModal}>
        <section class="dialog-modal-body">
          <div class="dialog-modal-body__wallets">
            <section class="wallets-container">
              <header class="wallets-header">
                <h2 class="wallets-header__modal-title dialog-text-solid">${this.modalTitle}</h2>

                <button @click=${() => this.closeModal()} class="wallets-header__button">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="#000000"
                    height="20px"
                    width="20px"
                    viewBox="0 0 490 490">
                    <polygon
                      points="456.851,0 245,212.564 33.149,0 0.708,32.337 212.669,245.004 0.708,457.678 33.149,490 245,277.443 456.851,490   489.292,457.678 277.331,245.004 489.292,32.337 " />
                  </svg>
                </button>
              </header>

              ${this.loadingAccounts ? loadingIcon : accountsList}
            </section>
          </div>
        </section>
      </dialog>

      <div
        style="position: fixed; z-index: 950"
        class="backdrop ${this.closingModal ? "closing" : ""}"
        @click=${() => this.closeModal()}></div>
    `;
  }
};
StellarSelectorModal.styles = [
  css`
      :host * {
        box-sizing: border-box;
      }

      .mb-0 {
        margin-bottom: 0 !important;
      }
    `,
  modalDialogStyles,
  modalDialogBodyStyles,
  modalHelpSection,
  backdropStyles,
  modalAnimations,
  modalWalletsSection
];
__decorateClass([
  property({ type: Boolean, reflect: true })
], StellarSelectorModal.prototype, "showModal", 2);
__decorateClass([
  property({ type: Boolean, reflect: true })
], StellarSelectorModal.prototype, "loadingAccounts", 2);
__decorateClass([
  state()
], StellarSelectorModal.prototype, "closingModal", 2);
__decorateClass([
  property({ type: String, reflect: true })
], StellarSelectorModal.prototype, "modalTitle", 2);
__decorateClass([
  property({ type: String, reflect: true, converter: (value) => value ? JSON.parse(value) : [] })
], StellarSelectorModal.prototype, "accounts", 2);
StellarSelectorModal = __decorateClass([
  customElement("stellar-accounts-selector")
], StellarSelectorModal);

export { StellarSelectorModal };
//# sourceMappingURL=stellar-selector-modal.mjs.map
