import binding, {
  type BuiltinPlugin,
  BuiltinPluginName,
} from '@rspack/binding';

import * as liteTapable from '@rspack/lite-tapable';
import type { Chunk } from '../Chunk';
import { type Compilation, checkCompilation } from '../Compilation';
import {
  COMPILER_HOOK_USAGE_TRACKERS,
  trackHookUsage,
} from '../HookUsageTracker';
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
      trackHookUsage(
        hooks.chunkHash,
        COMPILER_HOOK_USAGE_TRACKERS.get(compilation.compiler)!,
        binding.RegisterJsTapKind.JavascriptModulesChunkHash,
      );
      compilationHooksMap.set(compilation, hooks);
    }
    return hooks;
  }
}
