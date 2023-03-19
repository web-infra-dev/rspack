/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/declarations/WebpackOptions.d.ts
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import watchpack from "watchpack";
import webpackDevServer from "webpack-dev-server";
import { Compiler } from "../compiler";
import * as oldBuiltins from "./builtins";

export type { LoaderContext } from "./adapter-rule-use";

export type Configuration = RspackOptions;

export interface RspackOptions {
	name?: Name;
	dependencies?: Dependencies;
	context?: Context;
	mode?: Mode;
	entry?: Entry;
	output?: Output;
	resolve?: Resolve;
	module?: ModuleOptions;
	target?: Target;
	externals?: Externals;
	externalsType?: ExternalsType;
	externalsPresets?: ExternalsPresets;
	infrastructureLogging?: InfrastructureLogging;
	devtool?: DevTool;
	node?: Node;
	snapshot?: SnapshotOptions;
	cache?: CacheOptions;
	stats?: StatsValue;
	optimization?: Optimization;
	plugins?: Plugins;
	experiments?: Experiments;
	watch?: Watch;
	watchOptions?: WatchOptions;
	devServer?: DevServer;
	builtins?: Builtins;
}

export interface RspackOptionsNormalized {
	name?: Name;
	dependencies?: Dependencies;
	context?: Context;
	mode?: Mode;
	entry: EntryNormalized;
	output: OutputNormalized;
	resolve: Resolve;
	module: ModuleOptionsNormalized;
	target?: Target;
	externals?: Externals;
	externalsType?: ExternalsType;
	externalsPresets: ExternalsPresets;
	infrastructureLogging: InfrastructureLogging;
	devtool?: DevTool;
	node: Node;
	snapshot: SnapshotOptions;
	cache?: CacheOptions;
	stats: StatsValue;
	optimization: Optimization;
	plugins: Plugins;
	experiments: Experiments;
	watch?: Watch;
	watchOptions: WatchOptions;
	devServer?: DevServer;
	builtins: Builtins;
}

///// Name /////
export type Name = string;

///// Dependencies /////
export type Dependencies = Name[];

///// Context /////
export type Context = string;

///// Mode */////
export type Mode = "development" | "production" | "none";

///// Entry /////
export type Entry = EntryStatic;
export type EntryStatic = EntryObject | EntryUnnamed;
export type EntryUnnamed = EntryItem;
export type EntryRuntime = false | string;
export type EntryItem = string[] | string;
export interface EntryObject {
	[k: string]: EntryItem | EntryDescription;
}
export interface EntryDescription {
	import: EntryItem;
	runtime?: EntryRuntime;
}

export type EntryNormalized = EntryStaticNormalized;
export interface EntryStaticNormalized {
	[k: string]: EntryDescriptionNormalized;
}
export interface EntryDescriptionNormalized {
	import?: string[];
	runtime?: EntryRuntime;
}

///// Output /////
export interface Output {
	path?: Path;
	publicPath?: PublicPath;
	filename?: Filename;
	chunkFilename?: ChunkFilename;
	cssFilename?: CssFilename;
	cssChunkFilename?: CssChunkFilename;
	assetModuleFilename?: AssetModuleFilename;
	uniqueName?: UniqueName;
	enabledLibraryTypes?: EnabledLibraryTypes;
	libraryExport?: LibraryExport;
	libraryTarget?: LibraryType;
	auxiliaryComment?: AuxiliaryComment;
	umdNamedDefine?: UmdNamedDefine;
	module?: OutputModule;
	library?: Library;
	strictModuleErrorHandling?: StrictModuleErrorHandling;
	globalObject?: GlobalObject;
	importFunctionName?: ImportFunctionName;
}
export type Path = string;
export type PublicPath = "auto" | RawPublicPath;
export type RawPublicPath = string;
export type AssetModuleFilename = string;
export type Filename = FilenameTemplate;
export type ChunkFilename = FilenameTemplate;
export type CssFilename = FilenameTemplate;
export type CssChunkFilename = FilenameTemplate;
export type FilenameTemplate = string;
export type UniqueName = string;
export type Library = LibraryName | LibraryOptions;
export type StrictModuleErrorHandling = boolean;
export type OutputModule = boolean;
export interface LibraryCustomUmdCommentObject {
	amd?: string;
	commonjs?: string;
	commonjs2?: string;
	root?: string;
}
export interface LibraryOptions {
	auxiliaryComment?: AuxiliaryComment;
	export?: LibraryExport;
	name?: LibraryName;
	type: LibraryType;
	umdNamedDefine?: UmdNamedDefine;
}
export type LibraryName = string | string[] | LibraryCustomUmdObject;
export interface LibraryCustomUmdObject {
	amd?: string;
	commonjs?: string;
	root?: string | string[];
}
export type LibraryExport = string[] | string;
export type LibraryType =
	| (
			| "var"
			| "module"
			| "assign"
			| "assign-properties"
			| "this"
			| "window"
			| "self"
			| "global"
			| "commonjs"
			| "commonjs2"
			| "commonjs-module"
			| "commonjs-static"
			| "amd"
			| "amd-require"
			| "umd"
			| "umd2"
			| "jsonp"
			| "system"
	  )
	| string;
