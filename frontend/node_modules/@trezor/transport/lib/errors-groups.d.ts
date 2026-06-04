declare const ERRORS_WITHOUT_DEVICE_INTERACTION: ("wrong previous session" | "session not found" | "sessions background did not respond" | "already listening" | "interface not available" | "This transport can not be used in this environment" | "other call in progress" | "Network request failed")[];
export declare const isErrorWithoutDeviceInteraction: (error: string) => error is (typeof ERRORS_WITHOUT_DEVICE_INTERACTION)[number];
export {};
//# sourceMappingURL=errors-groups.d.ts.map