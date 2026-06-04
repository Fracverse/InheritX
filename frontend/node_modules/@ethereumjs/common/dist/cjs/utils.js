"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getPresetChainConfig = void 0;
exports.parseGethGenesis = parseGethGenesis;
const util_1 = require("@ethereumjs/util");
const chains_ts_1 = require("./chains.js");
const enums_ts_1 = require("./enums.js");
const hardforks_ts_1 = require("./hardforks.js");
/**
 * Transforms Geth formatted nonce (i.e. hex string) to 8 byte 0x-prefixed string used internally
 * @param nonce string parsed from the Geth genesis file
 * @returns nonce as a 0x-prefixed 8 byte string
 */
function formatNonce(nonce) {
    if (!nonce || nonce === '0x0') {
        return '0x0000000000000000';
    }
    if ((0, util_1.isHexString)(nonce)) {
        return `0x${(0, util_1.stripHexPrefix)(nonce).padStart(16, '0')}`;
    }
    return `0x${nonce.padStart(16, '0')}`;
}
/**
 * Converts Geth genesis parameters to an EthereumJS compatible `CommonOpts` object
 * @param gethGenesis GethGenesis object
 * @returns genesis parameters in a `CommonOpts` compliant object
 */
function parseGethParams(gethGenesis) {
    const { name, config, difficulty, mixHash, gasLimit, coinbase, baseFeePerGas, excessBlobGas, requestsHash, extraData: unparsedExtraData, nonce: unparsedNonce, timestamp: unparsedTimestamp, } = gethGenesis;
    const genesisTimestamp = Number(unparsedTimestamp);
    const { chainId, depositContractAddress } = config;
    // geth is not strictly putting empty fields with a 0x prefix
    const extraData = (0, util_1.addHexPrefix)(unparsedExtraData ?? '');
    // geth may use number for timestamp
    const timestamp = (0, util_1.isHexString)(unparsedTimestamp)
        ? unparsedTimestamp
        : (0, util_1.intToHex)(parseInt(unparsedTimestamp));
    // geth may not give us a nonce strictly formatted to an 8 byte 0x-prefixed hex string
    const nonce = unparsedNonce.length !== 18 ? formatNonce(unparsedNonce) : (0, util_1.addHexPrefix)(unparsedNonce);
    // EIP155 and EIP158 are both part of Spurious Dragon hardfork and must occur at the same time
    // but have different configuration parameters in geth genesis parameters
    if (config.eip155Block !== config.eip158Block) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('EIP155 block number must equal EIP 158 block number since both are part of SpuriousDragon hardfork and the client only supports activating the full hardfork');
    }
    let customHardforks = undefined;
    if (config.blobSchedule !== undefined) {
        customHardforks = {};
        const blobGasPerBlob = 131072;
        for (const [hfKey, hfSchedule] of Object.entries(config.blobSchedule)) {
            const hfConfig = hardforks_ts_1.hardforksDict[hfKey];
            if (hfConfig === undefined) {
                throw (0, util_1.EthereumJSErrorWithoutCode)(`unknown hardfork=${hfKey} specified in blobSchedule`);
            }
            const { target, max, baseFeeUpdateFraction: blobGasPriceUpdateFraction } = hfSchedule;
            if (target === undefined || max === undefined || blobGasPriceUpdateFraction === undefined) {
                throw (0, util_1.EthereumJSErrorWithoutCode)(`undefined target, max or baseFeeUpdateFraction specified in blobSchedule for hardfork=${hfKey}`);
            }
            // copy current hardfork info to custom and add blob config
            const customHfConfig = JSON.parse(JSON.stringify(hfConfig));
            customHfConfig.params = {
                ...customHardforks.params,
                // removes blobGasPriceUpdateFraction key to prevent undefined overriding if undefined
                ...{
                    targetBlobGasPerBlock: blobGasPerBlob * target,
                    maxBlobGasPerBlock: blobGasPerBlob * max,
                    blobGasPriceUpdateFraction,
                },
            };
            customHardforks[hfKey] = customHfConfig;
        }
    }
    const params = {
        name,
        chainId,
        depositContractAddress,
        genesis: {
            timestamp,
            gasLimit,
            difficulty,
            nonce,
            extraData,
            mixHash,
            coinbase,
            baseFeePerGas,
            excessBlobGas,
            requestsHash,
        },
        hardfork: undefined,
        hardforks: [],
        customHardforks,
        bootstrapNodes: [],
        consensus: config.clique !== undefined
            ? {
                type: 'poa',
                algorithm: 'clique',
                clique: {
                    // The recent geth genesis seems to be using blockperiodseconds // cspell:disable-line
                    // and epochlength for clique specification
                    // see: https://hackmd.io/PqZgMpnkSWCWv5joJoFymQ
                    period: config.clique.period ?? config.clique.blockperiodseconds, // cspell:disable-line
                    epoch: config.clique.epoch ?? config.clique.epochlength,
                },
            }
            : {
                type: 'pow',
                algorithm: 'ethash',
                ethash: {},
            },
    };
    const forkMap = {
        [enums_ts_1.Hardfork.Homestead]: { name: 'homesteadBlock' },
        [enums_ts_1.Hardfork.Dao]: { name: 'daoForkBlock' },
        [enums_ts_1.Hardfork.TangerineWhistle]: { name: 'eip150Block' },
        [enums_ts_1.Hardfork.SpuriousDragon]: { name: 'eip155Block' },
        [enums_ts_1.Hardfork.Byzantium]: { name: 'byzantiumBlock' },
        [enums_ts_1.Hardfork.Constantinople]: { name: 'constantinopleBlock' },
        [enums_ts_1.Hardfork.Petersburg]: { name: 'petersburgBlock' },
        [enums_ts_1.Hardfork.Istanbul]: { name: 'istanbulBlock' },
        [enums_ts_1.Hardfork.MuirGlacier]: { name: 'muirGlacierBlock' },
        [enums_ts_1.Hardfork.Berlin]: { name: 'berlinBlock' },
        [enums_ts_1.Hardfork.London]: { name: 'londonBlock' },
        [enums_ts_1.Hardfork.ArrowGlacier]: { name: 'arrowGlacierBlock' },
        [enums_ts_1.Hardfork.GrayGlacier]: { name: 'grayGlacierBlock' },
        [enums_ts_1.Hardfork.Paris]: { name: 'mergeForkBlock', postMerge: true },
        [enums_ts_1.Hardfork.MergeNetsplitBlock]: { name: 'mergeNetsplitBlock', postMerge: true },
        [enums_ts_1.Hardfork.Shanghai]: { name: 'shanghaiTime', postMerge: true, isTimestamp: true },
        [enums_ts_1.Hardfork.Cancun]: { name: 'cancunTime', postMerge: true, isTimestamp: true },
        [enums_ts_1.Hardfork.Prague]: { name: 'pragueTime', postMerge: true, isTimestamp: true },
        [enums_ts_1.Hardfork.Osaka]: { name: 'osakaTime', postMerge: true, isTimestamp: true },
        [enums_ts_1.Hardfork.Bpo1]: { name: 'bpo1Time', postMerge: true, isTimestamp: true },
        [enums_ts_1.Hardfork.Bpo2]: { name: 'bpo2Time', postMerge: true, isTimestamp: true },
        [enums_ts_1.Hardfork.Bpo3]: { name: 'bpo3Time', postMerge: true, isTimestamp: true },
        [enums_ts_1.Hardfork.Bpo4]: { name: 'bpo4Time', postMerge: true, isTimestamp: true },
        [enums_ts_1.Hardfork.Bpo5]: { name: 'bpo5Time', postMerge: true, isTimestamp: true },
    };
    // forkMapRev is the map from config field name to Hardfork
    const forkMapRev = Object.keys(forkMap).reduce((acc, elem) => {
        acc[forkMap[elem].name] = elem;
        return acc;
    }, {});
    params.hardforks = Object.entries(forkMapRev)
        .map(([nameBlock, hardfork]) => {
        const configValue = config[nameBlock];
        const isTimestamp = forkMap[hardfork].isTimestamp === true;
        const block = isTimestamp || typeof configValue !== 'number' ? null : configValue;
        const timestamp = isTimestamp && typeof configValue === 'number' ? configValue : undefined;
        return { name: hardfork, block, timestamp };
    })
        .filter(({ block, timestamp }) => block !== null || timestamp !== undefined);
    const mergeIndex = params.hardforks.findIndex((hf) => hf.name === enums_ts_1.Hardfork.Paris);
    let mergeNetsplitBlockIndex = params.hardforks.findIndex((hf) => hf.name === enums_ts_1.Hardfork.MergeNetsplitBlock);
    const firstPostMergeHFIndex = params.hardforks.findIndex((hf) => hf.timestamp !== undefined && hf.timestamp !== null);
    // If we are missing a mergeNetsplitBlock, we assume it is at the same block as Paris (if present)
    if (mergeIndex !== -1 && mergeNetsplitBlockIndex === -1) {
        params.hardforks.splice(mergeIndex + 1, 0, {
            name: enums_ts_1.Hardfork.MergeNetsplitBlock,
            block: params.hardforks[mergeIndex].block,
        });
        mergeNetsplitBlockIndex = mergeIndex + 1;
    }
    // or zero if not and a postmerge hardfork is set (since testnets using the geth genesis format are all currently start postmerge)
    if (firstPostMergeHFIndex !== -1) {
        if (mergeNetsplitBlockIndex === -1) {
            params.hardforks.splice(firstPostMergeHFIndex, 0, {
                name: enums_ts_1.Hardfork.MergeNetsplitBlock,
                block: 0,
            });
            mergeNetsplitBlockIndex = firstPostMergeHFIndex;
        }
        if (mergeIndex === -1) {
            // If we don't have a Paris hardfork, add it at the mergeNetsplitBlock
            params.hardforks.splice(mergeNetsplitBlockIndex, 0, {
                name: enums_ts_1.Hardfork.Paris,
                block: params.hardforks[mergeNetsplitBlockIndex].block,
            });
        }
        // Check for terminalTotalDifficultyPassed param in genesis config if no post merge hardforks are set
    }
    else if (config.terminalTotalDifficultyPassed === true) {
        if (mergeIndex === -1) {
            // If we don't have a Paris hardfork, add it at end of hardfork array
            params.hardforks.push({
                name: enums_ts_1.Hardfork.Paris,
                block: 0,
            });
        }
        // If we don't have a MergeNetsplitBlock hardfork, add it at end of hardfork array
        if (mergeNetsplitBlockIndex === -1) {
            params.hardforks.push({
                name: enums_ts_1.Hardfork.MergeNetsplitBlock,
                block: 0,
            });
            mergeNetsplitBlockIndex = firstPostMergeHFIndex;
        }
    }
    // TODO: Decide if we actually need to do this since `ForkMap` specifies the order we expect things in
    params.hardforks.sort(function (a, b) {
        return (a.block ?? Infinity) - (b.block ?? Infinity);
    });
    params.hardforks.sort(function (a, b) {
        // non timestamp forks come before any timestamp forks
        return (a.timestamp ?? 0) - (b.timestamp ?? 0);
    });
    // only set the genesis timestamp forks to zero post the above sort has happened
    // to get the correct sorting
    for (const hf of params.hardforks) {
        if (hf.timestamp === genesisTimestamp) {
            hf.timestamp = 0;
        }
    }
    const latestHardfork = params.hardforks.length > 0 ? params.hardforks.slice(-1)[0] : undefined;
    params.hardfork = latestHardfork?.name;
    params.hardforks.unshift({ name: enums_ts_1.Hardfork.Chainstart, block: 0 });
    return params;
}
/**
 * Parses a genesis object exported from Geth into parameters for Common instance
 * @param gethGenesis GethGenesis object
 * @param name optional chain name
 * @returns parsed params
 */
