import type { BitcoinNetworkInfo } from '../../types';
import { Blockchain } from '../Blockchain';
import { MiscFeeLevels } from './MiscFeeLevels';
export declare class BitcoinFeeLevels extends MiscFeeLevels {
    coinInfo: BitcoinNetworkInfo;
    longTermFeeRate: string;
    constructor(coinInfo: BitcoinNetworkInfo);
    load(blockchain: Blockchain): Promise<{
        feePerTx?: string | undefined;
        feeLimit?: string | undefined;
        baseFeePerGas?: string | undefined;
        maxFeePerGas?: string | undefined;
        maxPriorityFeePerGas?: string | undefined;
        label: "normal" | "custom" | "high" | "economy" | "low";
        blocks: number;
        feePerUnit: string;
    }[]>;
    updateBitcoinCustomFee(feePerUnit: string): void;
}
//# sourceMappingURL=BitcoinFeeLevels.d.ts.map