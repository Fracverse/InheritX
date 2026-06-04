export declare enum IntlErrorCode {
    MISSING_MESSAGE = "MISSING_MESSAGE",
    MISSING_FORMAT = "MISSING_FORMAT",
    ENVIRONMENT_FALLBACK = "ENVIRONMENT_FALLBACK",
    INSUFFICIENT_PATH = "INSUFFICIENT_PATH",
    INVALID_MESSAGE = "INVALID_MESSAGE",
    INVALID_KEY = "INVALID_KEY",
    FORMATTING_ERROR = "FORMATTING_ERROR"
}
export default class IntlError extends Error {
    readonly code: IntlErrorCode;
    readonly originalMessage: string | undefined;
    constructor(code: IntlErrorCode, originalMessage?: string);
}
