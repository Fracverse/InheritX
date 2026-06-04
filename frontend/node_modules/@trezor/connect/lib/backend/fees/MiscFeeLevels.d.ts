import type { CoinInfo, FeeLevel } from '../../types';
import { Blockchain } from '../Blockchain';
export declare class MiscFeeLevels {
    coinInfo: CoinInfo;
    levels: FeeLevel[];
    wasFetchedSuccessfully: boolean;
    constructor(coinInfo: CoinInfo);
    load(blockchain: Blockchain, request: Parameters<typeof blockchain.estimateFee>[0]): Promise<{
        feePerTx?: string | undefined;
        feeLimit?: string | undefined;
        baseFeePerGas?: string | undefined;
        maxFeePerGas?: string | undefined;
        maxPriorityFeePerGas?: string | undefined;
        label: "normal" | "custom" | "high" | "economy" | "low";
        blocks: number;
        feePerUnit: string;
    }[]>;
}
//# sourceMappingURL=MiscFeeLevels.d.ts.map