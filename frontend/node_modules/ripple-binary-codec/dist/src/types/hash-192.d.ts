import { Hash } from './hash';
/**
 * Hash with a width of 192 bits
 */
declare class Hash192 extends Hash {
    static readonly width = 24;
    static readonly ZERO_192: Hash192;
    constructor(bytes?: Uint8Array);
}
export { Hash192 };
