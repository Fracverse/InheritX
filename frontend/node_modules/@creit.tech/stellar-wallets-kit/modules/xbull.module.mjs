import { xBullWalletConnect } from '@creit.tech/xbull-wallet-connect';
import { ModuleType } from '../types.mjs';
import { parseError } from '../utils.mjs';

const XBULL_ID = "xbull";
class xBullModule {
  constructor() {
    this.moduleType = ModuleType.HOT_WALLET;
    this.productId = XBULL_ID;
    this.productName = "xBull";
    this.productUrl = "https://xbull.app";
    this.productIcon = "https://stellar.creit.tech/wallet-icons/xbull.png";
  }
  async isAvailable() {
    return true;
  }
  async getAddress() {
    try {
      const bridge = new xBullWalletConnect();
      const publicKey = await bridge.connect();
      bridge.closeConnections();
      return { address: publicKey };
    } catch (e) {
      throw parseError(e);
    }
  }
  async signTransaction(xdr, opts) {
    try {
      const bridge = new xBullWalletConnect();
      const signedXdr = await bridge.sign({
        xdr,
        publicKey: opts?.address,
        network: opts?.networkPassphrase
      });
      bridge.closeConnections();
      return { signedTxXdr: signedXdr, signerAddress: opts?.address };
    } catch (e) {
      throw parseError(e);
    }
  }
  async signAuthEntry() {
    throw {
      code: -3,
      message: 'xBull does not support the "signAuthEntry" function'
    };
  }
  async signMessage(message, opts) {
    try {
      const bridge = new xBullWalletConnect();
      const result = await bridge.signMessage(message, {
        address: opts?.address,
        networkPassphrase: opts?.networkPassphrase
      });
      bridge.closeConnections();
      return result;
    } catch (e) {
      throw parseError(e);
    }
  }
  async getNetwork() {
    throw {
      code: -3,
      message: 'xBull does not support the "getNetwork" function'
    };
  }
}

export { XBULL_ID, xBullModule };
//# sourceMappingURL=xbull.module.mjs.map
