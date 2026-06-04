'use strict';

exports.WalletNetwork = void 0;
(function (WalletNetwork) {
    WalletNetwork["PUBLIC"] = "Public Global Stellar Network ; September 2015";
    WalletNetwork["TESTNET"] = "Test SDF Network ; September 2015";
    WalletNetwork["FUTURENET"] = "Test SDF Future Network ; October 2022";
    WalletNetwork["SANDBOX"] = "Local Sandbox Stellar Network ; September 2022";
    WalletNetwork["STANDALONE"] = "Standalone Network ; February 2017";
})(exports.WalletNetwork || (exports.WalletNetwork = {}));
exports.ModuleType = void 0;
(function (ModuleType) {
    ModuleType["HW_WALLET"] = "HW_WALLET";
    ModuleType["HOT_WALLET"] = "HOT_WALLET";
    ModuleType["BRIDGE_WALLET"] = "BRIDGE_WALLET";
    ModuleType["AIR_GAPED_WALLET"] = "AIR_GAPED_WALLET";
})(exports.ModuleType || (exports.ModuleType = {}));
//# sourceMappingURL=types.cjs.map
