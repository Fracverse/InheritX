import type { ConnectSettings, LocalFirmwares } from '../types/settings';
export declare const DEFAULT_PRIORITY = 2;
export declare const parseLocalFirmwares: (localFirmwares: LocalFirmwares) => {
    firmwareDir: string;
    firmwareList: string[];
} | undefined;
export declare const corsValidator: (url?: string) => string | undefined;
export declare const parseConnectSettings: (input?: Partial<ConnectSettings>) => ConnectSettings;
//# sourceMappingURL=connectSettings.d.ts.map