import type { GethGenesis } from './gethGenesis.ts';
import type { HardforksDict } from './types.ts';
type ConfigHardfork = {
    name: string;
    block: null;
    timestamp: number;
} | {
    name: string;
    block: number;
    timestamp?: number;
};
/**
 * Parses a genesis object exported from Geth into parameters for Common instance
 * @param gethGenesis GethGenesis object
 * @param name optional chain name
 * @returns parsed params
 */
export declare function parseGethGenesis(gethGenesis: GethGenesis, name?: string): {
    name: string | undefined;
    chainId: number;
    depositContractAddress: string | undefined;
    genesis: {
        timestamp: `0x${string}`;
        gasLimit: `0x${string}`;
        difficulty: `0x${string}` | undefined;
        nonce: `0x${string}`;
        extraData: `0x${string}`;
        mixHash: `0x${string}` | undefined;
        coinbase: `0x${string}` | undefined;
        baseFeePerGas: number | `0x${string}` | null | undefined;
        excessBlobGas: string | undefined;
        requestsHash: string | undefined;
    };
    hardfork: string | undefined;
    hardforks: ConfigHardfork[];
    customHardforks: HardforksDict | undefined;
    bootstrapNodes: never[];
    consensus: {
        type: string;
        algorithm: string;
        clique: {
            period: number | undefined;
            epoch: number | undefined;
        };
        ethash?: undefined;
    } | {
        type: string;
        algorithm: string;
        ethash: {};
        clique?: undefined;
    };
};
/**
 * Return the preset chain config for one of the predefined chain configurations
 * @param chain the representing a network name (e.g. 'mainnet') or number representing the chain ID
 * @returns a {@link ChainConfig}
 */
export declare const getPresetChainConfig: (chain: string | number) => import("./types.ts").ChainConfig;
export {};
//# sourceMappingURL=utils.d.ts.map