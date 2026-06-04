"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CardanoDRepType = exports.CardanoAddressType = void 0;
var CardanoAddressType;
(function (CardanoAddressType) {
    CardanoAddressType[CardanoAddressType["BASE"] = 0] = "BASE";
    CardanoAddressType[CardanoAddressType["BASE_SCRIPT_KEY"] = 1] = "BASE_SCRIPT_KEY";
    CardanoAddressType[CardanoAddressType["BASE_KEY_SCRIPT"] = 2] = "BASE_KEY_SCRIPT";
    CardanoAddressType[CardanoAddressType["BASE_SCRIPT_SCRIPT"] = 3] = "BASE_SCRIPT_SCRIPT";
    CardanoAddressType[CardanoAddressType["POINTER"] = 4] = "POINTER";
    CardanoAddressType[CardanoAddressType["POINTER_SCRIPT"] = 5] = "POINTER_SCRIPT";
    CardanoAddressType[CardanoAddressType["ENTERPRISE"] = 6] = "ENTERPRISE";
    CardanoAddressType[CardanoAddressType["ENTERPRISE_SCRIPT"] = 7] = "ENTERPRISE_SCRIPT";
    CardanoAddressType[CardanoAddressType["BYRON"] = 8] = "BYRON";
    CardanoAddressType[CardanoAddressType["REWARD"] = 14] = "REWARD";
    CardanoAddressType[CardanoAddressType["REWARD_SCRIPT"] = 15] = "REWARD_SCRIPT";
})(CardanoAddressType || (exports.CardanoAddressType = CardanoAddressType = {}));
var CardanoDRepType;
(function (CardanoDRepType) {
    CardanoDRepType[CardanoDRepType["KEY_HASH"] = 0] = "KEY_HASH";
    CardanoDRepType[CardanoDRepType["SCRIPT_HASH"] = 1] = "SCRIPT_HASH";
    CardanoDRepType[CardanoDRepType["ABSTAIN"] = 2] = "ABSTAIN";
    CardanoDRepType[CardanoDRepType["NO_CONFIDENCE"] = 3] = "NO_CONFIDENCE";
})(CardanoDRepType || (exports.CardanoDRepType = CardanoDRepType = {}));
