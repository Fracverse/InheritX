import TrezorConnect from '@trezor/connect-web';
import { transformTransaction } from '@trezor/connect-plugin-stellar';
import { Transaction } from '@stellar/stellar-base';
import { firstValueFrom } from 'rxjs';
import { removeMnemonicPath, removeHardwareWalletPaths, mnemonicPath$, setHardwareWalletPaths, setMnemonicPath, hardwareWalletPaths$, selectedNetwork$ } from '../state/store.mjs';
import { ModuleType } from '../types.mjs';
import { parseError } from '../utils.mjs';

const TREZOR_ID = "TREZOR";
class TrezorModule {
  constructor(params) {
    this.TrezorConnect = "default" in TrezorConnect ? TrezorConnect.default : TrezorConnect;
    this._isAvailable = false;
    this.moduleType = ModuleType.HW_WALLET;
    this.productId = TREZOR_ID;
    this.productName = "Trezor";
    this.productUrl = "https://www.trezor.com/";
    this.productIcon = "https://stellar.creit.tech/wallet-icons/trezor.png";
    this.TrezorConnect.init({
      manifest: {
        appName: params.appName,
        appUrl: params.appUrl,
        email: params.email
      },
      // More advanced options
      debug: params.debug || false,
      lazyLoad: params.lazyLoad || false,
      coreMode: params.coreMode || "auto"
    }).then(() => {
      console.log("Trezor is ready");
      this._isAvailable = true;
    });
  }
  async disconnect() {
    removeMnemonicPath();
    removeHardwareWalletPaths();
  }
  /**
   * `TrezorConnect` needs to be started before we can use it but because users most likely
   * won't use their devices as soon as the site loads, we return `true` since it should be already started
   * once the user needs to interact with it.
   */
  async isAvailable() {
    return true;
  }
  async runChecks() {
    if (!this._isAvailable) {
      throw parseError(new Error("Trezor connection has not been started yet."));
    }
  }
  async getAddress(opts) {
    await this.runChecks();
    try {
      const mnemonicPath = opts?.path || await firstValueFrom(mnemonicPath$);
      if (!mnemonicPath) {
        const result = await this.openAccountSelector();
        return { address: result.publicKey };
      } else {
        const result = await this.TrezorConnect.stellarGetAddress({ path: mnemonicPath, showOnTrezor: false });
        if (!result.success) {
          throw parseError(new Error(result.payload.error));
        }
        return { address: result.payload.address };
      }
    } catch (e) {
      throw parseError(e);
    }
  }
  /**
   * This method is used by the Wallets Kit itself, if you're a dApp developer, most likely you don't need to use this method.
   * @param page - {Number}
   */
  async getAddresses(page = 0) {
    const startIndex = page * 10;
    const bundle = new Array(10).fill(void 0).map((_, i) => ({
      path: `m/44'/148'/${i + startIndex}'`,
      showOnTrezor: false
    }));
    const result = await this.TrezorConnect.stellarGetAddress({ bundle });
    if (!result.success) {
      throw parseError(new Error(result.payload.error));
    }
    const results = result.payload.map((item, i) => ({
      publicKey: item.address,
      index: i + startIndex
    }));
    setHardwareWalletPaths(results);
    return results;
  }
  /**
   * This method is used by the Wallets Kit itself, if you're a dApp developer, most likely you don't need to use this method.
   */
  async openAccountSelector() {
    return new Promise((resolve, reject) => {
      const el = document.createElement("stellar-accounts-selector");
      el.setAttribute("showModal", "");
      el.setAttribute("loadingAccounts", "");
      document.body.appendChild(el);
      this.getAddresses().then((addressesData) => {
        el.setAttribute("accounts", JSON.stringify(addressesData));
        el.removeAttribute("loadingAccounts");
      }).catch((err) => {
        el.remove();
        reject(err);
      });
      const listener = (event) => {
        const { publicKey, index } = event.detail;
        setMnemonicPath(`44'/148'/${index}'`);
        resolve({ publicKey, index });
        el.removeEventListener("account-selected", listener, false);
        document.body.removeChild(el);
      };
      el.addEventListener("account-selected", listener, false);
      const errorListener = (event) => {
        el.removeEventListener("account-selected", listener, false);
        el.removeEventListener("account-selector-closed", errorListener, false);
        document.body.removeChild(el);
        reject(event.detail);
      };
      el.addEventListener("account-selector-closed", errorListener, false);
    });
  }
  async signTransaction(xdr, opts) {
    await this.runChecks();
    let mnemonicPath;
    let account;
    if (opts?.path) {
      mnemonicPath = opts.path;
      const result2 = await this.TrezorConnect.stellarGetAddress({ path: mnemonicPath, showOnTrezor: false });
      if (!result2.success) {
        throw new Error(result2.payload.error);
      }
      account = result2.payload.address;
    } else if (opts?.address) {
      const paths = await firstValueFrom(hardwareWalletPaths$);
      const target = paths.find((p) => p.publicKey === opts.address);
      if (!target) throw parseError(new Error("This address has not been loaded from this device"));
      mnemonicPath = `m/44'/148'/${target.index}'`;
      account = target.publicKey;
    } else {
      mnemonicPath = await firstValueFrom(mnemonicPath$);
      if (!mnemonicPath)
        throw parseError(new Error("There is no path available, please call the `getAddress` method first."));
      const result2 = await this.TrezorConnect.stellarGetAddress({ path: mnemonicPath, showOnTrezor: false });
      if (!result2.success) {
        throw new Error(result2.payload.error);
      }
      account = result2.payload.address;
    }
    const network = opts?.networkPassphrase || await firstValueFrom(selectedNetwork$);
    if (!network) throw parseError(new Error("You need to provide or set a network passphrase"));
    const tx = new Transaction(xdr, network);
    const parsedTx = transformTransaction(mnemonicPath, tx);
    const result = await this.TrezorConnect.stellarSignTransaction(parsedTx);
    if (!result.success) {
      throw parseError(new Error(result.payload.error));
    }
    tx.addSignature(account, Buffer.from(result.payload.signature, "hex").toString("base64"));
    return {
      signedTxXdr: tx.toXDR(),
      signerAddress: account
    };
  }
  async signAuthEntry() {
    throw {
      code: -3,
      message: 'Trezor Wallets do not support the "signAuthEntry" method'
    };
  }
  async signMessage() {
    throw {
      code: -3,
      message: 'Trezor Wallets do not support the "signMessage" method'
    };
  }
  async getNetwork() {
    throw {
      code: -3,
      message: 'Trezor Wallets do not support the "getNetwork" method'
    };
  }
}

export { TREZOR_ID, TrezorModule };
//# sourceMappingURL=trezor.module.mjs.map
