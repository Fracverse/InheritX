import { CardanoAddressParameters, CardanoInput, CardanoOutput } from '../../types/trezor';
import { Asset, CardanoDRepType, FinalOutput, Utxo } from '../../types/types';
interface AssetInPolicy {
    assetNameBytes: string;
    amount: string;
}
export declare const transformToTokenBundle: (assets: Asset[]) => {
    policyId: string;
    tokenAmounts: AssetInPolicy[];
}[] | undefined;
export declare const transformToTrezorInputs: (utxos: Utxo[], trezorUtxos: {
    txid: string;
    vout: number;
    path: string;
}[]) => CardanoInput[];
export declare const transformToTrezorOutputs: (outputs: FinalOutput[], changeAddressParameters: CardanoAddressParameters) => CardanoOutput[];
export declare const drepIdToHex: (drepId: string) => {
    type: CardanoDRepType.KEY_HASH | CardanoDRepType.SCRIPT_HASH;
    hex: string;
};
export {};
