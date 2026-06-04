import type { EthereumNetworkInfo } from '../../types';
import { Blockchain } from '../Blockchain';
import { MiscFeeLevels } from './MiscFeeLevels';
export declare class EthereumFeeLevels extends MiscFeeLevels {
    coinInfo: EthereumNetworkInfo;
    constructor(coinInfo: EthereumNetworkInfo);
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
//# sourceMappingURL=EthereumFeeLevels.d.ts.map