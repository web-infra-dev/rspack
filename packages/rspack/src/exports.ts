// this is a hack to be compatible with plugin which detect webpack's version
const rspackVersion = RSPACK_VERSION;
const version = WEBPACK_VERSION;

export { rspackVersion, version };

export type {
  Asset,
  AssetInfo,
  Assets,
  ChunkPathData,
  CompilationParams,
  LogEntry,
  PathData,
} from './Compilation';
export { Compilation } from './Compilation';
export { Compiler, type CompilerHooks } from './Compiler';
export type { MultiCompilerOptions, MultiRspackOptions } from './MultiCompiler';
export { MultiCompiler } from './MultiCompiler';

import { RspackOptionsApply } from './rspackOptionsApply';
export { RspackOptionsApply, RspackOptionsApply as WebpackOptionsApply };

export type { ChunkGroup } from '@rspack/binding';
export {
  AsyncDependenciesBlock,
  Dependency,
  EntryDependency,
} from '@rspack/binding';
export type { Chunk } from './Chunk';
export { ConcatenatedModule } from './ConcatenatedModule';
export { ContextModule } from './ContextModule';
export { ExternalModule } from './ExternalModule';
export type { ResolveData, ResourceDataWithData } from './Module';
export { Module } from './Module';
export type { default as ModuleGraph } from './ModuleGraph';
export { MultiStats } from './MultiStats';
export { NormalModule } from './NormalModule';
export type { NormalModuleFactory } from './NormalModuleFactory';
export {
  type RspackError,
  type RspackSeverity,
  ValidationError,
} from './RspackError';
export { RuntimeGlobals } from './RuntimeGlobals';
export { RuntimeModule } from './RuntimeModule';
export type {
  StatsAsset,
  StatsChunk,
  StatsCompilation,
  StatsError,
  StatsModule,
} from './Stats';
export { Stats } from './Stats';
export { StatsErrorCode } from './stats/statsFactoryUtils';

// API extractor not working with some re-exports, see: https://github.com/microsoft/fluentui/issues/20694
import * as ModuleFilenameHelpers from './lib/ModuleFilenameHelpers';
export { ModuleFilenameHelpers };

// API extractor not working with some re-exports, see: https://github.com/microsoft/fluentui/issues/20694
export { Template } from './Template';

export const WebpackError = Error;

export type { Watching } from './Watching';

import sources = require('webpack-sources');

export { sources };

import {
  applyRspackOptionsDefaults,
  getNormalizedRspackOptions,
} from './config';

// Explicitly define this type to avoid type inference and type expansion.
type Config = {
  getNormalizedRspackOptions: typeof getNormalizedRspackOptions;
  applyRspackOptionsDefaults: typeof applyRspackOptionsDefaults;
  getNormalizedWebpackOptions: typeof getNormalizedRspackOptions;
  applyWebpackOptionsDefaults: typeof applyRspackOptionsDefaults;
};
export const config: Config = {
  getNormalizedRspackOptions,
  applyRspackOptionsDefaults,
  getNormalizedWebpackOptions: getNormalizedRspackOptions,
  applyWebpackOptionsDefaults: applyRspackOptionsDefaults,
};

export type * from './config';

import { cachedCleverMerge as cleverMerge } from './util/cleverMerge';
import { createHash } from './util/createHash';

export const util = { createHash, cleverMerge };

///// Internal Plugins /////
export type {
  BannerPluginArgument,
  DefinePluginOptions,
  EntryOptions,
  ProgressPluginArgument,
  ProvidePluginOptions,
} from './builtin-plugin';
export {
  BannerPlugin,
  CaseSensitivePlugin,
  /**
   * @deprecated Use `rspack.CaseSensitivePlugin` instead
   */
  CaseSensitivePlugin as WarnCaseSensitiveModulesPlugin,
  DefinePlugin,
  DynamicEntryPlugin,
  EntryPlugin,
  ExternalsPlugin,
  HotModuleReplacementPlugin,
  IgnorePlugin,
  type IgnorePluginOptions,
  NoEmitOnErrorsPlugin,
  ProgressPlugin,
  ProvidePlugin,
  RuntimePlugin,
} from './builtin-plugin';
export { DllPlugin, type DllPluginOptions } from './lib/DllPlugin';
export {
  DllReferencePlugin,
  type DllReferencePluginOptions,
  type DllReferencePluginOptionsContent,
  type DllReferencePluginOptionsManifest,
  type DllReferencePluginOptionsSourceType,
} from './lib/DllReferencePlugin';
export { default as EntryOptionPlugin } from './lib/EntryOptionPlugin';
export { EnvironmentPlugin } from './lib/EnvironmentPlugin';
export { LoaderOptionsPlugin } from './lib/LoaderOptionsPlugin';
export { LoaderTargetPlugin } from './lib/LoaderTargetPlugin';
export type { OutputFileSystem, WatchFileSystem } from './util/fs';

import {
  EsmLibraryPlugin,
  FetchCompileAsyncWasmPlugin,
  lazyCompilationMiddleware,
  SubresourceIntegrityPlugin,
} from './builtin-plugin';

