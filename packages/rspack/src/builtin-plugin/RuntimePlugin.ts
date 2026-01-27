import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';

import type { Chunk } from '../Chunk';
import { type Compilation, checkCompilation } from '../Compilation';
import type { CreatePartialRegisters } from '../taps/types';
import { create } from './base';

export const RuntimePluginImpl = create(
  binding.BuiltinPluginName.RuntimePlugin,
  () => {},
  'compilation',
);

export type RuntimePluginHooks = {
  createScript: liteTapable.SyncWaterfallHook<[string, Chunk]>;
  createLink: liteTapable.SyncWaterfallHook<[string, Chunk]>;
  linkPreload: liteTapable.SyncWaterfallHook<[string, Chunk]>;
  linkPrefetch: liteTapable.SyncWaterfallHook<[string, Chunk]>;
};

const RuntimePlugin = RuntimePluginImpl as typeof RuntimePluginImpl & {
  /**
   * @deprecated Use `getCompilationHooks` instead.
   */
  getHooks: (compilation: Compilation) => RuntimePluginHooks;
  getCompilationHooks: (compilation: Compilation) => RuntimePluginHooks;
};

const compilationHooksMap: WeakMap<Compilation, RuntimePluginHooks> =
  new WeakMap();

RuntimePlugin.getHooks = RuntimePlugin.getCompilationHooks = (
  compilation: Compilation,
) => {
  checkCompilation(compilation);

  let hooks = compilationHooksMap.get(compilation);
  if (hooks === undefined) {
    hooks = {
      createScript: new liteTapable.SyncWaterfallHook(['code', 'chunk']),
      createLink: new liteTapable.SyncWaterfallHook(['code', 'chunk']),
      linkPreload: new liteTapable.SyncWaterfallHook(['code', 'chunk']),
      linkPrefetch: new liteTapable.SyncWaterfallHook(['code', 'chunk']),
    };
    compilationHooksMap.set(compilation, hooks);
  }
  return hooks;
};

export const createRuntimePluginHooksRegisters: CreatePartialRegisters<
  `RuntimePlugin`
> = (getCompiler, createTap) => {
  return {
    registerRuntimePluginCreateScriptTaps: createTap(
      binding.RegisterJsTapKind.RuntimePluginCreateScript,
      function () {
        return RuntimePlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).createScript;
      },
      function (queried) {
        return function (data: binding.JsCreateScriptData) {
          return queried.call(data.code, data.chunk);
        };
      },
    ),
    registerRuntimePluginCreateLinkTaps: createTap(
      binding.RegisterJsTapKind.RuntimePluginCreateLink,
      function () {
        return RuntimePlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).createLink;
      },
      function (queried) {
        return function (data: binding.JsCreateLinkData) {
          return queried.call(data.code, data.chunk);
        };
      },
    ),
    registerRuntimePluginLinkPreloadTaps: createTap(
      binding.RegisterJsTapKind.RuntimePluginLinkPreload,
      function () {
        return RuntimePlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).linkPreload;
      },
      function (queried) {
        return function (data: binding.JsLinkPreloadData) {
          return queried.call(data.code, data.chunk);
        };
      },
    ),
    registerRuntimePluginLinkPrefetchTaps: createTap(
      binding.RegisterJsTapKind.RuntimePluginLinkPrefetch,
      function () {
        return RuntimePlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).linkPrefetch;
      },
      function (queried) {
        return function (data: binding.JsLinkPrefetchData) {
          return queried.call(data.code, data.chunk);
        };
      },
    ),
  };
};

export { RuntimePlugin };
