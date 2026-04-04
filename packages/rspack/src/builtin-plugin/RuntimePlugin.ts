import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';

import type { Chunk } from '../Chunk';
import { type Compilation, checkCompilation } from '../Compilation';
import {
  BindingSyncWaterfallHook,
  COMPILATION_HOOK_SUBSCRIPTION_BITSETS,
} from '../BindingHooks';
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
  getCompilationHooks: (compilation: Compilation) => RuntimePluginHooks;
};

const compilationHooksMap: WeakMap<Compilation, RuntimePluginHooks> =
  new WeakMap();

RuntimePlugin.getCompilationHooks = (compilation: Compilation) => {
  checkCompilation(compilation);

  let hooks = compilationHooksMap.get(compilation);
  if (hooks === undefined) {
    const hookSubscriptionBitset = COMPILATION_HOOK_SUBSCRIPTION_BITSETS.get(
      compilation.compiler,
    )!;
    hooks = {
      createScript: new BindingSyncWaterfallHook(
        ['code', 'chunk'],
        hookSubscriptionBitset,
        binding.CompilationHooks.RuntimePluginCreateScript,
      ),
      createLink: new BindingSyncWaterfallHook(
        ['code', 'chunk'],
        hookSubscriptionBitset,
        binding.CompilationHooks.RuntimePluginCreateLink,
      ),
      linkPreload: new BindingSyncWaterfallHook(
        ['code', 'chunk'],
        hookSubscriptionBitset,
        binding.CompilationHooks.RuntimePluginLinkPreload,
      ),
      linkPrefetch: new BindingSyncWaterfallHook(
        ['code', 'chunk'],
        hookSubscriptionBitset,
        binding.CompilationHooks.RuntimePluginLinkPrefetch,
      ),
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
      binding.CompilationHooks.RuntimePluginCreateScript,
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
      binding.CompilationHooks.RuntimePluginCreateLink,
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
      binding.CompilationHooks.RuntimePluginLinkPreload,
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
      binding.CompilationHooks.RuntimePluginLinkPrefetch,
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