export { SubresourceIntegrityPlugin };

interface Web {
  FetchCompileAsyncWasmPlugin: typeof FetchCompileAsyncWasmPlugin;
}

export const web: Web = {
  FetchCompileAsyncWasmPlugin,
};

import { NodeTargetPlugin } from './builtin-plugin';
import NodeEnvironmentPlugin from './node/NodeEnvironmentPlugin';
import NodeTemplatePlugin from './node/NodeTemplatePlugin';

interface Node {
  NodeTargetPlugin: typeof NodeTargetPlugin;
  NodeTemplatePlugin: typeof NodeTemplatePlugin;
  NodeEnvironmentPlugin: typeof NodeEnvironmentPlugin;
}

export { lazyCompilationMiddleware };

export const node: Node = {
  NodeTargetPlugin,
  NodeTemplatePlugin,
  NodeEnvironmentPlugin,
};

import { ElectronTargetPlugin } from './builtin-plugin';

interface Electron {
  ElectronTargetPlugin: typeof ElectronTargetPlugin;
}

export const electron: Electron = { ElectronTargetPlugin };

import { EnableLibraryPlugin } from './builtin-plugin';

interface Library {
  EnableLibraryPlugin: typeof EnableLibraryPlugin;
}

export const library: Library = { EnableLibraryPlugin };

import { EnableWasmLoadingPlugin } from './builtin-plugin';

interface Wasm {
  EnableWasmLoadingPlugin: typeof EnableWasmLoadingPlugin;
}

export const wasm: Wasm = { EnableWasmLoadingPlugin };

import {
  EnableChunkLoadingPlugin,
  JavascriptModulesPlugin,
} from './builtin-plugin';

interface JavaScript {
  EnableChunkLoadingPlugin: typeof EnableChunkLoadingPlugin;
  JavascriptModulesPlugin: typeof JavascriptModulesPlugin;
}

export const javascript: JavaScript = {
  EnableChunkLoadingPlugin,
  JavascriptModulesPlugin,
};

import { WebWorkerTemplatePlugin } from './builtin-plugin';

interface Webworker {
  WebWorkerTemplatePlugin: typeof WebWorkerTemplatePlugin;
}

export const webworker: Webworker = { WebWorkerTemplatePlugin };

import {
  CssChunkingPlugin,
  LimitChunkCountPlugin,
  RemoveDuplicateModulesPlugin,
  RsdoctorPlugin,
  RslibPlugin,
  RstestPlugin,
  RuntimeChunkPlugin,
  SplitChunksPlugin,
} from './builtin-plugin';

interface Optimize {
  LimitChunkCountPlugin: typeof LimitChunkCountPlugin;
  RuntimeChunkPlugin: typeof RuntimeChunkPlugin;
  SplitChunksPlugin: typeof SplitChunksPlugin;
}

export const optimize: Optimize = {
  LimitChunkCountPlugin,
  RuntimeChunkPlugin,
  SplitChunksPlugin,
};

import { ModuleFederationPlugin } from './container/ModuleFederationPlugin';

export type { ModuleFederationPluginOptions } from './container/ModuleFederationPlugin';

import { ModuleFederationPluginV1 } from './container/ModuleFederationPluginV1';

export type { ModuleFederationPluginV1Options } from './container/ModuleFederationPluginV1';

import { ContainerPlugin } from './container/ContainerPlugin';
import { ContainerReferencePlugin } from './container/ContainerReferencePlugin';

export type {
  ContainerPluginOptions,
  Exposes,
  ExposesConfig,
  ExposesItem,
  ExposesItems,
  ExposesObject,
} from './container/ContainerPlugin';
export type {
  ContainerReferencePluginOptions,
  Remotes,
  RemotesConfig,
  RemotesItem,
  RemotesItems,
  RemotesObject,
} from './container/ContainerReferencePlugin';
export const container = {
  ContainerPlugin,
  ContainerReferencePlugin,
  ModuleFederationPlugin,
  ModuleFederationPluginV1,
};

import { ConsumeSharedPlugin } from './sharing/ConsumeSharedPlugin';
import { ProvideSharedPlugin } from './sharing/ProvideSharedPlugin';
import { SharePlugin } from './sharing/SharePlugin';
import { TreeShakeSharedPlugin } from './sharing/TreeShakeSharedPlugin';

export type {
  ConsumeSharedPluginOptions,
  Consumes,
  ConsumesConfig,
  ConsumesItem,
  ConsumesObject,
} from './sharing/ConsumeSharedPlugin';
export type {
  ProvideSharedPluginOptions,
  Provides,
  ProvidesConfig,
  ProvidesItem,
  ProvidesObject,
} from './sharing/ProvideSharedPlugin';
export type {
  Shared,
  SharedConfig,
  SharedItem,
  SharedObject,
  SharePluginOptions,
} from './sharing/SharePlugin';
export type { TreeshakeSharedPluginOptions } from './sharing/TreeShakeSharedPlugin';
export const sharing = {
  ProvideSharedPlugin,
  TreeShakeSharedPlugin,
  ConsumeSharedPlugin,
  SharePlugin,
};

