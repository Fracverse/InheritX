import { OUTPUT_SCRIPT_LENGTH } from './coinselect/coinselectUtils';
declare const SCRIPT_TYPES: {
    p2pkh: string;
    p2sh: string;
    p2tr: string;
    p2wpkh: string;
};
type Input = {
    script_type: string;
    multisig?: {
        nodes?: any[];
        pubkeys: any[];
        m: number;
    };
    witness?: Buffer[];
    ownership_proof?: any;
};
export declare class TxWeightCalculator {
    inputs_count: number;
    outputs_count: number;
    counter: number;
    segwit_inputs_count: number;
    inputs: {
        length: number;
    }[];
    addInputByKey(type: keyof typeof SCRIPT_TYPES): void;
    addInput(input: Input): void;
    addOutputByKey(key: keyof typeof OUTPUT_SCRIPT_LENGTH): void;
    addOutput(script: {
        length: number;
    }): void;
    getTotal(): number;
    getVirtualBytes(): number;
}
export {};
//# sourceMappingURL=txWeightCalculator.d.ts.map