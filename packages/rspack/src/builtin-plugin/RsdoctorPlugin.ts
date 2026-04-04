import {
  BuiltinPluginName,
  type JsRsdoctorAsset,
  type JsRsdoctorAssetPatch,
  type JsRsdoctorChunk,
  type JsRsdoctorChunkAssets,
  type JsRsdoctorChunkGraph,
  type JsRsdoctorChunkModules,
  type JsRsdoctorConnectionsOnlyImport,
  type JsRsdoctorConnectionsOnlyImportConnection,
  type JsRsdoctorDependency,
  type JsRsdoctorEntrypoint,
  type JsRsdoctorEntrypointAssets,
  type JsRsdoctorExportInfo,
  type JsRsdoctorModule,
  type JsRsdoctorModuleGraph,
  type JsRsdoctorModuleGraphModule,
  type JsRsdoctorModuleIdsPatch,
  type JsRsdoctorModuleOriginalSource,
  type JsRsdoctorModuleSourcesPatch,
  type JsRsdoctorSideEffect,
  type JsRsdoctorSourcePosition,
  type JsRsdoctorSourceRange,
  type JsRsdoctorStatement,
  type JsRsdoctorVariable,
  type RawRsdoctorPluginOptions,
  CompilationHooks,
} from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import { type Compilation, checkCompilation } from '../Compilation';
import type { Compiler } from '../Compiler';
import {
  BindingAsyncSeriesBailHook,
  COMPILATION_HOOK_SUBSCRIPTION_BITSETS,
} from '../BindingHooks';
import type { CreatePartialRegisters } from '../taps/types';
import { create } from './base';

// eslint-disable-next-line @typescript-eslint/no-namespace
export declare namespace RsdoctorPluginData {
  export type {
    JsRsdoctorAsset as RsdoctorAsset,
    JsRsdoctorChunkGraph as RsdoctorChunkGraph,
    JsRsdoctorModuleGraph as RsdoctorModuleGraph,
    JsRsdoctorChunk as RsdoctorChunk,
    JsRsdoctorModule as RsdoctorModule,
    JsRsdoctorSideEffect as RsdoctorSideEffect,
    JsRsdoctorExportInfo as RsdoctorExportInfo,
    JsRsdoctorVariable as RsdoctorVariable,
    JsRsdoctorConnectionsOnlyImport as RsdoctorConnectionsOnlyImport,
    JsRsdoctorConnectionsOnlyImportConnection as RsdoctorConnectionsOnlyImportConnection,
    JsRsdoctorDependency as RsdoctorDependency,
    JsRsdoctorEntrypoint as RsdoctorEntrypoint,
    JsRsdoctorStatement as RsdoctorStatement,
    JsRsdoctorSourceRange as RsdoctorSourceRange,
    JsRsdoctorSourcePosition as RsdoctorSourcePosition,
    JsRsdoctorModuleGraphModule as RsdoctorModuleGraphModule,
    JsRsdoctorModuleIdsPatch as RsdoctorModuleIdsPatch,
    JsRsdoctorModuleOriginalSource as RsdoctorModuleOriginalSource,
    JsRsdoctorAssetPatch as RsdoctorAssetPatch,
    JsRsdoctorChunkAssets as RsdoctorChunkAssets,
    JsRsdoctorEntrypointAssets as RsdoctorEntrypointAssets,
    JsRsdoctorChunkModules as RsdoctorChunkModules,
    JsRsdoctorModuleSourcesPatch as RsdoctorModuleSourcesPatch,
  };
}

export type RsdoctorPluginOptions = {
  moduleGraphFeatures?: boolean | ('graph' | 'ids' | 'sources')[];
  chunkGraphFeatures?: boolean | ('graph' | 'assets')[];
  sourceMapFeatures?: {
    module?: boolean;
    cheap?: boolean;
  };
};

const RsdoctorPluginImpl = create(
  BuiltinPluginName.RsdoctorPlugin,
  function (
    this: Compiler,
    c: RsdoctorPluginOptions = {
      moduleGraphFeatures: true,
      chunkGraphFeatures: true,
    },
  ): RawRsdoctorPluginOptions {
    return {
      moduleGraphFeatures: c.moduleGraphFeatures ?? true,
      chunkGraphFeatures: c.chunkGraphFeatures ?? true,
      sourceMapFeatures: c.sourceMapFeatures,
    };
  },
);

