import { BaseRequest, BaseResponse } from './baseMethod';
export interface FeatureAllRequest extends BaseRequest {
    command: 'feature';
    feature?: never;
}
export interface FeatureOneRequest extends BaseRequest {
    command: 'feature';
    feature: string;
}
export type FeatureRequest = FeatureAllRequest | FeatureOneRequest;
export interface FeatureAllResponse extends BaseResponse {
    result: {
        features: Record<string, {
            enabled: boolean;
            name: string;
            supported: boolean;
        }>;
    };
}
export interface FeatureOneResponse extends BaseResponse {
    result: Record<string, {
        enabled: boolean;
        name: string;
        supported: boolean;
    }>;
}
export type FeatureResponse = FeatureAllResponse | FeatureOneResponse;
//# sourceMappingURL=feature.d.ts.map