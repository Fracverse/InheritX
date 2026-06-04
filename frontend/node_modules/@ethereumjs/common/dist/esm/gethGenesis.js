import { addHexPrefix, bigIntToHex, isHexString } from '@ethereumjs/util';
/**
 * Parses the geth genesis state into Blockchain {@link GenesisState}
 * @param gethGenesis GethGenesis object
 * @returns Parsed {@link GenesisState}
 */
export function parseGethGenesisState(gethGenesis) {
    const state = {};
    for (const address of Object.keys(gethGenesis.alloc)) {
        const { balance: rawBalance, code: rawCode, nonce: rawNonce, storage: rawStorage, } = gethGenesis.alloc[address];
        // create a map with lowercase for easy lookups
        const prefixedAddress = addHexPrefix(address.toLowerCase());
        const balance = isHexString(rawBalance) ? rawBalance : bigIntToHex(BigInt(rawBalance));
        const code = rawCode !== undefined ? addHexPrefix(rawCode) : undefined;
        const storage = rawStorage !== undefined
            ? Object.entries(rawStorage).map((storagePair) => storagePair.map(addHexPrefix))
            : undefined;
        const nonce = rawNonce !== undefined ? addHexPrefix(rawNonce) : undefined;
        state[prefixedAddress] = [balance, code, storage, nonce];
    }
    return state;
}
//# sourceMappingURL=gethGenesis.js.map