export type AuxiliaryComment = string | LibraryCustomUmdCommentObject;
export type UmdNamedDefine = boolean;
export type EnabledLibraryTypes = LibraryType[];
export type GlobalObject = string;
export type ImportFunctionName = string;
export interface OutputNormalized {
	path?: Path;
	publicPath?: PublicPath;
	filename?: Filename;
	chunkFilename?: ChunkFilename;
	cssFilename?: CssFilename;
	cssChunkFilename?: CssChunkFilename;
	assetModuleFilename?: AssetModuleFilename;
	uniqueName?: UniqueName;
	enabledLibraryTypes?: EnabledLibraryTypes;
	library?: LibraryOptions;
	module?: OutputModule;
	strictModuleErrorHandling?: StrictModuleErrorHandling;
	globalObject?: GlobalObject;
	importFunctionName?: ImportFunctionName;
}

///// Resolve /////
export type Resolve = ResolveOptions;
export interface ResolveOptions {
	alias?: ResolveAlias;
	/**
	 * This is `aliasField: ["browser"]` in webpack, because no one
	 * uses aliasField other than "browser". ---@bvanjoi
	 */
	browserField?: boolean;
	conditionNames?: string[];
	extensions?: string[];
	fallback?: ResolveAlias;
	mainFields?: string[];
	mainFiles?: string[];
	modules?: string[];
	preferRelative?: boolean;
	tsConfigPath?: string;
}
export type ResolveAlias = {
	[k: string]: false | string | Array<string | false>;
};

///// Module /////
export interface ModuleOptions {
	defaultRules?: RuleSetRules;
	rules?: RuleSetRules;
	parser?: ParserOptionsByModuleType;
}
export type RuleSetRules = ("..." | RuleSetRule)[];
export interface RuleSetRule {
	test?: RuleSetCondition;
	exclude?: RuleSetCondition;
	include?: RuleSetCondition;
	issuer?: RuleSetCondition;
	resource?: RuleSetCondition;
	resourceFragment?: RuleSetCondition;
	resourceQuery?: RuleSetCondition;
	oneOf?: RuleSetRule[];
	type?: string;
	use?: RuleSetUse;
	parser?: {
		[k: string]: any;
	};
	generator?: {
		[k: string]: any;
	};
	resolve?: ResolveOptions;
	sideEffects?: boolean;
}
export type RuleSetCondition =
	| RegExp
	| string
	| RuleSetConditions
	| RuleSetLogicalConditions
	| ((value: string) => boolean);
export type RuleSetConditions = RuleSetCondition[];
export interface RuleSetLogicalConditions {
	and?: RuleSetConditions;
	or?: RuleSetConditions;
	not?: RuleSetCondition;
}
export type RuleSetUse = RuleSetUseItem[] | RuleSetUseItem;
export type RuleSetUseItem = RuleSetLoaderWithOptions | RuleSetLoader;
export type RuleSetLoader = string;
export type RuleSetLoaderWithOptions = {
	// ident?: string;
	loader: RuleSetLoader;
	options?: RuleSetLoaderOptions;
};
export type RuleSetLoaderOptions =
	| string
	| {
			[k: string]: any;
	  };
export type ParserOptionsByModuleType = ParserOptionsByModuleTypeKnown;
export interface ParserOptionsByModuleTypeKnown {
	asset?: AssetParserOptions;
}
export interface AssetParserOptions {
	dataUrlCondition?: AssetParserDataUrlOptions;
}
export interface AssetParserDataUrlOptions {
	maxSize?: number;
}

export interface ModuleOptionsNormalized {
	defaultRules?: RuleSetRules;
	rules: RuleSetRules;
	parser: ParserOptionsByModuleType;
}

///// Target /////
export type Target = false | string[] | string;

///// Externals /////
export type Externals = ExternalItem[] | ExternalItem;
export type ExternalItem = string | RegExp | ExternalItemObjectUnknown;
export interface ExternalItemObjectUnknown {
	[k: string]: ExternalItemValue;
}
export type ExternalItemValue = string;

///// ExternalsType /////
export type ExternalsType =
	| "var"
	| "module"
	| "assign"
	| "this"
	| "window"
	| "self"
	| "global"
	| "commonjs"
	| "commonjs2"
	| "commonjs-module"
	| "commonjs-static"
	| "amd"
	| "amd-require"
	| "umd"
	| "umd2"
	| "jsonp"
	| "system"
	| "promise"
	| "import"
	| "script"
	| "node-commonjs";

///// ExternalsPresets /////
export interface ExternalsPresets {
	node?: boolean;
}

