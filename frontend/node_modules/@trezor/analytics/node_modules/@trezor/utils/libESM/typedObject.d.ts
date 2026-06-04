export declare const typedObjectEntries: <T extends Record<string, unknown>>(obj: T) => [keyof T, T[keyof T]][];
export declare function typedObjectFromEntries<T extends readonly (readonly [string, any])[]>(entries: T): {
    [K in T[number] as K[0]]: K[1];
};
export declare const typedObjectKeys: <T extends Record<any, any>>(obj: T) => Array<keyof T>;
export declare const typedObjectValues: <T extends Record<any, any>>(obj: T) => Array<T[keyof T]>;
//# sourceMappingURL=typedObject.d.ts.map