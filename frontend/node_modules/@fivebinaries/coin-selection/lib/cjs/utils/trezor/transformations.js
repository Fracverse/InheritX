"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.drepIdToHex = exports.transformToTrezorOutputs = exports.transformToTrezorInputs = exports.transformToTokenBundle = void 0;
const CardanoWasm = __importStar(require("@emurgo/cardano-serialization-lib-nodejs"));
const types_1 = require("../../types/types");
const common_1 = require("../common");
const transformToTokenBundle = (assets) => {
    // prepare token bundle used in trezor output
    if (assets.length === 0)
        return undefined;
    const uniquePolicies = [];
    assets.forEach(asset => {
        const { policyId } = (0, common_1.parseAsset)(asset.unit);
        if (!uniquePolicies.includes(policyId)) {
            uniquePolicies.push(policyId);
        }
    });
    const assetsByPolicy = [];
    uniquePolicies.forEach(policyId => {
        const assetsInPolicy = [];
        assets.forEach(asset => {
            const assetInfo = (0, common_1.parseAsset)(asset.unit);
            if (assetInfo.policyId !== policyId)
                return;
            assetsInPolicy.push({
                assetNameBytes: assetInfo.assetNameInHex,
                amount: asset.quantity,
            });
        }),
            assetsByPolicy.push({
                policyId,
                tokenAmounts: assetsInPolicy,
            });
    });
    return assetsByPolicy;
};
exports.transformToTokenBundle = transformToTokenBundle;
const transformToTrezorInputs = (utxos, trezorUtxos) => {
    return utxos.map(utxo => {
        const utxoWithPath = trezorUtxos.find(u => u.txid === utxo.txHash && u.vout === utxo.outputIndex);
        // shouldn't happen since utxos should be subset of trezorUtxos (with different shape/fields)
        if (!utxoWithPath)
            throw Error(`Cannot transform utxo ${utxo.txHash}:${utxo.outputIndex}`);
        return {
            path: utxoWithPath.path,
            prev_hash: utxo.txHash,
            prev_index: utxo.outputIndex,
        };
    });
};
exports.transformToTrezorInputs = transformToTrezorInputs;
const transformToTrezorOutputs = (outputs, changeAddressParameters) => {
    return outputs.map(output => {
        let params;
        if (output.isChange) {
            params = {
                addressParameters: changeAddressParameters,
            };
        }
        else {
            params = {
                address: output.address,
            };
        }
        return Object.assign(Object.assign({}, params), { amount: output.amount, tokenBundle: (0, exports.transformToTokenBundle)(output.assets) });
    });
};
exports.transformToTrezorOutputs = transformToTrezorOutputs;
const drepIdToHex = (drepId) => {
    var _a, _b;
    const drep = CardanoWasm.DRep.from_bech32(drepId);
    const kind = drep.kind();
    let drepHex;
    switch (kind) {
        case types_1.CardanoDRepType.KEY_HASH:
            drepHex = (_a = drep.to_key_hash()) === null || _a === void 0 ? void 0 : _a.to_hex();
            break;
        case types_1.CardanoDRepType.SCRIPT_HASH:
            drepHex = (_b = drep.to_script_hash()) === null || _b === void 0 ? void 0 : _b.to_hex();
            break;
    }
    if (!drepHex) {
        throw Error('Invalid drepId');
    }
    const drepData = {
        type: kind,
        hex: drepHex,
    };
    drep.free();
    return drepData;
};
exports.drepIdToHex = drepIdToHex;
