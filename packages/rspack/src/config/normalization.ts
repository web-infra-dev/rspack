/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/config/normalization.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type {
	EntryStatic,
	EntryStaticNormalized,
	LibraryOptions,
	OptimizationRuntimeChunk,
	OptimizationRuntimeChunkNormalized,
	RspackOptions,
	RspackOptionsNormalized
} from "./types";

export const getNormalizedRspackOptions = (
	config: RspackOptions
): RspackOptionsNormalized => {
	return {
		ignoreWarnings:
			config.ignoreWarnings !== undefined
				? config.ignoreWarnings.map(ignore => {
						if (typeof ignore === "function") {
							return ignore;
						} else {
							return warning => {
								return ignore.test(warning.message);
							};
						}
				  })
				: undefined,
		name: config.name,
		dependencies: config.dependencies,
		context: config.context,
		mode: config.mode,
		entry:
			config.entry === undefined
				? { main: {} }
				: getNormalizedEntryStatic(config.entry),
		output: nestedConfig(config.output, output => {
			const { library } = output;
			const libraryAsName = library;
			const libraryBase =
				typeof library === "object" &&
				library &&
				!Array.isArray(library) &&
				"type" in library
					? library
					: libraryAsName || output.libraryTarget
					? ({
							name: libraryAsName
					  } as LibraryOptions)
					: undefined;

			return {
				path: output.path,
				publicPath: output.publicPath,
				filename: output.filename,
				clean: output.clean,
				chunkFormat: output.chunkFormat,
				chunkLoading: output.chunkLoading,
				chunkFilename: output.chunkFilename,
				crossOriginLoading: output.crossOriginLoading,
				cssFilename: output.cssFilename,
				cssChunkFilename: output.cssChunkFilename,
				hotUpdateMainFilename: output.hotUpdateMainFilename,
				hotUpdateChunkFilename: output.hotUpdateChunkFilename,
				assetModuleFilename: output.assetModuleFilename,
				wasmLoading: output.wasmLoading,
				enabledChunkLoadingTypes: output.enabledChunkLoadingTypes
					? [...output.enabledChunkLoadingTypes]
					: ["..."],
				enabledWasmLoadingTypes: output.enabledWasmLoadingTypes
					? [...output.enabledWasmLoadingTypes]
					: ["..."],
				webassemblyModuleFilename: output.webassemblyModuleFilename,
				uniqueName: output.uniqueName,
				chunkLoadingGlobal: output.chunkLoadingGlobal,
				enabledLibraryTypes: output.enabledLibraryTypes
					? [...output.enabledLibraryTypes]
					: ["..."],
				globalObject: output.globalObject,
				importFunctionName: output.importFunctionName,
				iife: output.iife,
				module: output.module,
				sourceMapFilename: output.sourceMapFilename,
				library: libraryBase && {
					type:
						output.libraryTarget !== undefined
							? output.libraryTarget
							: libraryBase.type,
					auxiliaryComment:
						output.auxiliaryComment !== undefined
							? output.auxiliaryComment
							: libraryBase.auxiliaryComment,
					export:
						output.libraryExport !== undefined
							? output.libraryExport
							: libraryBase.export,
					name: libraryBase.name,
					umdNamedDefine:
						output.umdNamedDefine !== undefined
							? output.umdNamedDefine
							: libraryBase.umdNamedDefine
				},
				trustedTypes: optionalNestedConfig(
					output.trustedTypes,
					trustedTypes => {
						if (trustedTypes === true) return {};
						if (typeof trustedTypes === "string")
							return { policyName: trustedTypes };
						return { ...trustedTypes };
					}
				)
			};
		}),
		resolve: nestedConfig(config.resolve, resolve => ({
			...resolve
		})),
		module: nestedConfig(config.module, module => ({
			parser: keyedNestedConfig(
				module.parser as Record<string, any>,
				cloneObject,
				{}
			),
			defaultRules: optionalNestedArray(module.defaultRules, r => [...r]),
			rules: nestedArray(module.rules, r => [...r])
		})),
		target: config.target,
		externals: config.externals,
		externalsType: config.externalsType,
		externalsPresets: cloneObject(config.externalsPresets),
		infrastructureLogging: cloneObject(config.infrastructureLogging),
		devtool: config.devtool,
		node: nestedConfig(
			config.node,
			node =>
				node && {
					...node
				}
		),
		snapshot: nestedConfig(config.snapshot, snapshot => ({
			resolve: optionalNestedConfig(snapshot.resolve, resolve => ({
				timestamp: resolve.timestamp,
				hash: resolve.hash
			})),
			module: optionalNestedConfig(snapshot.module, module => ({
				timestamp: module.timestamp,
				hash: module.hash
			}))
		})),
		cache: optionalNestedConfig(config.cache, cache => cache),
		stats: nestedConfig(config.stats, stats => {
			if (stats === false) {
				return {
					preset: "none"
				};
			}
			if (stats === true) {
				return {
					preset: "normal"
				};
			}
			if (typeof stats === "string") {
				return {
					preset: stats
				};
			}
			return {
				...stats
			};
		}),
		optimization: nestedConfig(config.optimization, optimization => {
			return {
				...optimization,
				runtimeChunk: getNormalizedOptimizationRuntimeChunk(
					optimization.runtimeChunk
				),
				splitChunks: nestedConfig(
					optimization.splitChunks,
					splitChunks =>
						splitChunks && {
							...splitChunks,
							cacheGroups: cloneObject(splitChunks.cacheGroups)
						}
				)
			};
		}),
		plugins: nestedArray(config.plugins, p => [...p]),
		experiments: nestedConfig(config.experiments, experiments => ({
			...experiments
		})),
		watch: config.watch,
		watchOptions: cloneObject(config.watchOptions),
		devServer: config.devServer,
		builtins: nestedConfig(config.builtins, builtins => ({
			...builtins
		}))
	};
};

