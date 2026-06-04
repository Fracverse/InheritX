'use strict';

var stellarWalletsKit = require('./stellar-wallets-kit.cjs');
var stellarWalletsModal = require('./components/modal/stellar-wallets-modal.cjs');
var stellarWalletsButton = require('./components/button/stellar-wallets-button.cjs');
var stellarSelectorModal = require('./components/selector-modal/stellar-selector-modal.cjs');
var types = require('./types.cjs');
var utils = require('./utils.cjs');
var xbull_module = require('./modules/xbull.module.cjs');
var freighter_module = require('./modules/freighter.module.cjs');
var albedo_module = require('./modules/albedo.module.cjs');
var rabet_module = require('./modules/rabet.module.cjs');
var lobstr_module = require('./modules/lobstr.module.cjs');
var hana_module = require('./modules/hana.module.cjs');
var hotwallet_module = require('./modules/hotwallet.module.cjs');
var klever_module = require('./modules/klever.module.cjs');



exports.StellarWalletsKit = stellarWalletsKit.StellarWalletsKit;
exports.ModalThemes = stellarWalletsModal.ModalThemes;
Object.defineProperty(exports, "StellarWalletsModal", {
	enumerable: true,
	get: function () { return stellarWalletsModal.StellarWalletsModal; }
});
exports.ButtonThemes = stellarWalletsButton.ButtonThemes;
Object.defineProperty(exports, "StellarWalletsButton", {
	enumerable: true,
	get: function () { return stellarWalletsButton.StellarWalletsButton; }
});
Object.defineProperty(exports, "StellarSelectorModal", {
	enumerable: true,
	get: function () { return stellarSelectorModal.StellarSelectorModal; }
});
Object.defineProperty(exports, "ModuleType", {
	enumerable: true,
	get: function () { return types.ModuleType; }
});
Object.defineProperty(exports, "WalletNetwork", {
	enumerable: true,
	get: function () { return types.WalletNetwork; }
});
exports.allowAllModules = utils.allowAllModules;
exports.parseError = utils.parseError;
exports.sep43Modules = utils.sep43Modules;
exports.XBULL_ID = xbull_module.XBULL_ID;
exports.xBullModule = xbull_module.xBullModule;
exports.FREIGHTER_ID = freighter_module.FREIGHTER_ID;
exports.FreighterModule = freighter_module.FreighterModule;
exports.ALBEDO_ID = albedo_module.ALBEDO_ID;
exports.AlbedoModule = albedo_module.AlbedoModule;
Object.defineProperty(exports, "AlbedoNetwork", {
	enumerable: true,
	get: function () { return albedo_module.AlbedoNetwork; }
});
exports.RABET_ID = rabet_module.RABET_ID;
exports.RabetModule = rabet_module.RabetModule;
Object.defineProperty(exports, "RabetNetwork", {
	enumerable: true,
	get: function () { return rabet_module.RabetNetwork; }
});
exports.LOBSTR_ID = lobstr_module.LOBSTR_ID;
exports.LobstrModule = lobstr_module.LobstrModule;
exports.HANA_ID = hana_module.HANA_ID;
exports.HanaModule = hana_module.HanaModule;
exports.HOTWALLET_ID = hotwallet_module.HOTWALLET_ID;
exports.HotWalletModule = hotwallet_module.HotWalletModule;
exports.KLEVER_ID = klever_module.KLEVER_ID;
exports.KleverModule = klever_module.KleverModule;
//# sourceMappingURL=index.cjs.map
