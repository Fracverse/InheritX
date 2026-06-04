"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ConsensusAlgorithm = exports.ConsensusType = exports.Hardfork = exports.ChainGenesis = exports.ChainNameFromNumber = exports.Chain = void 0;
const util_1 = require("@ethereumjs/util");
exports.Chain = {
    Mainnet: 1,
    Sepolia: 11155111,
    Holesky: 17000,
    Hoodi: 560048,
};
// Reverse mapping: from numeric value back to the key name
exports.ChainNameFromNumber = Object.entries(exports.Chain).reduce((acc, [key, value]) => {
    acc[value] = key;
    return acc;
}, {});
// Having this info as record will force typescript to make sure no chain is missed
/**
 * GenesisState info about well known ethereum chains
 */
exports.ChainGenesis = {
    [exports.Chain.Mainnet]: {
        name: 'mainnet',
        blockNumber: util_1.BIGINT_0,
        stateRoot: (0, util_1.hexToBytes)('0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544'),
    },
    [exports.Chain.Sepolia]: {
        name: 'sepolia',
        blockNumber: util_1.BIGINT_0,
        stateRoot: (0, util_1.hexToBytes)('0x5eb6e371a698b8d68f665192350ffcecbbbf322916f4b51bd79bb6887da3f494'),
    },
    [exports.Chain.Holesky]: {
        name: 'holesky',
        blockNumber: util_1.BIGINT_0,
        stateRoot: (0, util_1.hexToBytes)('0x69d8c9d72f6fa4ad42d4702b433707212f90db395eb54dc20bc85de253788783'),
    },
    [exports.Chain.Hoodi]: {
        name: 'hoodi',
        blockNumber: util_1.BIGINT_0,
        stateRoot: (0, util_1.hexToBytes)('0xda87d7f5f91c51508791bbcbd4aa5baf04917830b86985eeb9ad3d5bfb657576'),
    },
};
exports.Hardfork = {
    Chainstart: 'chainstart',
    Homestead: 'homestead',
    Dao: 'dao',
    TangerineWhistle: 'tangerineWhistle',
    SpuriousDragon: 'spuriousDragon',
    Byzantium: 'byzantium',
    Constantinople: 'constantinople',
    Petersburg: 'petersburg',
    Istanbul: 'istanbul',
    MuirGlacier: 'muirGlacier',
    Berlin: 'berlin',
    London: 'london',
    ArrowGlacier: 'arrowGlacier',
    GrayGlacier: 'grayGlacier',
    MergeNetsplitBlock: 'mergeNetsplitBlock',
    Paris: 'paris',
    Shanghai: 'shanghai',
    Cancun: 'cancun',
    Prague: 'prague',
    Osaka: 'osaka',
    Bpo1: 'bpo1',
    Bpo2: 'bpo2',
    Bpo3: 'bpo3',
    Bpo4: 'bpo4',
    Bpo5: 'bpo5',
    Amsterdam: 'amsterdam',
};
exports.ConsensusType = {
    ProofOfStake: 'pos',
    ProofOfWork: 'pow',
    ProofOfAuthority: 'poa',
};
exports.ConsensusAlgorithm = {
    Ethash: 'ethash',
    Clique: 'clique',
    Casper: 'casper',
};
//# sourceMappingURL=enums.js.map