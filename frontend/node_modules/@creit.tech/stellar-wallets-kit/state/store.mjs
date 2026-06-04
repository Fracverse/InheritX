import { createStore, withProps, select, setProp } from '@ngneat/elf';
import { WalletNetwork } from '../types.mjs';

const store = createStore(
  { name: "state" },
  withProps({
    allowedWallets: [],
    hardwareWalletPaths: []
  })
);
const allowedWallets$ = store.pipe(
  select((state) => state.allowedWallets)
);
const selectedNetwork$ = store.pipe(
  select((state) => state.selectedNetwork)
);
const modalTheme$ = store.pipe(
  select((state) => state.modalTheme)
);
const buttonTheme$ = store.pipe(
  select((state) => state.buttonTheme)
);
const activeAddress$ = store.pipe(
  select((state) => state.activeAddress)
);
const horizonUrl$ = store.pipe(select((state) => state.horizonUrl));
const mnemonicPath$ = store.pipe(
  select((state) => state.mnemonicPath)
);
const hardwareWalletPaths$ = store.pipe(
  select((state) => state.hardwareWalletPaths)
);
function setSelectedModuleId(moduleId) {
  store.update(setProp("selectedModuleId", moduleId));
}
function setNetwork(network) {
  if (!Object.values(WalletNetwork).includes(network)) {
    throw new Error(`Wallet network "${network}" is not supported`);
  }
  store.update(setProp("selectedNetwork", network));
}
function setModalTheme(theme) {
  store.update(setProp("modalTheme", theme));
}
function seButtonTheme(theme) {
  store.update(setProp("buttonTheme", theme));
}
function setAllowedWallets(data) {
  store.update(setProp("allowedWallets", data));
}
function setAddress(address) {
  store.update(setProp("activeAddress", address));
}
function removeAddress() {
  store.update(setProp("activeAddress", void 0));
}
function setHorizonUrl(url) {
  store.update(setProp("horizonUrl", url));
}
function setMnemonicPath(path) {
  store.update(setProp("mnemonicPath", path));
}
function removeMnemonicPath() {
  store.update(setProp("mnemonicPath", void 0));
}
function setHardwareWalletPaths(accounts) {
  store.update(setProp("hardwareWalletPaths", accounts));
}
function removeHardwareWalletPaths() {
  store.update(setProp("hardwareWalletPaths", []));
}

export { activeAddress$, allowedWallets$, buttonTheme$, hardwareWalletPaths$, horizonUrl$, mnemonicPath$, modalTheme$, removeAddress, removeHardwareWalletPaths, removeMnemonicPath, seButtonTheme, selectedNetwork$, setAddress, setAllowedWallets, setHardwareWalletPaths, setHorizonUrl, setMnemonicPath, setModalTheme, setNetwork, setSelectedModuleId, store };
//# sourceMappingURL=store.mjs.map
