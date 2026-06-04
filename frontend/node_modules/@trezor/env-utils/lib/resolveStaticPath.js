"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.resolveConnectPath = exports.resolveStaticPath = void 0;
const getEnvAssetPrefix = () => typeof process !== 'undefined' ? process.env?.ASSET_PREFIX : undefined;
const getDefaultPrefix = () => getEnvAssetPrefix() ?? '';
const resolveStaticPath = (path, pathPrefix = getDefaultPrefix()) => `${pathPrefix}/static/${path.replace(/^\/+/, '')}`;
exports.resolveStaticPath = resolveStaticPath;
const resolveConnectPath = (path, pathPrefix = getDefaultPrefix()) => `${pathPrefix}/${path.replace(/^\/+/, '')}`;
exports.resolveConnectPath = resolveConnectPath;
//# sourceMappingURL=resolveStaticPath.js.map