import { AlbedoModule } from './modules/albedo.module.mjs';
import { FreighterModule } from './modules/freighter.module.mjs';
import { LobstrModule } from './modules/lobstr.module.mjs';
import { RabetModule } from './modules/rabet.module.mjs';
import { xBullModule } from './modules/xbull.module.mjs';
import { HotWalletModule } from './modules/hotwallet.module.mjs';
import { HanaModule } from './modules/hana.module.mjs';
import { KleverModule } from './modules/klever.module.mjs';

function allowAllModules(opts) {
  const modules = [
    new AlbedoModule(),
    new FreighterModule(),
    new RabetModule(),
    new xBullModule(),
    new LobstrModule(),
    new HanaModule(),
    new HotWalletModule(),
    new KleverModule()
  ];
  return opts?.filterBy ? modules.filter(opts.filterBy) : modules;
}
function sep43Modules(opts) {
  const modules = [new FreighterModule(), new HotWalletModule()];
  return opts?.filterBy ? modules.filter(opts.filterBy) : modules;
}
function parseError(e) {
  return {
    code: e?.error?.code || e?.code || -1,
    message: e?.error?.message || e?.message || typeof e === "string" && e || "Unhandled error from the wallet",
    ext: e?.error?.ext || e?.ext
  };
}

export { allowAllModules, parseError, sep43Modules };
//# sourceMappingURL=utils.mjs.map
