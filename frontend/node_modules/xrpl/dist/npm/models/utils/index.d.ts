import type { Transaction } from '../transactions';
export declare const INTEGER_SANITY_CHECK: RegExp;
export declare function onlyHasFields(obj: Record<string, unknown>, fields: string[]): boolean;
export declare function isFlagEnabled(Flags: number, checkFlag: number): boolean;
export declare function hasFlag(tx: Transaction | Record<string, unknown>, flag: number, flagName: string): boolean;
export declare function isHex(str: string): boolean;
//# sourceMappingURL=index.d.ts.map