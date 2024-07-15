/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/

import type { Compilation, CreateStatsOptionsContext } from "../Compilation";
import type { Compiler } from "../Compiler";
import type { StatsError } from "../Stats";
import type { StatsOptions } from "../config";

const applyDefaults = (
	options: Partial<StatsOptions>,
	defaults: StatsOptions
) => {
	for (const key of Object.keys(defaults)) {
		if (typeof options[key as keyof StatsOptions] === "undefined") {
			options[key as keyof StatsOptions] = defaults[
				key as keyof StatsOptions
			] as any;
		}
	}
};

const NAMED_PRESETS: Record<string, StatsOptions> = {
	verbose: {
		hash: true,
		builtAt: true,
		// relatedAssets: true,
		entrypoints: true,
		chunkGroups: true,
		ids: true,
		modules: false,
		chunks: true,
		chunkRelations: true,
		chunkModules: true,
		// dependentModules: true,
		// chunkOrigins: true,
		depth: true,
		// env: true,
		reasons: true,
		usedExports: true,
		providedExports: true,
		optimizationBailout: true,
		// errorDetails: true,
		// errorStack: true,
		publicPath: true,
		logging: "verbose",
		orphanModules: true,
		runtimeModules: true,
		// exclude: false,
		// errorsSpace: Infinity,
		// warningsSpace: Infinity,
		modulesSpace: Number.POSITIVE_INFINITY,
		// chunkModulesSpace: Infinity,
		// assetsSpace: Infinity,
		// reasonsSpace: Infinity,
		children: true
	},
	detailed: {
		hash: true,
		builtAt: true,
		// relatedAssets: true,
		entrypoints: true,
		chunkGroups: true,
		ids: true,
		chunks: true,
		chunkRelations: true,
		chunkModules: false,
		// chunkOrigins: true,
		depth: true,
		usedExports: true,
		providedExports: true,
		optimizationBailout: true,
		// errorDetails: true,
		publicPath: true,
		logging: true,
		runtimeModules: true,
		// exclude: false,
		// errorsSpace: 1000,
		// warningsSpace: 1000,
		modulesSpace: 1000
		// assetsSpace: 1000,
		// reasonsSpace: 1000
	},
	minimal: {
		all: false,
		version: true,
		timings: true,
		modules: true,
		// errorsSpace: 0,
		// warningsSpace: 0,
		modulesSpace: 0,
		assets: true,
		// assetsSpace: 0,
		errors: true,
		errorsCount: true,
		warnings: true,
		warningsCount: true,
		logging: "warn"
	},
	"errors-only": {
		all: false,
		errors: true,
		errorsCount: true,
		// errorsSpace: Infinity,
		moduleTrace: true,
		logging: "error"
	},
	"errors-warnings": {
		all: false,
		errors: true,
		errorsCount: true,
		// errorsSpace: Infinity,
		warnings: true,
		warningsCount: true,
		// warningsSpace: Infinity,
		logging: "warn"
	},
	summary: {
		all: false,
		version: true,
		errorsCount: true,
		warningsCount: true
	},
	none: {
		all: false
	}
};

type StatsFunc = (
	options: StatsOptions,
	context: CreateStatsOptionsContext,
	compilation: Compilation
) => any;

type StatsDefault = {
	[K in keyof StatsOptions]: StatsFunc;
};

const NORMAL_ON: StatsFunc = ({ all }) => all !== false;
const NORMAL_OFF: StatsFunc = ({ all }) => all === true;
const ON_FOR_TO_STRING: StatsFunc = ({ all }, { forToString }) =>
	forToString ? all !== false : all === true;
const OFF_FOR_TO_STRING: StatsFunc = ({ all }, { forToString }) =>
	forToString ? all === true : all !== false;
const AUTO_FOR_TO_STRING: StatsFunc = ({ all }, { forToString }) => {
	if (all === false) return false;
	if (all === true) return true;
	if (forToString) return "auto";
	return true;
};