export type {
  FeatureOptions as LightningcssFeatureOptions,
  LoaderOptions as LightningcssLoaderOptions,
} from './builtin-loader/lightningcss/index';
///// Rspack Postfixed Internal Loaders /////
export type {
  SwcLoaderEnvConfig,
  SwcLoaderEsParserConfig,
  SwcLoaderJscConfig,
  SwcLoaderModuleConfig,
  SwcLoaderOptions,
  SwcLoaderParserConfig,
  SwcLoaderTransformConfig,
  SwcLoaderTsParserConfig,
} from './builtin-loader/swc/index';
///// Rspack Postfixed Internal Plugins /////
export type {
  CircularDependencyRspackPluginOptions,
  CopyRspackPluginOptions,
  CssExtractRspackLoaderOptions,
  CssExtractRspackPluginOptions,
  EvalDevToolModulePluginOptions,
  HtmlRspackPluginOptions,
  LightningCssMinimizerRspackPluginOptions,
  RsdoctorPluginData,
  RsdoctorPluginHooks,
  SourceMapDevToolPluginOptions,
  SubresourceIntegrityPluginOptions,
  SwcJsMinimizerRspackPluginOptions,
} from './builtin-plugin';
export {
  CircularDependencyRspackPlugin,
  ContextReplacementPlugin,
  CopyRspackPlugin,
  CssExtractRspackPlugin,
  EvalDevToolModulePlugin,
  EvalSourceMapDevToolPlugin,
  HtmlRspackPlugin,
  LightningCssMinimizerRspackPlugin,
  NormalModuleReplacementPlugin,
  SourceMapDevToolPlugin,
  SwcJsMinimizerRspackPlugin,
} from './builtin-plugin';

///// Experiments Stuff /////
///// Experiments rspack_resolver
import {
  cleanupGlobalTrace,
  EnforceExtension,
  ResolverFactory,
  registerGlobalTrace,
  async as resolveAsync,
  sync as resolveSync,
  syncTraceEvent,
} from '@rspack/binding';
import { createNativePlugin } from './builtin-plugin';
///// Experiments SWC /////
import { minify, minifySync, transform, transformSync } from './swc';
import { JavaScriptTracer } from './trace';
import { VirtualModulesPlugin } from './VirtualModulesPlugin';

interface Experiments {
  globalTrace: {
    register: (
      filter: string,
      layer: 'logger' | 'perfetto',
      output: string,
    ) => Promise<void>;
    cleanup: () => Promise<void>;
  };
  RemoveDuplicateModulesPlugin: typeof RemoveDuplicateModulesPlugin;
  /**
   * @deprecated Use `rspack.SubresourceIntegrityPlugin` instead
   */
  SubresourceIntegrityPlugin: typeof SubresourceIntegrityPlugin;
  EsmLibraryPlugin: typeof EsmLibraryPlugin;
  RsdoctorPlugin: typeof RsdoctorPlugin;
  RstestPlugin: typeof RstestPlugin;
  RslibPlugin: typeof RslibPlugin;
  /**
   * @deprecated Use `rspack.lazyCompilationMiddleware` instead
   */
  lazyCompilationMiddleware: typeof lazyCompilationMiddleware;
  swc: {
    transform: typeof transform;
    minify: typeof minify;
    transformSync: typeof transformSync;
    minifySync: typeof minifySync;
  };
  resolver: {
    ResolverFactory: typeof ResolverFactory;
    EnforceExtension: typeof EnforceExtension;
    async: typeof resolveAsync;
    sync: typeof resolveSync;
  };
  CssChunkingPlugin: typeof CssChunkingPlugin;
  createNativePlugin: typeof createNativePlugin;
  VirtualModulesPlugin: typeof VirtualModulesPlugin;
}

export const experiments: Experiments = {
  globalTrace: {
    async register(filter, layer, output) {
      await JavaScriptTracer.initJavaScriptTrace(layer, output);
      registerGlobalTrace(filter, layer, output);
      // lazy init cpuProfiler to make sure js and rust's timestamp is much aligned
      JavaScriptTracer.initCpuProfiler();
    },
    async cleanup() {
      await JavaScriptTracer.cleanupJavaScriptTrace();
      syncTraceEvent(JavaScriptTracer.events);
      cleanupGlobalTrace();
    },
  },
  RemoveDuplicateModulesPlugin,
  SubresourceIntegrityPlugin,
  EsmLibraryPlugin,
  /**
   * Note: This plugin is unstable yet
   *
   * @internal
   */
  RsdoctorPlugin,
  /**
   * Note: This plugin is unstable yet
   *
   * @internal
   */
  RstestPlugin,
  /**
   * Note: This plugin is unstable yet
   *
   * @internal
   */
  RslibPlugin,
  lazyCompilationMiddleware,
  swc: {
    minify,
    transform,
    minifySync,
    transformSync,
  },
  resolver: {
    ResolverFactory,
    EnforceExtension,
    async: resolveAsync,
    sync: resolveSync,
  },
  CssChunkingPlugin,
  createNativePlugin,
  VirtualModulesPlugin,
};
