'use strict';

var albedo_module = require('./modules/albedo.module.cjs');
var freighter_module = require('./modules/freighter.module.cjs');
var lobstr_module = require('./modules/lobstr.module.cjs');
var rabet_module = require('./modules/rabet.module.cjs');
var xbull_module = require('./modules/xbull.module.cjs');
var hotwallet_module = require('./modules/hotwallet.module.cjs');
var hana_module = require('./modules/hana.module.cjs');
var klever_module = require('./modules/klever.module.cjs');

/**
 * This method returns all modules that don't require extra configuration before they can be loaded
 * You can provide a filter function if needed
 */
function allowAllModules(opts) {
    const modules = [
        new albedo_module.AlbedoModule(),
        new freighter_module.FreighterModule(),
        new rabet_module.RabetModule(),
        new xbull_module.xBullModule(),
        new lobstr_module.LobstrModule(),
        new hana_module.HanaModule(),
        new hotwallet_module.HotWalletModule(),
        new klever_module.KleverModule(),
    ];
    return opts?.filterBy ? modules.filter(opts.filterBy) : modules;
}
/**
 * This method only returns those modules from wallet that follow exactly the SEP-43 standard and don't require extra configuration before they can be loaded
 * You can provide a filter function if needed
 */
function sep43Modules(opts) {
    const modules = [new freighter_module.FreighterModule(), new hotwallet_module.HotWalletModule()];
    return opts?.filterBy ? modules.filter(opts.filterBy) : modules;
}
function parseError(e) {
    return {
        code: e?.error?.code || e?.code || -1,
        message: e?.error?.message || e?.message || (typeof e === 'string' && e) || 'Unhandled error from the wallet',
        ext: e?.error?.ext || e?.ext,
    };
}

exports.allowAllModules = allowAllModules;
exports.parseError = parseError;
exports.sep43Modules = sep43Modules;
//# sourceMappingURL=utils.cjs.map