function parseGethGenesis(gethGenesis, name) {
    try {
        const required = ['config', 'difficulty', 'gasLimit', 'nonce', 'alloc'];
        if (required.some((field) => !(field in gethGenesis))) {
            const missingField = required.filter((field) => !(field in gethGenesis));
            throw (0, util_1.EthereumJSErrorWithoutCode)(`Invalid format, expected geth genesis field "${missingField}" missing`);
        }
        // We copy the object here because it's frozen in browser and properties can't be modified
        const finalGethGenesis = { ...gethGenesis };
        if (name !== undefined) {
            finalGethGenesis.name = name;
        }
        return parseGethParams(finalGethGenesis);
    }
    catch (e) {
        throw (0, util_1.EthereumJSErrorWithoutCode)(`Error parsing parameters file: ${e.message}`);
    }
}
/**
 * Return the preset chain config for one of the predefined chain configurations
 * @param chain the representing a network name (e.g. 'mainnet') or number representing the chain ID
 * @returns a {@link ChainConfig}
 */
const getPresetChainConfig = (chain) => {
    switch (chain) {
        case 'holesky':
        case 17000:
            return chains_ts_1.Holesky;
        case 'hoodi':
        case 560048:
            return chains_ts_1.Hoodi;
        case 'sepolia':
        case 11155111:
            return chains_ts_1.Sepolia;
        case 'mainnet':
        case 1:
        default:
            return chains_ts_1.Mainnet;
    }
};
exports.getPresetChainConfig = getPresetChainConfig;
//# sourceMappingURL=utils.js.map