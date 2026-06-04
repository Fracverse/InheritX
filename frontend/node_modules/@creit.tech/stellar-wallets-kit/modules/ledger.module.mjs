import Str from '@ledgerhq/hw-app-str';
import TransportWebUSB from '@ledgerhq/hw-transport-webusb';
import { firstValueFrom } from 'rxjs';
import { removeMnemonicPath, removeHardwareWalletPaths, mnemonicPath$, setHardwareWalletPaths, setMnemonicPath, hardwareWalletPaths$, selectedNetwork$ } from '../state/store.mjs';
import { ModuleType } from '../types.mjs';
import { StrKey, Transaction } from '@stellar/stellar-base';
import { parseError } from '../utils.mjs';

const LEDGER_ID = "LEDGER";
class LedgerModule {
  constructor() {
    this.moduleType = ModuleType.HW_WALLET;
    this.productId = LEDGER_ID;
    this.productName = "Ledger";
    this.productUrl = "https://www.ledger.com/";
    this.productIcon = "https://stellar.creit.tech/wallet-icons/ledger.png";
  }
  async transport() {
    if (!await TransportWebUSB.isSupported()) throw new Error("Ledger can not be used with this device.");
    if (!this._transport) {
      this._transport = await TransportWebUSB.create();
    }
    return this._transport;
  }
  async disconnect() {
    removeMnemonicPath();
    removeHardwareWalletPaths();
    this._transport?.close();
    this._transport = void 0;
  }
  /**
   * This always return true because in theory ledgers aren't supposed
   * to be connected at all time
   */
  async isAvailable() {
    return TransportWebUSB.isSupported();
  }
  async runChecks() {
    if (!await this.isAvailable()) {
      throw new Error("Ledger wallets can not be used");
    }
  }
  async getAddress(opts) {
    await this.runChecks();
    try {
      const finalTransport = await this.transport();
      const str = new Str(finalTransport);
      let mnemonicPath = opts?.path || await firstValueFrom(mnemonicPath$);
      if (!mnemonicPath) {
        await this.openAccountSelector();
        mnemonicPath = await firstValueFrom(mnemonicPath$);
      }
      const result = await str.getPublicKey(mnemonicPath);
      return { address: StrKey.encodeEd25519PublicKey(result.rawPublicKey) };
    } catch (e) {
      throw parseError(e);
    }
  }
  /**
   * This method is used by the Wallets Kit itself, if you're a dApp developer, most likely you don't need to use this method.
   * @param page - {Number}
   */
  async getAddresses(page = 0) {
    const finalTransport = await this.transport();
    const str = new Str(finalTransport);
    const startIndex = page * 10;
    const results = [];
    for (let i = 0; i < 10; i++) {
      const result = await str.getPublicKey(`44'/148'/${i + startIndex}'`);
      results.push({
        publicKey: StrKey.encodeEd25519PublicKey(result.rawPublicKey),
        index: i + startIndex
      });
    }
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
    const finalTransport = await this.transport();
    const str = new Str(finalTransport);
    let mnemonicPath;
    let account;
    if (opts?.path) {
      mnemonicPath = opts.path;
      const result2 = await str.getPublicKey(mnemonicPath);
      account = StrKey.encodeEd25519PublicKey(result2.rawPublicKey);
    } else if (opts?.address) {
      const paths = await firstValueFrom(hardwareWalletPaths$);
      const target = paths.find((p) => p.publicKey === opts.address);
      if (!target) throw new Error("This address has not been loaded from this ledger");
      mnemonicPath = `44'/148'/${target.index}'`;
      account = target.publicKey;
    } else {
      mnemonicPath = await firstValueFrom(mnemonicPath$);
      if (!mnemonicPath) throw new Error("There is no path available, please call the `getAddress` method first.");
      const result2 = await str.getPublicKey(mnemonicPath);
      account = StrKey.encodeEd25519PublicKey(result2.rawPublicKey);
    }
    const network = opts?.networkPassphrase || await firstValueFrom(selectedNetwork$);
    if (!network) throw new Error("You need to provide or set a network passphrase");
    const tx = new Transaction(xdr, network);
    const result = opts?.nonBlindTx ? await str.signTransaction(mnemonicPath, tx.signatureBase()) : await str.signHash(mnemonicPath, tx.hash());
    tx.addSignature(account, result.signature.toString("base64"));
    return {
      signedTxXdr: tx.toXDR(),
      signerAddress: account
    };
  }
  async signAuthEntry() {
    throw {
      code: -3,
      message: 'Ledger Wallets do not support the "signAuthEntry" function'
    };
  }
  async signMessage() {
    throw {
      code: -3,
      message: 'Ledger Wallets do not support the "signMessage" function'
    };
  }
  async getNetwork() {
    throw {
      code: -3,
      message: 'Ledger Wallets do not support the "getNetwork" function'
    };
  }
}

export { LEDGER_ID, LedgerModule };
//# sourceMappingURL=ledger.module.mjs.map
