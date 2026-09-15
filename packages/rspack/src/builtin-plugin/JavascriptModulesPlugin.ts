import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';

import * as liteTapable from '@rspack/lite-tapable';
import type { Chunk } from '../Chunk';
import { type Compilation, checkCompilation } from '../Compilation';
import type { Filename, OutputNormalized } from '../config';
import type Hash from '../util/hash';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export type CompilationHooks = {
  chunkHash: liteTapable.SyncHook<[Chunk, Hash]>;
};

const compilationHooksMap: WeakMap<Compilation, CompilationHooks> =
  new WeakMap();

export class JavascriptModulesPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.JavascriptModulesPlugin;
  affectedHooks = 'compilation' as const;

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, undefined);
  }

  static getCompilationHooks(compilation: Compilation) {
    checkCompilation(compilation);

    let hooks = compilationHooksMap.get(compilation);
    if (hooks === undefined) {
      hooks = {
        chunkHash: new liteTapable.SyncHook(['chunk', 'hash']),
      };
      compilationHooksMap.set(compilation, hooks);
    }
    return hooks;
  }

  /**
   * Returns the filename template that is used to render the JavaScript file
   * of the given chunk.
   *
   * Mirrors the resolution order of `get_js_chunk_filename_template` in
   * `crates/rspack_core/src/options/output.rs`, which itself aligns with
   * webpack's `JavascriptModulesPlugin.getChunkFilenameTemplate`.
   *
   * Note: webpack additionally returns `output.hotUpdateChunkFilename` for
   * `HotUpdateChunk` instances. Rspack keeps hot update chunks inside the Rust
   * HMR pipeline and never surfaces them to the JavaScript side, so no chunk
   * reachable from here can be a hot update chunk.
   */
  static getChunkFilenameTemplate(
    chunk: Chunk,
    outputOptions: OutputNormalized,
  ): Filename | undefined {
    if (chunk.filenameTemplate) {
      return chunk.filenameTemplate;
    }
    if (chunk.canBeInitial()) {
      return outputOptions.filename;
    }
    return outputOptions.chunkFilename;
  }
}
