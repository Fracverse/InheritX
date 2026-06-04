"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateOracleSet = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const common_1 = require("./common");
const PRICE_DATA_SERIES_MAX_LENGTH = 10;
const SCALE_MAX = 10;
const MINIMUM_ASSET_PRICE_LENGTH = 1;
const MAXIMUM_ASSET_PRICE_LENGTH = 16;
function validateOracleSet(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'OracleDocumentID', common_1.isNumber);
    (0, common_1.validateRequiredField)(tx, 'LastUpdateTime', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'Provider', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'URI', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'AssetClass', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'PriceDataSeries', (value) => {
        if (!(0, common_1.isArray)(value)) {
            throw new errors_1.ValidationError('OracleSet: PriceDataSeries must be an array');
        }
        if (value.length > PRICE_DATA_SERIES_MAX_LENGTH) {
            throw new errors_1.ValidationError(`OracleSet: PriceDataSeries must have at most ${PRICE_DATA_SERIES_MAX_LENGTH} PriceData objects`);
        }
        for (const priceData of value) {
            if (!(0, common_1.isRecord)(priceData)) {
                throw new errors_1.ValidationError('OracleSet: PriceDataSeries must be an array of objects');
            }
            const priceDataInner = priceData.PriceData;
            if (!(0, common_1.isRecord)(priceDataInner)) {
                throw new errors_1.ValidationError('OracleSet: PriceDataSeries must have a `PriceData` object');
            }
            if (Object.keys(priceData).length !== 1) {
                throw new errors_1.ValidationError('OracleSet: PriceDataSeries must only have a single PriceData object');
            }
            if (priceDataInner.BaseAsset == null ||
                typeof priceDataInner.BaseAsset !== 'string') {
                throw new errors_1.ValidationError('OracleSet: PriceDataSeries must have a `BaseAsset` string');
            }
            if (typeof priceDataInner.QuoteAsset !== 'string') {
                throw new errors_1.ValidationError('OracleSet: PriceDataSeries must have a `QuoteAsset` string');
            }
            if ((priceDataInner.AssetPrice == null) !==
                (priceDataInner.Scale == null)) {
                throw new errors_1.ValidationError('OracleSet: PriceDataSeries must have both `AssetPrice` and `Scale` if any are present');
            }
            if ('AssetPrice' in priceDataInner) {
                if (!(0, common_1.isNumber)(priceDataInner.AssetPrice)) {
                    if (typeof priceDataInner.AssetPrice !== 'string') {
                        throw new errors_1.ValidationError('OracleSet: Field AssetPrice must be a string or a number');
                    }
                    if (!(0, utils_1.isHex)(priceDataInner.AssetPrice)) {
                        throw new errors_1.ValidationError('OracleSet: Field AssetPrice must be a valid hex string');
                    }
                    if (priceDataInner.AssetPrice.length < MINIMUM_ASSET_PRICE_LENGTH ||
                        priceDataInner.AssetPrice.length > MAXIMUM_ASSET_PRICE_LENGTH) {
                        throw new errors_1.ValidationError(`OracleSet: Length of AssetPrice field must be between ${MINIMUM_ASSET_PRICE_LENGTH} and ${MAXIMUM_ASSET_PRICE_LENGTH} characters long`);
                    }
                }
            }
            if ('Scale' in priceDataInner) {
                if (!(0, common_1.isNumber)(priceDataInner.Scale)) {
                    throw new errors_1.ValidationError('OracleSet: invalid field Scale');
                }
                if (priceDataInner.Scale < 0 || priceDataInner.Scale > SCALE_MAX) {
                    throw new errors_1.ValidationError(`OracleSet: Scale must be in range 0-${SCALE_MAX}`);
                }
            }
        }
        return true;
    });
}
exports.validateOracleSet = validateOracleSet;
//# sourceMappingURL=oracleSet.js.map