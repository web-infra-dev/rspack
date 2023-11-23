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

export type { MultiStats } from "./MultiStats";

export type { ChunkGroup } from "./ChunkGroup";

export type { NormalModuleFactory } from "./NormalModuleFactory";

export { NormalModule } from "./NormalModule";

export { default as ModuleFilenameHelpers } from "./lib/ModuleFilenameHelpers";

export { default as Template } from "./Template";

export const WebpackError = Error;

export type { Watching } from "./Watching";

const sources = require("webpack-sources"); // use require to avoid wrong types, @types/webpack-sources is outdate
export { sources };

import {
	getNormalizedRspackOptions,
	applyRspackOptionsDefaults
} from "./config";
export const config = {
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

///// Internal Plugins /////
export { BannerPlugin } from "./builtin-plugin";
export type { BannerPluginArgument } from "./builtin-plugin";

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

import NodeTemplatePlugin from "./node/NodeTemplatePlugin";
import { NodeTargetPlugin } from "./builtin-plugin";
export const node = { NodeTargetPlugin, NodeTemplatePlugin };

import { ElectronTargetPlugin } from "./builtin-plugin";
export const electron = { ElectronTargetPlugin };

import { EnableLibraryPlugin } from "./builtin-plugin";
export const library = { EnableLibraryPlugin };

import { EnableWasmLoadingPlugin } from "./builtin-plugin";
export const wasm = { EnableWasmLoadingPlugin };

import { EnableChunkLoadingPlugin } from "./builtin-plugin";
export const javascript = { EnableChunkLoadingPlugin };

import { WebWorkerTemplatePlugin } from "./builtin-plugin";
export const webworker = { WebWorkerTemplatePlugin };

import { LimitChunkCountPlugin } from "./builtin-plugin";
export const optimize = { LimitChunkCountPlugin };

import { ContainerPlugin } from "./container/ContainerPlugin";
import { ModuleFederationPlugin } from "./container/ModuleFederationPlugin";
export type { ModuleFederationPluginOptions } from "./container/ModuleFederationPlugin";
export type {
	ContainerPluginOptions,
	Exposes,
	ExposesItem,
	ExposesItems,
	ExposesObject,
	ExposesConfig
} from "./container/ContainerPlugin";
export const container = { ContainerPlugin, ModuleFederationPlugin };

///// Rspack Postfixed Internal Plugins /////
export { HtmlRspackPlugin } from "./builtin-plugin";
export type { HtmlRspackPluginOptions } from "./builtin-plugin";

export { SwcJsMinimizerRspackPlugin } from "./builtin-plugin";
export type { SwcJsMinimizerRspackPluginOptions } from "./builtin-plugin";

export { SwcCssMinimizerRspackPlugin } from "./builtin-plugin";

export { CopyRspackPlugin } from "./builtin-plugin";
export type { CopyRspackPluginOptions } from "./builtin-plugin";
