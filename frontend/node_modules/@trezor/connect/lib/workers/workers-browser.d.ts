import type { BaseWorker } from '@trezor/blockchain-link/lib/workers/baseWorker';
import BlockbookWorker from '@trezor/blockchain-link/lib/workers/blockbook';
import BlockfrostWorker from '@trezor/blockchain-link/lib/workers/blockfrost';
import RippleWorker from '@trezor/blockchain-link/lib/workers/ripple';
import StellarWorker from '@trezor/blockchain-link/lib/workers/stellar';
type WorkerAsyncImporter = () => Promise<BaseWorker<unknown>>;
declare const SolanaWorker: WorkerAsyncImporter;
declare const ElectrumWorker: undefined;
export { BlockbookWorker, RippleWorker, BlockfrostWorker, ElectrumWorker, SolanaWorker, StellarWorker, };
//# sourceMappingURL=workers-browser.d.ts.map