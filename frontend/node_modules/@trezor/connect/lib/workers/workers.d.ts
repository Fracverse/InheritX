import type { BaseWorker } from '@trezor/blockchain-link/lib/workers/baseWorker';
type WorkerAsyncImporter = () => Promise<BaseWorker<unknown>>;
declare const BlockbookWorker: WorkerAsyncImporter;
declare const RippleWorker: WorkerAsyncImporter;
declare const BlockfrostWorker: WorkerAsyncImporter;
declare const ElectrumWorker: WorkerAsyncImporter;
declare const SolanaWorker: WorkerAsyncImporter;
declare const StellarWorker: WorkerAsyncImporter;
export { BlockbookWorker, RippleWorker, BlockfrostWorker, ElectrumWorker, SolanaWorker, StellarWorker, };
//# sourceMappingURL=workers.d.ts.map