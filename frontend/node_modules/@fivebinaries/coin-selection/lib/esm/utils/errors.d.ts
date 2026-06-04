import { ERROR } from '../constants';
type ErrorTypeKeys = keyof typeof ERROR;
type ErrorObjectType = typeof ERROR[ErrorTypeKeys];
export declare class CoinSelectionError extends Error {
    code: ErrorObjectType['code'];
    constructor(errorObject: ErrorObjectType);
}
export {};
