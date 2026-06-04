'use strict';

var elf = require('@ngneat/elf');
var types = require('../types.cjs');

const store = elf.createStore({ name: 'state' }, elf.withProps({
    allowedWallets: [],
    hardwareWalletPaths: [],
}));
const allowedWallets$ = store.pipe(elf.select((state) => state.allowedWallets));
const selectedNetwork$ = store.pipe(elf.select((state) => state.selectedNetwork));
const modalTheme$ = store.pipe(elf.select((state) => state.modalTheme));
const buttonTheme$ = store.pipe(elf.select((state) => state.buttonTheme));
const activeAddress$ = store.pipe(elf.select((state) => state.activeAddress));
const horizonUrl$ = store.pipe(elf.select((state) => state.horizonUrl));
const mnemonicPath$ = store.pipe(elf.select((state) => state.mnemonicPath));
const hardwareWalletPaths$ = store.pipe(elf.select((state) => state.hardwareWalletPaths));
function setSelectedModuleId(moduleId) {
    store.update(elf.setProp('selectedModuleId', moduleId));
}
function setNetwork(network) {
    if (!Object.values(types.WalletNetwork).includes(network)) {
        throw new Error(`Wallet network "${network}" is not supported`);
    }
    store.update(elf.setProp('selectedNetwork', network));
}
function setModalTheme(theme) {
    store.update(elf.setProp('modalTheme', theme));
}
function seButtonTheme(theme) {
    store.update(elf.setProp('buttonTheme', theme));
}
function setAllowedWallets(data) {
    store.update(elf.setProp('allowedWallets', data));
}
function setAddress(address) {
    store.update(elf.setProp('activeAddress', address));
}
function removeAddress() {
    store.update(elf.setProp('activeAddress', undefined));
}
function setHorizonUrl(url) {
    store.update(elf.setProp('horizonUrl', url));
}
function setMnemonicPath(path) {
    store.update(elf.setProp('mnemonicPath', path));
}
function removeMnemonicPath() {
    store.update(elf.setProp('mnemonicPath', undefined));
}
function setHardwareWalletPaths(accounts) {
    store.update(elf.setProp('hardwareWalletPaths', accounts));
}
function removeHardwareWalletPaths() {
    store.update(elf.setProp('hardwareWalletPaths', []));
}

exports.activeAddress$ = activeAddress$;
exports.allowedWallets$ = allowedWallets$;
exports.buttonTheme$ = buttonTheme$;
exports.hardwareWalletPaths$ = hardwareWalletPaths$;
exports.horizonUrl$ = horizonUrl$;
exports.mnemonicPath$ = mnemonicPath$;
exports.modalTheme$ = modalTheme$;
exports.removeAddress = removeAddress;
exports.removeHardwareWalletPaths = removeHardwareWalletPaths;
exports.removeMnemonicPath = removeMnemonicPath;
exports.seButtonTheme = seButtonTheme;
exports.selectedNetwork$ = selectedNetwork$;
exports.setAddress = setAddress;
exports.setAllowedWallets = setAllowedWallets;
exports.setHardwareWalletPaths = setHardwareWalletPaths;
exports.setHorizonUrl = setHorizonUrl;
exports.setMnemonicPath = setMnemonicPath;
exports.setModalTheme = setModalTheme;
exports.setNetwork = setNetwork;
exports.setSelectedModuleId = setSelectedModuleId;
exports.store = store;
//# sourceMappingURL=store.cjs.map
