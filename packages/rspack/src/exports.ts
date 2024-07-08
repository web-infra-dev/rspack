const { version: rspackVersion, webpackVersion } = require("../package.json");
export { rspackVersion, webpackVersion as version };

export type {
	Asset,
	AssetInfo,
	Assets,
	CompilationParams,
	LogEntry
} from "./Compilation";
export { Compilation } from "./Compilation";
export { Compiler } from "./Compiler";
export type { MultiCompilerOptions, MultiRspackOptions } from "./MultiCompiler";
export { MultiCompiler } from "./MultiCompiler";

import { RspackOptionsApply } from "./rspackOptionsApply";
export { RspackOptionsApply, RspackOptionsApply as WebpackOptionsApply };

export type { Chunk } from "./Chunk";
export type { ChunkGroup } from "./ChunkGroup";
export type { Module } from "./Module";
export { MultiStats } from "./MultiStats";
export { NormalModule } from "./NormalModule";
export type { NormalModuleFactory } from "./NormalModuleFactory";
export { RuntimeGlobals } from "./RuntimeGlobals";
export type {
	StatsAsset,
	StatsChunk,
	StatsCompilation,
	StatsError,
	StatsModule,
	StatsWarnings
} from "./Stats";
export { Stats } from "./Stats";

// API extractor not working with some re-exports, see: https://github.com/microsoft/fluentui/issues/20694
import * as ModuleFilenameHelpers from "./lib/ModuleFilenameHelpers";
export { ModuleFilenameHelpers };

// API extractor not working with some re-exports, see: https://github.com/microsoft/fluentui/issues/20694
import Template = require("./Template");
export { Template };

export const WebpackError = Error;

export type { Watching } from "./Watching";

import sources = require("webpack-sources");
export { sources };

