"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stringToDeviceModelId = void 0;
exports.isDeviceModelId = isDeviceModelId;
const _1 = require(".");
function isDeviceModelId(val) {
    if (!val)
        return false;
    return Object.values(_1.DeviceModelId).includes(val);
}
const stringToDeviceModelId = (strDeviceModelId, defaultDeviceModelId) => {
    if (!isDeviceModelId(strDeviceModelId))
        return defaultDeviceModelId;
    return _1.DeviceModelId[strDeviceModelId];
};
exports.stringToDeviceModelId = stringToDeviceModelId;
//# sourceMappingURL=helpers.js.map