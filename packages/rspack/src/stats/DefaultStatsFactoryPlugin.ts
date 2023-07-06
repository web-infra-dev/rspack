// TODO: implement statsFactory and DefaultStatsFactoryPlugin
import * as binding from "@rspack/binding";

export type StatsChunkGroup = binding.JsStatsChunkGroup & Record<string, any>;

export type StatsAsset = binding.JsStatsAsset[] & Record<string, any>;

export type StatsChunk = binding.JsStatsChunk & Record<string, any>;

export type StatsModule = binding.JsStatsModule & Record<string, any>;

type StatsError = binding.JsStatsError & Record<string, any>;

type StatsWarnings = binding.JsStatsWarning & Record<string, any>;

export type StatsModuleReason = binding.JsStatsModuleReason &
	Record<string, any>;

export type KnownStatsCompilation = {
	version?: string;
	rspackVersion?: string;
	name?: string;
	hash?: string;
	time?: number;
	builtAt?: number;
	publicPath?: string;
	outputPath?: string;
	assets?: StatsAsset;
	assetsByChunkName?: Record<string, string[]>;
	chunks?: StatsChunk[];
	modules?: StatsModule[];
	entrypoints?: Record<string, StatsChunkGroup>;
	namedChunkGroups?: Record<string, StatsChunkGroup>;
	errors?: StatsError[];
	errorsCount?: number;
	warnings?: StatsWarnings[];
	warningsCount?: number;
	filteredModules?: number;
	children?: StatsCompilation[];

	// TODO: not aligned with webpack
	// env?: any;
	// needAdditionalPass?: boolean;
	// filteredAssets?: number;
	// logging?: Record<string, StatsLogging>;
};

export type StatsCompilation = KnownStatsCompilation & Record<string, any>;
