import albedo from '@albedo-link/intent';
import { ModuleType, WalletNetwork } from '../types.mjs';
import { parseError } from '../utils.mjs';

const ALBEDO_ID = "albedo";
class AlbedoModule {
  constructor() {
    this.moduleType = ModuleType.HOT_WALLET;
    this.productId = ALBEDO_ID;
    this.productName = "Albedo";
    this.productUrl = "https://albedo.link/";
    this.productIcon = "https://stellar.creit.tech/wallet-icons/albedo.png";
  }
  async isAvailable() {
    return true;
  }
  async getAddress() {
    return albedo.publicKey({}).then((result) => ({ address: result.pubkey })).catch((e) => {
      throw parseError(e);
    });
  }
  async signTransaction(xdr, opts) {
    return albedo.tx({
      xdr,
      pubkey: opts?.address,
      network: opts?.networkPassphrase ? opts.networkPassphrase === WalletNetwork.PUBLIC ? "public" /* PUBLIC */ : "testnet" /* TESTNET */ : void 0
    }).then(({ signed_envelope_xdr }) => ({
      signedTxXdr: signed_envelope_xdr,
      signerAddress: opts?.address
    })).catch((e) => {
      throw parseError(e);
    });
  }
  async signAuthEntry() {
    throw {
      code: -3,
      message: 'Albedo does not support the "signAuthEntry" function'
    };
  }
  /**
   * We understand that Albedo has a method to sign a message, but that method is not compatible with SEP-0043
   */
  async signMessage() {
    throw {
      code: -3,
      message: 'Albedo does not support the "signMessage" function'
    };
  }
  async getNetwork() {
    throw {
      code: -3,
      message: 'Albedo does not support the "getNetwork" function'
    };
  }
}
var AlbedoNetwork = /* @__PURE__ */ ((AlbedoNetwork2) => {
  AlbedoNetwork2["PUBLIC"] = "public";
  AlbedoNetwork2["TESTNET"] = "testnet";
  return AlbedoNetwork2;
})(AlbedoNetwork || {});

export { ALBEDO_ID, AlbedoModule, AlbedoNetwork };
//# sourceMappingURL=albedo.module.mjs.map
