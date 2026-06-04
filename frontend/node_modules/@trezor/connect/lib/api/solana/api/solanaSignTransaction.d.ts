import { TokenInfo } from '@trezor/blockchain-link-types';
import { PROTO } from '../../../constants';
import { AbstractMethod } from '../../../core/AbstractMethod';
type Params = PROTO.SolanaSignTx & {
    serialize: boolean;
};
export default class SolanaSignTransaction extends AbstractMethod<'solanaSignTransaction', Params> {
    init(): void;
    initAsync(): Promise<void>;
    get info(): string;
    payloadToPrecomposed(): Promise<undefined> | Promise<{
        type: "final";
        inputs: never[];
        outputsPermutation: number[];
        outputs: {
            address: string;
            amount: string;
            script_type: "PAYTOADDRESS";
        }[];
        totalSpent: string;
        fee: string;
        feePerByte: string;
        feeLimit: string;
        bytes: number;
        max: undefined;
        isTokenKnown: boolean;
        token: TokenInfo | undefined;
        createdTimestamp: number | undefined;
    }>;
    run(): Promise<{
        signature: string;
        serializedTx: string;
    } | {
        signature: string;
        serializedTx?: undefined;
    }>;
}
export {};
//# sourceMappingURL=solanaSignTransaction.d.ts.map