///// InfrastructureLogging /////
export interface InfrastructureLogging {
	appendOnly?: boolean;
	colors?: boolean;
	console?: Console;
	debug?: boolean | FilterTypes;
	level?: "none" | "error" | "warn" | "info" | "log" | "verbose";
	stream?: NodeJS.WritableStream;
}
export type FilterTypes = FilterItemTypes[] | FilterItemTypes;
export type FilterItemTypes = RegExp | string | ((value: string) => boolean);

///// DevTool /////
export type DevTool =
	| false
	| "cheap-source-map"
	| "cheap-module-source-map"
	| "source-map"
	| "inline-cheap-source-map"
	| "inline-cheap-module-source-map"
	| "inline-source-map"
	| "inline-nosources-cheap-module-source-map"
	| "inline-nosources-source-map"
	| "nosources-cheap-source-map"
	| "nosources-cheap-module-source-map"
	| "nosources-source-map"
	| "hidden-nosources-cheap-source-map"
	| "hidden-nosources-cheap-module-source-map"
	| "hidden-nosources-source-map"
	| "hidden-cheap-source-map"
	| "hidden-cheap-module-source-map"
	| "hidden-source-map"
	| "eval-cheap-source-map"
	| "eval-cheap-module-source-map"
	| "eval-source-map"
	| "eval-nosources-cheap-source-map"
	| "eval-nosources-cheap-module-source-map"
	| "eval-nosources-source-map";

///// Node /////
export type Node = NodeOptions;
// TODO: align with webpack
// | false;
export interface NodeOptions {
	__dirname?: false | true | "warn-mock" | "mock" | "eval-only";
	__filename?: false | true | "warn-mock" | "mock" | "eval-only";
	global?: boolean | "warn";
}

///// Snapshot /////
export interface SnapshotOptions {
	module?: {
		hash?: boolean;
		timestamp?: boolean;
	};
	resolve?: {
		hash?: boolean;
		timestamp?: boolean;
	};
}

///// Cache /////
// TODO: align with webpack
export type CacheOptions = true | false;

///// Stats /////
export type StatsValue =
	| ("none" | "errors-only" | "errors-warnings" | "normal" | "verbose")
	| boolean
	| StatsOptions;
export interface StatsOptions {
	all?: boolean;
	preset?: "normal" | "none" | "verbose" | "errors-only" | "errors-warnings";
	assets?: boolean;
	chunks?: boolean;
	modules?: boolean;
	entrypoints?: boolean;
	chunkGroups?: boolean;
	warnings?: boolean;
	warningsCount?: boolean;
	errors?: boolean;
	errorsCount?: boolean;
	colors?: boolean;
	hash?: boolean;
	reasons?: boolean;
	publicPath?: boolean;
	outputPath?: boolean;
	chunkModules?: boolean;
	chunkRelations?: boolean;
}

///// Optimization /////
export interface Optimization {
	moduleIds?: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | RspackPluginInstance)[];
	splitChunks?: OptimizationSplitChunksOptions | false;
	runtimeChunk?: OptimizationRuntimeChunk;
	removeAvailableModules?: boolean;
	sideEffects?: "flag" | boolean;
}
export interface OptimizationSplitChunksOptions {
	cacheGroups?: {
		[k: string]: OptimizationSplitChunksCacheGroup;
	};
	chunks?: "initial" | "async" | "all";
	maxAsyncRequests?: number;
	maxInitialRequests?: number;
	minChunks?: number;
	minSize?: OptimizationSplitChunksSizes;
	enforceSizeThreshold?: OptimizationSplitChunksSizes;
	minRemainingSize?: OptimizationSplitChunksSizes;
}
export interface OptimizationSplitChunksCacheGroup {
	chunks?: "initial" | "async" | "all";
	minChunks?: number;
	name?: string;
	priority?: number;
	reuseExistingChunk?: boolean;
	test?: RegExp;
}
export type OptimizationSplitChunksSizes = number;
export type OptimizationRuntimeChunk =
	| ("single" | "multiple")
	| boolean
	| {
			name?: string | Function;
	  };
export type OptimizationRuntimeChunkNormalized =
	| false
	| {
			name: Function;
	  };

///// Plugins /////
export type Plugins = (RspackPluginInstance | RspackPluginFunction)[];
export interface RspackPluginInstance {
	apply: (compiler: Compiler) => void;
	[k: string]: any;
}
export type RspackPluginFunction = (this: Compiler, compiler: Compiler) => void;

///// Experiments /////
export interface Experiments {
	lazyCompilation?: boolean;
	incrementalRebuild?: boolean;
}

///// Watch /////
export type Watch = boolean;

///// WatchOptions /////
export type WatchOptions = watchpack.WatchOptions;

///// DevServer /////
export interface DevServer extends webpackDevServer.Configuration {
	hot?: boolean;
}

///// Builtins /////
export type Builtins = oldBuiltins.Builtins;
