import { ModuleType, WalletNetwork } from '../types.mjs';
import { parseError } from '../utils.mjs';

const RABET_ID = "rabet";
class RabetModule {
  constructor() {
    this.moduleType = ModuleType.HOT_WALLET;
    this.productId = RABET_ID;
    this.productName = "Rabet";
    this.productUrl = "https://rabet.io/";
    this.productIcon = "https://stellar.creit.tech/wallet-icons/rabet.png";
  }
  isAvailable() {
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve(typeof window !== "undefined" && !!window.rabet);
      }, 100);
    });
  }
  async getAddress() {
    const runChecks = async () => {
      if (!await this.isAvailable()) {
        throw new Error("Rabet is not installed");
      }
    };
    return runChecks().then(() => window.rabet.connect()).then(({ publicKey }) => ({ address: publicKey })).catch((e) => {
      throw parseError(e);
    });
  }
  async signTransaction(xdr, opts) {
    const runChecks = async () => {
      if (!await this.isAvailable()) {
        throw new Error("Rabet is not installed");
      }
      if (opts?.address && opts.networkPassphrase !== WalletNetwork.PUBLIC && opts.networkPassphrase !== WalletNetwork.TESTNET) {
        throw new Error(`Rabet doesn't support the network: ${opts.networkPassphrase}`);
      }
      if (opts?.address) {
        console.warn(`Rabet doesn't allow specifying the network that should be used, we skip the value`);
      }
    };
    const sign = async () => window.rabet.sign(
      xdr,
      opts?.networkPassphrase === WalletNetwork.PUBLIC ? "mainnet" /* PUBLIC */ : "testnet" /* TESTNET */
    );
    return runChecks().then(sign).then((result) => ({ signedTxXdr: result?.xdr })).catch((e) => {
      throw parseError(e);
    });
  }
  async signAuthEntry() {
    throw {
      code: -3,
      message: 'Rabet does not support the "signAuthEntry" function'
    };
  }
  async signMessage() {
    throw {
      code: -3,
      message: 'Rabet does not support the "signMessage" function'
    };
  }
  async getNetwork() {
    throw {
      code: -3,
      message: 'Rabet does not support the "getNetwork" function'
    };
  }
}
var RabetNetwork = /* @__PURE__ */ ((RabetNetwork2) => {
  RabetNetwork2["PUBLIC"] = "mainnet";
  RabetNetwork2["TESTNET"] = "testnet";
  return RabetNetwork2;
})(RabetNetwork || {});

export { RABET_ID, RabetModule, RabetNetwork };
//# sourceMappingURL=rabet.module.mjs.map
