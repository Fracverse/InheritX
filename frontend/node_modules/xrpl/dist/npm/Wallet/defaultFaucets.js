"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getFaucetPath = exports.getFaucetHost = exports.faucetNetworkIDs = exports.faucetNetworkPaths = exports.FaucetNetwork = void 0;
const errors_1 = require("../errors");
var FaucetNetwork;
(function (FaucetNetwork) {
    FaucetNetwork["Testnet"] = "faucet.altnet.rippletest.net";
    FaucetNetwork["Devnet"] = "faucet.devnet.rippletest.net";
})(FaucetNetwork || (exports.FaucetNetwork = FaucetNetwork = {}));
exports.faucetNetworkPaths = {
    [FaucetNetwork.Testnet]: '/accounts',
    [FaucetNetwork.Devnet]: '/accounts',
};
exports.faucetNetworkIDs = new Map([
    [1, FaucetNetwork.Testnet],
    [2, FaucetNetwork.Devnet],
]);
function getFaucetHost(client) {
    if (client.networkID == null) {
        throw new errors_1.XRPLFaucetError('Cannot create faucet URL without networkID or the faucetHost information');
    }
    if (exports.faucetNetworkIDs.has(client.networkID)) {
        return exports.faucetNetworkIDs.get(client.networkID);
    }
    if (client.networkID === 0) {
        throw new errors_1.XRPLFaucetError('Faucet is not available for mainnet.');
    }
    throw new errors_1.XRPLFaucetError('Faucet URL is not defined or inferrable.');
}
exports.getFaucetHost = getFaucetHost;
function getFaucetPath(hostname) {
    if (hostname === undefined) {
        return '/accounts';
    }
    return exports.faucetNetworkPaths[hostname] || '/accounts';
}
exports.getFaucetPath = getFaucetPath;
//# sourceMappingURL=defaultFaucets.js.map