import {
	applyRspackOptionsDefaults,
	getNormalizedRspackOptions
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

import { cachedCleverMerge as cleverMerge } from "./util/cleverMerge";
import { createHash } from "./util/createHash";
export const util = { createHash, cleverMerge };

export { default as EntryOptionPlugin } from "./lib/EntryOptionPlugin";
export { type OutputFileSystem } from "./util/fs";

///// Internal Plugins /////
export type { BannerPluginArgument } from "./builtin-plugin";
export type { ProvidePluginOptions } from "./builtin-plugin";
export type { DefinePluginOptions } from "./builtin-plugin";
export type { ProgressPluginArgument } from "./builtin-plugin";
export type { EntryOptions } from "./builtin-plugin";
export { BannerPlugin } from "./builtin-plugin";
export { IgnorePlugin, type IgnorePluginOptions } from "./builtin-plugin";
export { ProvidePlugin } from "./builtin-plugin";
export { DefinePlugin } from "./builtin-plugin";
export { ProgressPlugin } from "./builtin-plugin";
export { EntryPlugin } from "./builtin-plugin";
export { DynamicEntryPlugin } from "./builtin-plugin";
export { ExternalsPlugin } from "./builtin-plugin";
export { HotModuleReplacementPlugin } from "./builtin-plugin";
export { EnvironmentPlugin } from "./lib/EnvironmentPlugin";
export { LoaderOptionsPlugin } from "./lib/LoaderOptionsPlugin";
export { LoaderTargetPlugin } from "./lib/LoaderTargetPlugin";
export { NormalModuleReplacementPlugin } from "./lib/NormalModuleReplacementPlugin";

import { FetchCompileAsyncWasmPlugin } from "./builtin-plugin";
interface Web {
	FetchCompileAsyncWasmPlugin: typeof FetchCompileAsyncWasmPlugin;
}
export const web: Web = {
	FetchCompileAsyncWasmPlugin
};

import { NodeTargetPlugin } from "./builtin-plugin";
import NodeEnvironmentPlugin from "./node/NodeEnvironmentPlugin";
import NodeTemplatePlugin from "./node/NodeTemplatePlugin";
interface Node {
	NodeTargetPlugin: typeof NodeTargetPlugin;
	NodeTemplatePlugin: typeof NodeTemplatePlugin;
	NodeEnvironmentPlugin: typeof NodeEnvironmentPlugin;
}
export const node: Node = {
	NodeTargetPlugin,
	NodeTemplatePlugin,
	NodeEnvironmentPlugin
};

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

import {
	EnableChunkLoadingPlugin,
	JavascriptModulesPlugin
} from "./builtin-plugin";
interface JavaScript {
	EnableChunkLoadingPlugin: typeof EnableChunkLoadingPlugin;
	JavascriptModulesPlugin: typeof JavascriptModulesPlugin;
}
export const javascript: JavaScript = {
	EnableChunkLoadingPlugin,
	JavascriptModulesPlugin
};

import { WebWorkerTemplatePlugin } from "./builtin-plugin";
interface Webworker {
	WebWorkerTemplatePlugin: typeof WebWorkerTemplatePlugin;
}
export const webworker: Webworker = { WebWorkerTemplatePlugin };

import { LimitChunkCountPlugin } from "./builtin-plugin";
import { RuntimeChunkPlugin } from "./builtin-plugin";
import { SplitChunksPlugin } from "./builtin-plugin";
interface Optimize {
	LimitChunkCountPlugin: typeof LimitChunkCountPlugin;
	RuntimeChunkPlugin: typeof RuntimeChunkPlugin;
	SplitChunksPlugin: typeof SplitChunksPlugin;
}
export const optimize: Optimize = {
	LimitChunkCountPlugin,
	RuntimeChunkPlugin,
	SplitChunksPlugin
};

import { ModuleFederationPlugin } from "./container/ModuleFederationPlugin";
export type { ModuleFederationPluginOptions } from "./container/ModuleFederationPlugin";
import { ModuleFederationPluginV1 } from "./container/ModuleFederationPluginV1";
export type { ModuleFederationPluginV1Options } from "./container/ModuleFederationPluginV1";
import { ContainerPlugin } from "./container/ContainerPlugin";
import { ContainerReferencePlugin } from "./container/ContainerReferencePlugin";
export type {
	ContainerPluginOptions,
	Exposes,
	ExposesConfig,
	ExposesItem,
	ExposesItems,
	ExposesObject
} from "./container/ContainerPlugin";
export type {
	ContainerReferencePluginOptions,
	Remotes,
	RemotesConfig,
	RemotesItem,
	RemotesItems,
	RemotesObject
} from "./container/ContainerReferencePlugin";
export const container = {
	ContainerPlugin,
	ContainerReferencePlugin,
	ModuleFederationPlugin,
	ModuleFederationPluginV1
};

import { ConsumeSharedPlugin } from "./sharing/ConsumeSharedPlugin";
import { ProvideSharedPlugin } from "./sharing/ProvideSharedPlugin";
import { SharePlugin } from "./sharing/SharePlugin";
export type {
	Consumes,
	ConsumesConfig,
	ConsumeSharedPluginOptions,
	ConsumesItem,
	ConsumesObject
} from "./sharing/ConsumeSharedPlugin";
export type {
	Provides,
	ProvidesConfig,
	ProvideSharedPluginOptions,
	ProvidesItem,
	ProvidesObject
} from "./sharing/ProvideSharedPlugin";
export type {
	Shared,
	SharedConfig,
	SharedItem,
	SharedObject,
	SharePluginOptions
} from "./sharing/SharePlugin";
export const sharing = {
	ProvideSharedPlugin,
	ConsumeSharedPlugin,
	SharePlugin
};

///// Rspack Postfixed Internal Plugins /////
export type { HtmlRspackPluginOptions } from "./builtin-plugin";
export type { SwcJsMinimizerRspackPluginOptions } from "./builtin-plugin";
export type { LightningCssMinimizerRspackPluginOptions } from "./builtin-plugin";
export type { CopyRspackPluginOptions } from "./builtin-plugin";
export type { SourceMapDevToolPluginOptions } from "./builtin-plugin";
export type { EvalDevToolModulePluginOptions } from "./builtin-plugin";
export type {
	CssExtractRspackLoaderOptions,
	CssExtractRspackPluginOptions
} from "./builtin-plugin";
export { HtmlRspackPlugin } from "./builtin-plugin";
export { SwcJsMinimizerRspackPlugin } from "./builtin-plugin";
export { SwcCssMinimizerRspackPlugin } from "./builtin-plugin";
export { LightningCssMinimizerRspackPlugin } from "./builtin-plugin";
export { CopyRspackPlugin } from "./builtin-plugin";
export { SourceMapDevToolPlugin } from "./builtin-plugin";
export { EvalSourceMapDevToolPlugin } from "./builtin-plugin";
export { EvalDevToolModulePlugin } from "./builtin-plugin";
export { CssExtractRspackPlugin } from "./builtin-plugin";

///// Rspack Postfixed Internal Loaders /////
export type {
	SwcLoaderEnvConfig,
	SwcLoaderEsParserConfig,
	SwcLoaderJscConfig,
	SwcLoaderModuleConfig,
	SwcLoaderOptions,
	SwcLoaderParserConfig,
	SwcLoaderTransformConfig,
	SwcLoaderTsParserConfig
} from "./builtin-loader/swc/index";

///// Experiments Stuff /////
import { cleanupGlobalTrace, registerGlobalTrace } from "@rspack/binding";
interface Experiments {
	globalTrace: {
		register: typeof registerGlobalTrace;
		cleanup: typeof cleanupGlobalTrace;
	};
}
export const experiments: Experiments = {
	globalTrace: {
		register: registerGlobalTrace,
		cleanup: cleanupGlobalTrace
	}
};
