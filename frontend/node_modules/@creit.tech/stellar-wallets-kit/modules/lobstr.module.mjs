import { isConnected, getPublicKey, signTransaction } from '@lobstrco/signer-extension-api';
import { ModuleType } from '../types.mjs';
import { parseError } from '../utils.mjs';

const LOBSTR_ID = "lobstr";
class LobstrModule {
  constructor() {
    this.moduleType = ModuleType.HOT_WALLET;
    this.productId = LOBSTR_ID;
    this.productName = "LOBSTR";
    this.productUrl = "https://lobstr.co";
    this.productIcon = "https://stellar.creit.tech/wallet-icons/lobstr.png";
  }
  async isAvailable() {
    return isConnected();
  }
  async getAddress() {
    const runChecks = async () => {
      if (!await isConnected()) {
        throw new Error(`Lobstr is not connected`);
      }
    };
    return runChecks().then(() => getPublicKey()).then((address) => ({ address })).catch((e) => {
      throw parseError(e);
    });
  }
  async signTransaction(xdr, opts) {
    const runChecks = async () => {
      if (!await isConnected()) {
        throw new Error(`Lobstr is not connected`);
      }
      if (opts?.address) {
        console.warn(`Lobstr doesn't allow specifying what public key should sign the transaction, we skip the value`);
      }
      if (opts?.networkPassphrase) {
        console.warn(`Lobstr doesn't allow specifying the network that should be used, we skip the value`);
      }
    };
    return runChecks().then(() => signTransaction(xdr)).then((signedTxXdr) => ({ signedTxXdr })).catch((e) => {
      throw parseError(e);
    });
  }
  async signAuthEntry() {
    throw {
      code: -3,
      message: 'Lobstr does not support the "signAuthEntry" function'
    };
  }
  async signMessage() {
    throw {
      code: -3,
      message: 'Lobstr does not support the "signMessage" function'
    };
  }
  async getNetwork() {
    throw {
      code: -3,
      message: 'Lobstr does not support the "getNetwork" function'
    };
  }
}

export { LOBSTR_ID, LobstrModule };
//# sourceMappingURL=lobstr.module.mjs.map
