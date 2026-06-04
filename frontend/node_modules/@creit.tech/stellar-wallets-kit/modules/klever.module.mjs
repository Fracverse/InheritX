import { ModuleType } from '../types.mjs';
import { parseError } from '../utils.mjs';

const KLEVER_ID = "klever";
class KleverModule {
  constructor() {
    this.moduleType = ModuleType.HOT_WALLET;
    this.productId = KLEVER_ID;
    this.productName = "Klever Wallet";
    this.productUrl = "https://klever.io/";
    this.productIcon = "https://stellar.creit.tech/wallet-icons/klever.png";
  }
  async runChecks() {
    if (!await this.isAvailable()) {
      throw new Error("Klever Wallet is not installed");
    }
  }
  isAvailable() {
    return new Promise((resolve) => resolve(typeof window !== "undefined" && !!window.kleverWallet?.stellar));
  }
  async getAddress() {
    return this.runChecks().then(() => window.kleverWallet.stellar.getAddress()).catch((e) => {
      throw parseError(e);
    });
  }
  async signTransaction(xdr, opts) {
    return this.runChecks().then(() => window.kleverWallet.stellar.signTransaction(xdr, opts)).catch((e) => {
      throw parseError(e);
    });
  }
  async signAuthEntry(authEntry, opts) {
    return this.runChecks().then(() => window.kleverWallet.stellar.signAuthEntry(authEntry, opts)).catch((e) => {
      throw parseError(e);
    });
  }
  async signMessage(message, opts) {
    return this.runChecks().then(() => window.kleverWallet.stellar.signMessage(message, opts)).catch((e) => {
      throw parseError(e);
    });
  }
  async getNetwork() {
    return this.runChecks().then(() => window.kleverWallet.stellar.getNetwork()).catch((e) => {
      throw parseError(e);
    });
  }
}

export { KLEVER_ID, KleverModule };
//# sourceMappingURL=klever.module.mjs.map
