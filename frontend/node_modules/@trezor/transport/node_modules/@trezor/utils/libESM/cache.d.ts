export declare class Cache {
    store: Map<string, {
        value: any;
        ttl: number;
    }>;
    constructor();
    set(key: string, value: any, ttl: number): void;
    get(key: string): any;
    delete(key: string): void;
}
//# sourceMappingURL=cache.d.ts.map