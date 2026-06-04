export type Chain = (typeof Chain)[keyof typeof Chain];
export declare const Chain: {
    readonly Mainnet: 1;
    readonly Sepolia: 11155111;
    readonly Holesky: 17000;
    readonly Hoodi: 560048;
};
export declare const ChainNameFromNumber: {
    [key in Chain]: string;
};
/**
 * Genesis state meta info which is decoupled from common's genesis params
 */
type GenesisState = {
    name: string;
    blockNumber: bigint;
    stateRoot: Uint8Array;
};
/**
 * GenesisState info about well known ethereum chains
 */
export declare const ChainGenesis: Record<Chain, GenesisState>;
export type Hardfork = (typeof Hardfork)[keyof typeof Hardfork];
export declare const Hardfork: {
    readonly Chainstart: "chainstart";
    readonly Homestead: "homestead";
    readonly Dao: "dao";
    readonly TangerineWhistle: "tangerineWhistle";
    readonly SpuriousDragon: "spuriousDragon";
    readonly Byzantium: "byzantium";
    readonly Constantinople: "constantinople";
    readonly Petersburg: "petersburg";
    readonly Istanbul: "istanbul";
    readonly MuirGlacier: "muirGlacier";
    readonly Berlin: "berlin";
    readonly London: "london";
    readonly ArrowGlacier: "arrowGlacier";
    readonly GrayGlacier: "grayGlacier";
    readonly MergeNetsplitBlock: "mergeNetsplitBlock";
    readonly Paris: "paris";
    readonly Shanghai: "shanghai";
    readonly Cancun: "cancun";
    readonly Prague: "prague";
    readonly Osaka: "osaka";
    readonly Bpo1: "bpo1";
    readonly Bpo2: "bpo2";
    readonly Bpo3: "bpo3";
    readonly Bpo4: "bpo4";
    readonly Bpo5: "bpo5";
    readonly Amsterdam: "amsterdam";
};
export type ConsensusType = (typeof ConsensusType)[keyof typeof ConsensusType];
export declare const ConsensusType: {
    readonly ProofOfStake: "pos";
    readonly ProofOfWork: "pow";
    readonly ProofOfAuthority: "poa";
};
export type ConsensusAlgorithm = (typeof ConsensusAlgorithm)[keyof typeof ConsensusAlgorithm];
export declare const ConsensusAlgorithm: {
    readonly Ethash: "ethash";
    readonly Clique: "clique";
    readonly Casper: "casper";
};
export {};
//# sourceMappingURL=enums.d.ts.map