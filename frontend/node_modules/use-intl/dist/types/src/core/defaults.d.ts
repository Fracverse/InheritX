import IntlError from './IntlError';
/**
 * Contains defaults that are used for all entry points into the core.
 * See also `InitializedIntlConfiguration`.
 */
export declare function defaultGetMessageFallback(props: {
    error: IntlError;
    key: string;
    namespace?: string;
}): string;
export declare function defaultOnError(error: IntlError): void;