const DEFAULTS: StatsDefault = {
	// context: (options, context, compilation) => compilation.compiler.context,
	// requestShortener: (options, context, compilation) =>
	// 	compilation.compiler.context === options.context
	// 		? compilation.requestShortener
	// 		: new RequestShortener(options.context, compilation.compiler.root),
	performance: NORMAL_ON,
	hash: OFF_FOR_TO_STRING,
	env: NORMAL_OFF,
	version: NORMAL_ON,
	timings: NORMAL_ON,
	builtAt: OFF_FOR_TO_STRING,
	assets: NORMAL_ON,
	entrypoints: AUTO_FOR_TO_STRING,
	chunkGroups: OFF_FOR_TO_STRING,
	chunkGroupAuxiliary: OFF_FOR_TO_STRING,
	chunkGroupChildren: OFF_FOR_TO_STRING,
	chunkGroupMaxAssets: (o, { forToString }) =>
		forToString ? 5 : Number.POSITIVE_INFINITY,
	chunks: OFF_FOR_TO_STRING,
	chunkRelations: OFF_FOR_TO_STRING,
	chunkModules: ({ all, modules }) => {
		if (all === false) return false;
		if (all === true) return true;
		if (modules) return false;
		return true;
	},
	dependentModules: OFF_FOR_TO_STRING,
	chunkOrigins: OFF_FOR_TO_STRING,
	ids: OFF_FOR_TO_STRING,
	modules: ({ all, chunks, chunkModules }, { forToString }) => {
		if (all === false) return false;
		if (all === true) return true;
		if (forToString && chunks && chunkModules) return false;
		return true;
	},
	nestedModules: OFF_FOR_TO_STRING,
	groupModulesByType: ON_FOR_TO_STRING,
	groupModulesByCacheStatus: ON_FOR_TO_STRING,
	groupModulesByLayer: ON_FOR_TO_STRING,
	groupModulesByAttributes: ON_FOR_TO_STRING,
	groupModulesByPath: ON_FOR_TO_STRING,
	groupModulesByExtension: ON_FOR_TO_STRING,
	modulesSpace: (o, { forToString }) =>
		forToString ? 15 : Number.POSITIVE_INFINITY,
	chunkModulesSpace: (o, { forToString }) =>
		forToString ? 10 : Number.POSITIVE_INFINITY,
	nestedModulesSpace: (o, { forToString }) =>
		forToString ? 10 : Number.POSITIVE_INFINITY,
	relatedAssets: OFF_FOR_TO_STRING,
	groupAssetsByEmitStatus: ON_FOR_TO_STRING,
	groupAssetsByInfo: ON_FOR_TO_STRING,
	groupAssetsByPath: ON_FOR_TO_STRING,
	groupAssetsByExtension: ON_FOR_TO_STRING,
	groupAssetsByChunk: ON_FOR_TO_STRING,
	assetsSpace: (o, { forToString }) =>
		forToString ? 15 : Number.POSITIVE_INFINITY,
	orphanModules: OFF_FOR_TO_STRING,
	runtimeModules: ({ all, runtime }, { forToString }) =>
		runtime !== undefined
			? runtime
			: forToString
				? all === true
				: all !== false,
	// cachedModules: ({ all, cached }, { forToString }) =>
	// 	cached !== undefined ? cached : forToString ? all === true : all !== false,
	moduleAssets: OFF_FOR_TO_STRING,
	depth: OFF_FOR_TO_STRING,
	// cachedAssets: OFF_FOR_TO_STRING,
	reasons: OFF_FOR_TO_STRING,
	reasonsSpace: (o, { forToString }) =>
		forToString ? 15 : Number.POSITIVE_INFINITY,
	groupReasonsByOrigin: ON_FOR_TO_STRING,
	usedExports: OFF_FOR_TO_STRING,
	providedExports: OFF_FOR_TO_STRING,
	optimizationBailout: OFF_FOR_TO_STRING,
	children: OFF_FOR_TO_STRING,
	source: NORMAL_OFF,
	moduleTrace: NORMAL_ON,
	errors: NORMAL_ON,
	errorsCount: NORMAL_ON,
	errorDetails: AUTO_FOR_TO_STRING,
	errorStack: OFF_FOR_TO_STRING,
	warnings: NORMAL_ON,
	warningsCount: NORMAL_ON,
	publicPath: OFF_FOR_TO_STRING,
	logging: ({ all }, { forToString }) =>
		forToString && all !== false ? "info" : false,
	loggingDebug: () => [],
	loggingTrace: OFF_FOR_TO_STRING,
	excludeModules: () => [],
	excludeAssets: () => [],
	modulesSort: () => "depth",
	chunkModulesSort: () => "name",
	nestedModulesSort: () => false,
	chunksSort: () => false,
	assetsSort: () => "!size",
	outputPath: OFF_FOR_TO_STRING,
	colors: () => false
};

