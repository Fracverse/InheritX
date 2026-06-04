"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseGethGenesisState = parseGethGenesisState;
const util_1 = require("@ethereumjs/util");
/**
 * Parses the geth genesis state into Blockchain {@link GenesisState}
 * @param gethGenesis GethGenesis object
 * @returns Parsed {@link GenesisState}
 */
function parseGethGenesisState(gethGenesis) {
    const state = {};
    for (const address of Object.keys(gethGenesis.alloc)) {
        const { balance: rawBalance, code: rawCode, nonce: rawNonce, storage: rawStorage, } = gethGenesis.alloc[address];
        // create a map with lowercase for easy lookups
        const prefixedAddress = (0, util_1.addHexPrefix)(address.toLowerCase());
        const balance = (0, util_1.isHexString)(rawBalance) ? rawBalance : (0, util_1.bigIntToHex)(BigInt(rawBalance));
        const code = rawCode !== undefined ? (0, util_1.addHexPrefix)(rawCode) : undefined;
        const storage = rawStorage !== undefined
            ? Object.entries(rawStorage).map((storagePair) => storagePair.map(util_1.addHexPrefix))
            : undefined;
        const nonce = rawNonce !== undefined ? (0, util_1.addHexPrefix)(rawNonce) : undefined;
        state[prefixedAddress] = [balance, code, storage, nonce];
    }
    return state;
}
//# sourceMappingURL=gethGenesis.js.map