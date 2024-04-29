const { version: rspackVersion, webpackVersion } = require("../package.json");
export { webpackVersion as version, rspackVersion };

export { Compiler } from "./Compiler";

export { Compilation } from "./Compilation";
export type {
	Asset,
	AssetInfo,
	Assets,
	LogEntry,
	CompilationParams
} from "./Compilation";

export { MultiCompiler } from "./MultiCompiler";
export type { MultiCompilerOptions, MultiRspackOptions } from "./MultiCompiler";

import { RspackOptionsApply } from "./rspackOptionsApply";
export { RspackOptionsApply, RspackOptionsApply as WebpackOptionsApply };

export { RuntimeGlobals } from "./RuntimeGlobals";

export { Stats } from "./Stats";
export type {
	StatsCompilation,
	StatsAsset,
	StatsChunk,
	StatsError,
	StatsModule,
	StatsWarnings
} from "./Stats";

export { MultiStats } from "./MultiStats";

export type { ChunkGroup } from "./ChunkGroup";

export type { NormalModuleFactory } from "./NormalModuleFactory";

export { NormalModule } from "./NormalModule";

// API extractor not working with some re-exports, see: https://github.com/microsoft/fluentui/issues/20694
import * as ModuleFilenameHelpers from "./lib/ModuleFilenameHelpers";
export { ModuleFilenameHelpers };

// API extractor not working with some re-exports, see: https://github.com/microsoft/fluentui/issues/20694
import Template = require("./Template");
export { Template };

export const WebpackError = Error;

export type { Watching } from "./Watching";

const sources = require("webpack-sources"); // use require to avoid wrong types, @types/webpack-sources is outdate
export { sources };

import {
	getNormalizedRspackOptions,
	applyRspackOptionsDefaults
} from "./config";

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
	applyWebpackOptionsDefaults: applyRspackOptionsDefaults
};

export type * from "./config";

import { createHash } from "./util/createHash";
import { cachedCleverMerge as cleverMerge } from "./util/cleverMerge";
export const util = { createHash, cleverMerge };

export {
	registerGlobalTrace as experimental_registerGlobalTrace,
	cleanupGlobalTrace as experimental_cleanupGlobalTrace
} from "@rspack/binding";

export { default as EntryOptionPlugin } from "./lib/EntryOptionPlugin";

///// Internal Plugins /////
export { BannerPlugin } from "./builtin-plugin";
export type { BannerPluginArgument } from "./builtin-plugin";

export { IgnorePlugin, type IgnorePluginOptions } from "./builtin-plugin";

export { ProvidePlugin } from "./builtin-plugin";
export type { ProvidePluginOptions } from "./builtin-plugin";

export { DefinePlugin } from "./builtin-plugin";
export type { DefinePluginOptions } from "./builtin-plugin";

export { ProgressPlugin } from "./builtin-plugin";
export type { ProgressPluginArgument } from "./builtin-plugin";

export { EntryPlugin } from "./builtin-plugin";
export type { EntryOptions } from "./builtin-plugin";

export { ExternalsPlugin } from "./builtin-plugin";

export { HotModuleReplacementPlugin } from "./builtin-plugin";

export { LoaderOptionsPlugin } from "./lib/LoaderOptionsPlugin";

export { LoaderTargetPlugin } from "./lib/LoaderTargetPlugin";

export { EnvironmentPlugin } from "./lib/EnvironmentPlugin";

export { NormalModuleReplacementPlugin } from "./lib/NormalModuleReplacementPlugin";

import NodeTemplatePlugin from "./node/NodeTemplatePlugin";
import { NodeTargetPlugin } from "./builtin-plugin";
interface Node {
	NodeTargetPlugin: typeof NodeTargetPlugin;
	NodeTemplatePlugin: typeof NodeTemplatePlugin;
}
export const node: Node = { NodeTargetPlugin, NodeTemplatePlugin };

import { ElectronTargetPlugin } from "./builtin-plugin";
interface Electron {
	ElectronTargetPlugin: typeof ElectronTargetPlugin;
}
export const electron: Electron = { ElectronTargetPlugin };

import { EnableLibraryPlugin } from "./builtin-plugin";
interface Library {
	EnableLibraryPlugin: typeof EnableLibraryPlugin;
}
export const library: Library = { EnableLibraryPlugin };

import { EnableWasmLoadingPlugin } from "./builtin-plugin";
interface Wasm {
	EnableWasmLoadingPlugin: typeof EnableWasmLoadingPlugin;
}
export const wasm: Wasm = { EnableWasmLoadingPlugin };