const normalizeFilter: (
	item: unknown
) => ((ident: string) => void) | undefined = item => {
	if (typeof item === "string") {
		const regExp = new RegExp(
			`[\\\\/]${item.replace(/[-[\]{}()*+?.\\^$|]/g, "\\$&")}([\\\\/]|$|!|\\?)`
		);
		return ident => regExp.test(ident);
	}
	if (
		item &&
		typeof item === "object" &&
		"test" in item &&
		typeof item.test === "function"
	) {
		const test = item.test.bind(item);
		return ident => test(ident);
	}
	if (typeof item === "function") {
		return item as (ident: string) => void;
	}
	if (typeof item === "boolean") {
		return () => item;
	}
};

const NORMALIZER = {
	excludeModules: (value: any) => {
		const array = !Array.isArray(value) ? (value ? [value] : []) : value;
		return array.map(normalizeFilter);
	},
	excludeAssets: (value: any) => {
		const array = !Array.isArray(value) ? (value ? [value] : []) : value;
		return array.map(normalizeFilter);
	},
	warningsFilter: (value: any) => {
		const array = !Array.isArray(value) ? (value ? [value] : []) : value;
		return array.map(filter => {
			if (typeof filter === "string") {
				return (warning: StatsError, warningString: string) =>
					warningString.includes(filter);
			}
			if (filter instanceof RegExp) {
				return (warning: StatsError, warningString: string) =>
					filter.test(warningString);
			}
			if (typeof filter === "function") {
				return filter;
			}
			throw new Error(
				`Can only filter warnings with Strings or RegExps. (Given: ${filter})`
			);
		});
	},
	logging: (value: any) => {
		if (value === true) value = "log";
		return value;
	},
	loggingDebug: (value: any) => {
		const array = !Array.isArray(value) ? (value ? [value] : []) : value;
		return array.map(normalizeFilter);
	}
};

export class DefaultStatsPresetPlugin {
	apply(compiler: Compiler) {
		compiler.hooks.compilation.tap("DefaultStatsPresetPlugin", compilation => {
			for (const key of Object.keys(NAMED_PRESETS)) {
				const defaults = NAMED_PRESETS[key as keyof typeof NAMED_PRESETS];
				compilation.hooks.statsPreset
					.for(key)
					.tap("DefaultStatsPresetPlugin", (options, context) => {
						applyDefaults(options, defaults);
					});
			}
			compilation.hooks.statsNormalize.tap(
				"DefaultStatsPresetPlugin",
				(options, context) => {
					for (const key of Object.keys(DEFAULTS)) {
						if (options[key as keyof StatsOptions] === undefined)
							options[key as keyof StatsOptions] = DEFAULTS[
								key as keyof typeof DEFAULTS
							]!(options, context, compilation);
					}
					for (const key of Object.keys(NORMALIZER)) {
						options[key as keyof StatsOptions] = NORMALIZER[
							key as keyof typeof NORMALIZER
						](options[key as keyof StatsOptions]);
					}
				}
			);
		});
	}
}