const getNormalizedEntryStatic = (entry: EntryStatic) => {
	if (typeof entry === "string") {
		return {
			main: {
				import: [entry]
			}
		};
	}
	if (Array.isArray(entry)) {
		return {
			main: {
				import: entry
			}
		};
	}
	const result: EntryStaticNormalized = {};
	for (const key of Object.keys(entry)) {
		const value = entry[key];
		if (typeof value === "string") {
			result[key] = {
				import: [value]
			};
		} else if (Array.isArray(value)) {
			result[key] = {
				import: value
			};
		} else {
			result[key] = {
				import: Array.isArray(value.import) ? value.import : [value.import],
				runtime: value.runtime
			};
		}
	}
	return result;
};

const getNormalizedOptimizationRuntimeChunk = (
	runtimeChunk?: OptimizationRuntimeChunk
): OptimizationRuntimeChunkNormalized | undefined => {
	if (runtimeChunk === undefined) return undefined;
	if (runtimeChunk === false) return false;
	if (runtimeChunk === "single") {
		return {
			name: () => "runtime"
		};
	}
	if (runtimeChunk === true || runtimeChunk === "multiple") {
		return {
			name: (entrypoint: { name: string }) => `runtime~${entrypoint.name}`
		};
	}
	const { name } = runtimeChunk;
	return {
		name: typeof name === "function" ? name : () => name
	};
};

const nestedConfig = <T, R>(value: T | undefined, fn: (value: T) => R) =>
	value === undefined ? fn({} as T) : fn(value);

const optionalNestedConfig = <T, R>(
	value: T | undefined,
	fn: (value: T) => R
) => (value === undefined ? undefined : fn(value));

const nestedArray = <T, R>(value: T[] | undefined, fn: (value: T[]) => R[]) =>
	Array.isArray(value) ? fn(value) : fn([]);

const optionalNestedArray = <T, R>(
	value: T[] | undefined,
	fn: (value: T[]) => R[]
) => (Array.isArray(value) ? fn(value) : undefined);

const cloneObject = <T>(value?: T) => ({ ...value });

const keyedNestedConfig = <T, R>(
	value: Record<string, T> | undefined,
	fn: (value: T) => R,
	customKeys: Record<string, (value: T) => R>
) => {
	const result =
		value === undefined
			? {}
			: Object.keys(value).reduce(
					(obj, key) => (
						(obj[key] = (
							customKeys && key in customKeys ? customKeys[key] : fn
						)(value[key])),
						obj
					),
					{} as Record<string, R>
			  );
	if (customKeys) {
		for (const key of Object.keys(customKeys)) {
			if (!(key in result)) {
				result[key] = customKeys[key]({} as T);
			}
		}
	}
	return result;
};
