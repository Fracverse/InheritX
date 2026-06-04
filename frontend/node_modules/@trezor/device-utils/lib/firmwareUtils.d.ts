import { FirmwareSource, FirmwareVersionString, PartialDevice, VersionArray } from './types';
export declare const getFirmwareSource: (device?: PartialDevice) => FirmwareSource;
export declare const getFirmwareRevision: (device?: PartialDevice) => string;
export declare const getFirmwareVersionArray: (device?: PartialDevice) => VersionArray | null;
export declare const getFirmwareVersion: (device?: PartialDevice) => "" | FirmwareVersionString;
export declare const hasBitcoinOnlyFirmware: (device?: PartialDevice) => boolean;
export declare const isBitcoinOnlyDevice: (device?: PartialDevice) => boolean;
//# sourceMappingURL=firmwareUtils.d.ts.map