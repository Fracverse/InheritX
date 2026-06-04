"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateMPTokenMetadata = exports.decodeMPTokenMetadata = exports.encodeMPTokenMetadata = exports.MPT_META_WARNING_HEADER = exports.MAX_MPT_META_BYTE_LENGTH = void 0;
const utils_1 = require("@xrplf/isomorphic/utils");
const fast_json_stable_stringify_1 = __importDefault(require("fast-json-stable-stringify"));
const common_1 = require("../transactions/common");
const _1 = require(".");
exports.MAX_MPT_META_BYTE_LENGTH = 1024;
exports.MPT_META_WARNING_HEADER = 'MPTokenMetadata is not properly formatted as JSON as per the XLS-89 standard. ' +
    "While adherence to this standard is not mandatory, such non-compliant MPToken's might not be discoverable " +
    'by Explorers and Indexers in the XRPL ecosystem.';
const MPT_META_URI_FIELDS = [
    {
        long: 'uri',
        compact: 'u',
    },
    {
        long: 'category',
        compact: 'c',
    },
    {
        long: 'title',
        compact: 't',
    },
];
const MPT_META_ALL_FIELDS = [
    {
        long: 'ticker',
        compact: 't',
        validate(obj) {
            var _a;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            if (!(0, common_1.isString)(value) || !/^[A-Z0-9]{1,6}$/u.test(value)) {
                return [
                    `${this.long}/${this.compact}: should have uppercase letters (A-Z) and digits (0-9) only. Max 6 characters recommended.`,
                ];
            }
            return [];
        },
    },
    {
        long: 'name',
        compact: 'n',
        validate(obj) {
            var _a;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            if (!(0, common_1.isString)(value) || value.length === 0) {
                return [`${this.long}/${this.compact}: should be a non-empty string.`];
            }
            return [];
        },
    },
    {
        long: 'icon',
        compact: 'i',
        validate(obj) {
            var _a;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            if (!(0, common_1.isString)(value) || value.length === 0) {
                return [`${this.long}/${this.compact}: should be a non-empty string.`];
            }
            return [];
        },
    },
    {
        long: 'asset_class',
        compact: 'ac',
        validate(obj) {
            var _a;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            const MPT_META_ASSET_CLASSES = [
                'rwa',
                'memes',
                'wrapped',
                'gaming',
                'defi',
                'other',
            ];
            if (!(0, common_1.isString)(value) || !MPT_META_ASSET_CLASSES.includes(value)) {
                return [
                    `${this.long}/${this.compact}: should be one of ${MPT_META_ASSET_CLASSES.join(', ')}.`,
                ];
            }
            return [];
        },
    },
    {
        long: 'issuer_name',
        compact: 'in',
        validate(obj) {
            var _a;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            if (!(0, common_1.isString)(value) || value.length === 0) {
                return [`${this.long}/${this.compact}: should be a non-empty string.`];
            }
            return [];
        },
    },
    {
        long: 'desc',
        compact: 'd',
        validate(obj) {
            var _a;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            if (obj[this.long] === undefined && obj[this.compact] === undefined) {
                return [];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            if (!(0, common_1.isString)(value) || value.length === 0) {
                return [`${this.long}/${this.compact}: should be a non-empty string.`];
            }
            return [];
        },
    },
    {
        long: 'asset_subclass',
        compact: 'as',
        required: false,
        validate(obj) {
            var _a;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            if ((obj.asset_class === 'rwa' || obj.ac === 'rwa') &&
                value === undefined) {
                return [
                    `${this.long}/${this.compact}: required when asset_class is rwa.`,
                ];
            }
            if (obj[this.long] === undefined && obj[this.compact] === undefined) {
                return [];
            }
            const MPT_META_ASSET_SUB_CLASSES = [
                'stablecoin',
                'commodity',
                'real_estate',
                'private_credit',
                'equity',
                'treasury',
                'other',
            ];
            if (!(0, common_1.isString)(value) || !MPT_META_ASSET_SUB_CLASSES.includes(value)) {
                return [
                    `${this.long}/${this.compact}: should be one of ${MPT_META_ASSET_SUB_CLASSES.join(', ')}.`,
                ];
            }
            return [];
        },
    },
    {
        long: 'uris',
        compact: 'us',
        required: false,
        validate(obj) {
            var _a, _b, _c, _d;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            if (obj[this.long] === undefined && obj[this.compact] === undefined) {
                return [];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            if (!Array.isArray(value) || value.length === 0) {
                return [`${this.long}/${this.compact}: should be a non-empty array.`];
            }
            const messages = [];
            for (const uriObj of value) {
                if (!(0, common_1.isRecord)(uriObj) ||
                    Object.keys(uriObj).length !== MPT_META_URI_FIELDS.length) {
                    messages.push(`${this.long}/${this.compact}: should be an array of objects each with uri/u, category/c, and title/t properties.`);
                    continue;
                }
                for (const uriField of MPT_META_URI_FIELDS) {
                    if (uriObj[uriField.long] != null &&
                        uriObj[uriField.compact] != null) {
                        messages.push(`${this.long}/${this.compact}: should not have both ${uriField.long} and ${uriField.compact} fields.`);
                        break;
                    }
                }
                const uri = (_b = uriObj.uri) !== null && _b !== void 0 ? _b : uriObj.u;
                const category = (_c = uriObj.category) !== null && _c !== void 0 ? _c : uriObj.c;
                const title = (_d = uriObj.title) !== null && _d !== void 0 ? _d : uriObj.t;
                if (!(0, common_1.isString)(uri) || !(0, common_1.isString)(category) || !(0, common_1.isString)(title)) {
                    messages.push(`${this.long}/${this.compact}: should be an array of objects each with uri/u, category/c, and title/t properties.`);
                }
            }
            return messages;
        },
    },
    {
        long: 'additional_info',
        compact: 'ai',
        required: false,
        validate(obj) {
            var _a;
            if (obj[this.long] != null && obj[this.compact] != null) {
                return [
                    `${this.long}/${this.compact}: both long and compact forms present. expected only one.`,
                ];
            }
            if (obj[this.long] === undefined && obj[this.compact] === undefined) {
                return [];
            }
            const value = (_a = obj[this.long]) !== null && _a !== void 0 ? _a : obj[this.compact];
            if (!(0, common_1.isString)(value) && !(0, common_1.isRecord)(value)) {
                return [
                    `${this.long}/${this.compact}: should be a string or JSON object.`,
                ];
            }
            return [];
        },
    },
];
function shortenKeys(input, mappings) {
    const output = {};
    for (const [key, value] of Object.entries(input)) {
        const mapping = mappings.find(({ long, compact }) => long === key || compact === key);
        if (mapping === undefined) {
            output[key] = value;
            continue;
        }
        if (input[mapping.long] !== undefined &&
            input[mapping.compact] !== undefined) {
            output[key] = value;
            continue;
        }
        output[mapping.compact] = value;
    }
    return output;
}
function encodeMPTokenMetadata(mptokenMetadata) {
    let input = mptokenMetadata;
    if (!(0, common_1.isRecord)(input)) {
        throw new Error('MPTokenMetadata must be JSON object.');
    }
    input = shortenKeys(input, MPT_META_ALL_FIELDS);
    if (Array.isArray(input.uris)) {
        input.uris = input.uris.map((uri) => {
            if ((0, common_1.isRecord)(uri)) {
                return shortenKeys(uri, MPT_META_URI_FIELDS);
            }
            return uri;
        });
    }
    if (Array.isArray(input.us)) {
        input.us = input.us.map((uri) => {
            if ((0, common_1.isRecord)(uri)) {
                return shortenKeys(uri, MPT_META_URI_FIELDS);
            }
            return uri;
        });
    }
    return (0, utils_1.stringToHex)((0, fast_json_stable_stringify_1.default)(input)).toUpperCase();
}
exports.encodeMPTokenMetadata = encodeMPTokenMetadata;
function expandKeys(input, mappings) {
    const output = {};
    for (const [key, value] of Object.entries(input)) {
        const mapping = mappings.find(({ long, compact }) => long === key || compact === key);
        if (mapping === undefined) {
            output[key] = value;
            continue;
        }
        if (input[mapping.long] !== undefined &&
            input[mapping.compact] !== undefined) {
            output[key] = value;
            continue;
        }
        output[mapping.long] = value;
    }
    return output;
}
function decodeMPTokenMetadata(input) {
    if (!(0, _1.isHex)(input)) {
        throw new Error('MPTokenMetadata must be in hex format.');
    }
    let jsonMetaData;
    try {
        jsonMetaData = JSON.parse((0, utils_1.hexToString)(input));
    }
    catch (err) {
        throw new Error(`MPTokenMetadata is not properly formatted as JSON - ${String(err)}`);
    }
    if (!(0, common_1.isRecord)(jsonMetaData)) {
        throw new Error('MPTokenMetadata must be a JSON object.');
    }
    let output = jsonMetaData;
    output = expandKeys(output, MPT_META_ALL_FIELDS);
    if (Array.isArray(output.uris)) {
        output.uris = output.uris.map((uri) => {
            if ((0, common_1.isRecord)(uri)) {
                return expandKeys(uri, MPT_META_URI_FIELDS);
            }
            return uri;
        });
    }
    if (Array.isArray(output.us)) {
        output.us = output.us.map((uri) => {
            if ((0, common_1.isRecord)(uri)) {
                return expandKeys(uri, MPT_META_URI_FIELDS);
            }
            return uri;
        });
    }
    return output;
}
exports.decodeMPTokenMetadata = decodeMPTokenMetadata;
function validateMPTokenMetadata(input) {
    const validationMessages = [];
    if (!(0, _1.isHex)(input)) {
        validationMessages.push(`MPTokenMetadata must be in hex format.`);
        return validationMessages;
    }
    if (input.length / 2 > exports.MAX_MPT_META_BYTE_LENGTH) {
        validationMessages.push(`MPTokenMetadata must be max ${exports.MAX_MPT_META_BYTE_LENGTH} bytes.`);
        return validationMessages;
    }
    let jsonMetaData;
    try {
        jsonMetaData = JSON.parse((0, utils_1.hexToString)(input));
    }
    catch (err) {
        validationMessages.push(`MPTokenMetadata is not properly formatted as JSON - ${String(err)}`);
        return validationMessages;
    }
    if (!(0, common_1.isRecord)(jsonMetaData)) {
        validationMessages.push('MPTokenMetadata is not properly formatted JSON object as per XLS-89.');
        return validationMessages;
    }
    if (Object.keys(jsonMetaData).length > MPT_META_ALL_FIELDS.length) {
        validationMessages.push(`MPTokenMetadata must not contain more than ${MPT_META_ALL_FIELDS.length} top-level fields (found ${Object.keys(jsonMetaData).length}).`);
    }
    const obj = jsonMetaData;
    for (const property of MPT_META_ALL_FIELDS) {
        validationMessages.push(...property.validate(obj));
    }
    return validationMessages;
}
exports.validateMPTokenMetadata = validateMPTokenMetadata;
//# sourceMappingURL=mptokenMetadata.js.map