export type RsdoctorPluginHooks = {
  moduleGraph: liteTapable.AsyncSeriesBailHook<
    [JsRsdoctorModuleGraph],
    false | void
  >;
  chunkGraph: liteTapable.AsyncSeriesBailHook<
    [JsRsdoctorChunkGraph],
    false | void
  >;
  moduleIds: liteTapable.AsyncSeriesBailHook<
    [JsRsdoctorModuleIdsPatch],
    false | void
  >;
  moduleSources: liteTapable.AsyncSeriesBailHook<
    [JsRsdoctorModuleSourcesPatch],
    false | void
  >;
  assets: liteTapable.AsyncSeriesBailHook<[JsRsdoctorAssetPatch], false | void>;
};

const compilationHooksMap: WeakMap<Compilation, RsdoctorPluginHooks> =
  new WeakMap();

const RsdoctorPlugin = RsdoctorPluginImpl as typeof RsdoctorPluginImpl & {
  getCompilationHooks: (compilation: Compilation) => RsdoctorPluginHooks;
};

RsdoctorPlugin.getCompilationHooks = (compilation: Compilation) => {
  checkCompilation(compilation);

  const existingHooks = compilationHooksMap.get(compilation);
  if (existingHooks) {
    return existingHooks;
  }

  const hookSubscriptionBitset = COMPILATION_HOOK_SUBSCRIPTION_BITSETS.get(
    compilation.compiler,
  )!;
  const hooks: RsdoctorPluginHooks = {
    moduleGraph: new BindingAsyncSeriesBailHook<
      [JsRsdoctorModuleGraph],
      false | void
    >(
      ['moduleGraph'],
      hookSubscriptionBitset,
      CompilationHooks.RsdoctorPluginModuleGraph,
    ),
    chunkGraph: new BindingAsyncSeriesBailHook<
      [JsRsdoctorChunkGraph],
      false | void
    >(
      ['chunkGraph'],
      hookSubscriptionBitset,
      CompilationHooks.RsdoctorPluginChunkGraph,
    ),
    moduleIds: new BindingAsyncSeriesBailHook<
      [JsRsdoctorModuleIdsPatch],
      false | void
    >(
      ['moduleIdsPatch'],
      hookSubscriptionBitset,
      CompilationHooks.RsdoctorPluginModuleIds,
    ),
    moduleSources: new BindingAsyncSeriesBailHook<
      [JsRsdoctorModuleSourcesPatch],
      false | void
    >(
      ['moduleSourcesPatch'],
      hookSubscriptionBitset,
      CompilationHooks.RsdoctorPluginModuleSources,
    ),
    assets: new BindingAsyncSeriesBailHook<
      [JsRsdoctorAssetPatch],
      false | void
    >(
      ['assetPatch'],
      hookSubscriptionBitset,
      CompilationHooks.RsdoctorPluginAssets,
    ),
  };
  compilationHooksMap.set(compilation, hooks);
  return hooks;
};

export const createRsdoctorPluginHooksRegisters: CreatePartialRegisters<
  `RsdoctorPlugin`
> = (getCompiler, createTap) => {
  return {
    registerRsdoctorPluginModuleGraphTaps: createTap(
      CompilationHooks.RsdoctorPluginModuleGraph,
      function () {
        return RsdoctorPlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).moduleGraph;
      },
      function (queried) {
        return async function (data: JsRsdoctorModuleGraph) {
          return queried.promise(data);
        };
      },
    ),
    registerRsdoctorPluginChunkGraphTaps: createTap(
      CompilationHooks.RsdoctorPluginChunkGraph,
      function () {
        return RsdoctorPlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).chunkGraph;
      },
      function (queried) {
        return async function (data: JsRsdoctorChunkGraph) {
          return queried.promise(data);
        };
      },
    ),
    registerRsdoctorPluginModuleIdsTaps: createTap(
      CompilationHooks.RsdoctorPluginModuleIds,
      function () {
        return RsdoctorPlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).moduleIds;
      },
      function (queried) {
        return async function (data: JsRsdoctorModuleIdsPatch) {
          return queried.promise(data);
        };
      },
    ),
    registerRsdoctorPluginModuleSourcesTaps: createTap(
      CompilationHooks.RsdoctorPluginModuleSources,
      function () {
        return RsdoctorPlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).moduleSources;
      },
      function (queried) {
        return async function (data: JsRsdoctorModuleSourcesPatch) {
          return queried.promise(data);
        };
      },
    ),
    registerRsdoctorPluginAssetsTaps: createTap(
      CompilationHooks.RsdoctorPluginAssets,
      function () {
        return RsdoctorPlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).assets;
      },
      function (queried) {
        return async function (data: JsRsdoctorAssetPatch) {
          return queried.promise(data);
        };
      },
    ),
  };
};

export { RsdoctorPlugin };
