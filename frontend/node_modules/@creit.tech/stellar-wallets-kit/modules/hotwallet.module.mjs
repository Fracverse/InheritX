import { HOT } from '@hot-wallet/sdk';
import { ModuleType, WalletNetwork } from '../types.mjs';

const HOTWALLET_ID = "hot-wallet";
class HotWalletModule {
  constructor() {
    this.moduleType = ModuleType.HOT_WALLET;
    this.productId = HOTWALLET_ID;
    this.productName = "HOT Wallet";
    this.productUrl = "https://hot-labs.org/wallet";
    this.productIcon = "https://storage.herewallet.app/logo.png";
  }
  async isAvailable() {
    return true;
  }
  async getAddress() {
    return await HOT.request("stellar:getAddress", {});
  }
  async signTransaction(xdr, opts) {
    return await HOT.request("stellar:signTransaction", { xdr, accountToSign: opts?.address });
  }
  async signAuthEntry(authEntry, opts) {
    return await HOT.request("stellar:signAuthEntry", { authEntry, accountToSign: opts?.address });
  }
  async signMessage(message, opts) {
    return await HOT.request("stellar:signMessage", { message, accountToSign: opts?.address });
  }
  async getNetwork() {
    return { network: "mainnet", networkPassphrase: WalletNetwork.PUBLIC };
  }
}

export { HOTWALLET_ID, HotWalletModule };
//# sourceMappingURL=hotwallet.module.mjs.map
