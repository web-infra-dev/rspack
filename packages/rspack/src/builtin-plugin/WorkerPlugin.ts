import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';

import type { Compiler } from '../Compiler';
import type {
  ChunkLoading,
  OutputModule,
  WorkerPublicPath,
} from '../config';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';
import { EnableChunkLoadingPlugin } from './EnableChunkLoadingPlugin';

export class WorkerPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.WorkerPlugin;
  affectedHooks = 'compilation' as const;

  constructor(
    private chunkLoading: ChunkLoading,
    _wasmLoading: false,
    // @ts-expect-error not implemented
    private module: OutputModule,
    // @ts-expect-error not implemented
    private workerPublicPath: WorkerPublicPath,
  ) {
    super();
  }

  raw(compiler: Compiler): BuiltinPlugin {
    if (this.chunkLoading) {
      new EnableChunkLoadingPlugin(this.chunkLoading).apply(compiler);
    }
    return createBuiltinPlugin(this.name, undefined);
  }
}
