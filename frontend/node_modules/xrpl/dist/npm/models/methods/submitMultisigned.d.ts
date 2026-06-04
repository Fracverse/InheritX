import { APIVersion, DEFAULT_API_VERSION, RIPPLED_API_V1 } from '../common';
import { Transaction } from '../transactions';
import { BaseRequest, BaseResponse } from './baseMethod';
export interface SubmitMultisignedRequest extends BaseRequest {
    command: 'submit_multisigned';
    tx_json: Transaction;
    fail_hard?: boolean;
}
interface BaseSubmitMultisignedResult {
    engine_result: string;
    engine_result_code: number;
    engine_result_message: string;
    tx_blob: string;
    tx_json: Transaction;
}
export interface SubmitMultisignedResponse extends BaseResponse {
    result: BaseSubmitMultisignedResult & {
        hash?: string;
    };
}
export interface SubmitMultisignedV1Response extends BaseResponse {
    result: BaseSubmitMultisignedResult & {
        tx_json: Transaction & {
            hash?: string;
        };
    };
}
export type SubmitMultisignedVersionResponseMap<Version extends APIVersion = typeof DEFAULT_API_VERSION> = Version extends typeof RIPPLED_API_V1 ? SubmitMultisignedV1Response : SubmitMultisignedResponse;
export {};
//# sourceMappingURL=submitMultisigned.d.ts.map