import { EnableChunkLoadingPlugin } from "./builtin-plugin";
interface JavaScript {
	EnableChunkLoadingPlugin: typeof EnableChunkLoadingPlugin;
}
export const javascript: JavaScript = { EnableChunkLoadingPlugin };

import { WebWorkerTemplatePlugin } from "./builtin-plugin";
interface Webworker {
	WebWorkerTemplatePlugin: typeof WebWorkerTemplatePlugin;
}
export const webworker: Webworker = { WebWorkerTemplatePlugin };

import { LimitChunkCountPlugin } from "./builtin-plugin";
import { RuntimeChunkPlugin } from "./builtin-plugin";
interface Optimize {
	LimitChunkCountPlugin: typeof LimitChunkCountPlugin;
	RuntimeChunkPlugin: typeof RuntimeChunkPlugin;
}
export const optimize: Optimize = { LimitChunkCountPlugin, RuntimeChunkPlugin };

import { ModuleFederationPlugin } from "./container/ModuleFederationPlugin";
export type { ModuleFederationPluginOptions } from "./container/ModuleFederationPlugin";
import { ModuleFederationPluginV1 } from "./container/ModuleFederationPluginV1";
export type { ModuleFederationPluginV1Options } from "./container/ModuleFederationPluginV1";
import { ContainerPlugin } from "./container/ContainerPlugin";
import { ContainerReferencePlugin } from "./container/ContainerReferencePlugin";
export type {
	ContainerPluginOptions,
	Exposes,
	ExposesItem,
	ExposesItems,
	ExposesObject,
	ExposesConfig
} from "./container/ContainerPlugin";
export type {
	ContainerReferencePluginOptions,
	Remotes,
	RemotesItem,
	RemotesItems,
	RemotesObject,
	RemotesConfig
} from "./container/ContainerReferencePlugin";
export const container = {
	ContainerPlugin,
	ContainerReferencePlugin,
	ModuleFederationPlugin,
	ModuleFederationPluginV1
};

import { ProvideSharedPlugin } from "./sharing/ProvideSharedPlugin";
import { ConsumeSharedPlugin } from "./sharing/ConsumeSharedPlugin";
import { SharePlugin } from "./sharing/SharePlugin";
export type {
	ProvideSharedPluginOptions,
	Provides,
	ProvidesConfig,
	ProvidesItem,
	ProvidesObject
} from "./sharing/ProvideSharedPlugin";
export type {
	ConsumeSharedPluginOptions,
	Consumes,
	ConsumesConfig,
	ConsumesItem,
	ConsumesObject
} from "./sharing/ConsumeSharedPlugin";
export type {
	SharePluginOptions,
	Shared,
	SharedConfig,
	SharedItem,
	SharedObject
} from "./sharing/SharePlugin";
export const sharing = {
	ProvideSharedPlugin,
	ConsumeSharedPlugin,
	SharePlugin
};

///// Rspack Postfixed Internal Plugins /////
export { HtmlRspackPlugin } from "./builtin-plugin";
export type { HtmlRspackPluginOptions } from "./builtin-plugin";

export { SwcJsMinimizerRspackPlugin } from "./builtin-plugin";
export type { SwcJsMinimizerRspackPluginOptions } from "./builtin-plugin";

export { SwcCssMinimizerRspackPlugin } from "./builtin-plugin";

export { CopyRspackPlugin } from "./builtin-plugin";
export type { CopyRspackPluginOptions } from "./builtin-plugin";

export { SourceMapDevToolPlugin } from "./builtin-plugin";
export { EvalSourceMapDevToolPlugin } from "./builtin-plugin";
export type { SourceMapDevToolPluginOptions } from "./builtin-plugin";

export { EvalDevToolModulePlugin } from "./builtin-plugin";
export type { EvalDevToolModulePluginOptions } from "./builtin-plugin";

export { CssExtractRspackPlugin } from "./builtin-plugin";

///// Rspack Postfixed Internal Loaders /////
export type {
	SwcLoaderOptions,
	SwcLoaderEnvConfig,
	SwcLoaderJscConfig,
	SwcLoaderModuleConfig,
	SwcLoaderParserConfig,
	SwcLoaderEsParserConfig,
	SwcLoaderTsParserConfig,
	SwcLoaderTransformConfig
} from "./builtin-loader/swc/index";
