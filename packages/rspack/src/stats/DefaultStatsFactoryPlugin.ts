/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/stats
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import * as util from "node:util";

import type {
	JsOriginRecord,
	JsStatsAssetInfo,
	JsStatsError,
	JsStatsModule,
	JsStatsWarning
} from "@rspack/binding";
import type { Chunk } from "../Chunk";
import type { NormalizedStatsOptions } from "../Compilation";
import type { Compiler } from "../Compiler";
import type { StatsOptions } from "../config";
import {
	LogType,
	type LogTypeEnum,
	getLogTypeBitFlag,
	getLogTypesBitFlag
} from "../logging/Logger";
import {
	type Comparator,
	compareIds as _compareIds,
	compareNumbers,
	compareSelect
} from "../util/comparators";
import { makePathsRelative, parseResource } from "../util/identifier";
import type { GroupConfig } from "../util/smartGrouping";
import type { KnownStatsFactoryContext, StatsFactory } from "./StatsFactory";
import type {
	KnownStatsAsset,
	KnownStatsLoggingEntry,
	KnownStatsModule,
	PreprocessedAsset,
	SimpleExtractors,
	StatsAsset,
	StatsChunk,
	StatsChunkOrigin,
	StatsError,
	StatsProfile,
	StatsWarnings
} from "./statsFactoryUtils";
import {
	assetGroup,
	countWithChildren,
	iterateConfig,
	mergeToObject,
	moduleGroup,
	resolveStatsMillisecond,
	sortByField,
	spaceLimited
} from "./statsFactoryUtils";

const compareIds = _compareIds as <T>(a: T, b: T) => -1 | 0 | 1;
const GROUP_EXTENSION_REGEXP = /(\.[^.]+?)(?:\?|(?: \+ \d+ modules?)?$)/;
const GROUP_PATH_REGEXP = /(.+)[/\\][^/\\]+?(?:\?|(?: \+ \d+ modules?)?$)/;

const ITEM_NAMES: Record<string, string> = {
	"compilation.children[]": "compilation",
	"compilation.modules[]": "module",
	"compilation.entrypoints[]": "chunkGroup",
	"compilation.namedChunkGroups[]": "chunkGroup",
	"compilation.errors[]": "error",
	"compilation.warnings[]": "warning",
	"chunk.modules[]": "module",
	"chunk.rootModules[]": "module",
	"chunk.origins[]": "chunkOrigin",
	"compilation.chunks[]": "chunk",
	"compilation.assets[]": "asset",
	"asset.related[]": "asset",
	"module.issuerPath[]": "moduleIssuer",
	"module.reasons[]": "moduleReason",
	"module.modules[]": "module",
	"module.children[]": "module",
	"moduleTrace[]": "moduleTraceItem"
	// "moduleTraceItem.dependencies[]": "moduleTraceDependency"
};

const MERGER: Record<
	string,
	(
		items: {
			[key: string]: any;
			name: string;
		}[]
	) => any
> = {
	"compilation.entrypoints": mergeToObject,
	"compilation.namedChunkGroups": mergeToObject
};

const ASSETS_GROUPERS: Record<
	string,
	(
		groupConfigs: GroupConfig[],
		context: KnownStatsFactoryContext,
		options: NormalizedStatsOptions
	) => void
> = {
	_: (groupConfigs, context, options) => {
		const groupByFlag = (name: keyof KnownStatsAsset, exclude?: boolean) => {
			groupConfigs.push({
				getKeys: (asset: KnownStatsAsset) => {
					return asset[name] ? ["1"] : undefined;
				},
				getOptions: () => {
					return {
						groupChildren: !exclude,
						force: exclude
					};
				},
				// @ts-expect-error
				createGroup: (
					key: string,
					children: KnownStatsAsset[],
					assets: KnownStatsAsset[]
				) => {
					return exclude
						? {
								type: "assets by status",
								[name]: !!key,
								filteredChildren: assets.length,
								...assetGroup(children)
							}
						: {
								type: "assets by status",
								[name]: !!key,
								children,
								...assetGroup(children)
							};
				}
			});
		};
		const {
			groupAssetsByEmitStatus,
			groupAssetsByPath,
			groupAssetsByExtension
		} = options;
		if (groupAssetsByEmitStatus) {
			groupByFlag("emitted");
			// groupByFlag("comparedForEmit");
			// groupByFlag("isOverSizeLimit");
		}
		// if (groupAssetsByEmitStatus || !options.cachedAssets) {
		// 	groupByFlag("cached", !options.cachedAssets);
		// }
		if (groupAssetsByPath || groupAssetsByExtension) {
			groupConfigs.push({
				getKeys: asset => {
					const extensionMatch =
						groupAssetsByExtension && GROUP_EXTENSION_REGEXP.exec(asset.name);
					const extension = extensionMatch ? extensionMatch[1] : "";
					const pathMatch =
						groupAssetsByPath && GROUP_PATH_REGEXP.exec(asset.name);
					const path = pathMatch ? pathMatch[1].split(/[/\\]/) : [];
					const keys = [];
					if (groupAssetsByPath) {
						keys.push(".");
						if (extension)
							keys.push(
								path.length
									? `${path.join("/")}/*${extension}`
									: `*${extension}`
							);
						while (path.length > 0) {
							keys.push(path.join("/") + "/");
							path.pop();
						}
					} else {
						if (extension) keys.push(`*${extension}`);
					}
					return keys;
				},
				// @ts-expect-error
				createGroup: (key, children: KnownStatsAsset[]) => {
					return {
						type: groupAssetsByPath ? "assets by path" : "assets by extension",
						name: key,
						children,
						...assetGroup(children)
					};
				}
			});
		}
	},
	groupAssetsByInfo: groupConfigs => {
		const groupByAssetInfoFlag = (name: keyof JsStatsAssetInfo) => {
			groupConfigs.push({
				getKeys: asset => {
					return asset.info && asset.info[name] ? ["1"] : undefined;
				},
				// @ts-expect-error
				createGroup: (key, children: KnownStatsAsset[]) => {
					return {
						type: "assets by info",
						info: {
							[name]: !!key
						},
						children,
						...assetGroup(children)
					};
				}
			});
		};
		groupByAssetInfoFlag("immutable");
		groupByAssetInfoFlag("development");
		groupByAssetInfoFlag("hotModuleReplacement");
	},
	groupAssetsByChunk: groupConfigs => {
		const groupByNames = (
			name: keyof Pick<
				KnownStatsAsset,
				| "chunkNames"
				| "chunkIdHints"
				| "auxiliaryChunkNames"
				| "auxiliaryChunkIdHints"
			>
		) => {
			groupConfigs.push({
				getKeys: asset => {
					return asset[name];
				},
				// @ts-expect-error
				createGroup: (key, children: KnownStatsAsset[]) => {
					return {
						type: "assets by chunk",
						[name]: [key],
						children,
						...assetGroup(children)
					};
				}
			});
		};
		groupByNames("chunkNames");
		groupByNames("auxiliaryChunkNames");
		groupByNames("chunkIdHints");
		groupByNames("auxiliaryChunkIdHints");
	},
	excludeAssets: (groupConfigs, context, { excludeAssets }) => {
		groupConfigs.push({
			getKeys: asset => {
				const ident = asset.name;
				const excluded = excludeAssets.some(fn => fn(ident, asset));
				if (excluded) return ["excluded"];
			},
			getOptions: () => ({
				groupChildren: false,
				force: true
			}),
			// @ts-expect-error
			createGroup: (key, children: KnownStatsAsset[], assets) => ({
				type: "hidden assets",
				filteredChildren: assets.length,
				...assetGroup(children)
			})
		});
	}
};

const MODULES_GROUPERS = (
	type: "module" | "chunk" | "root-of-chunk" | "nested"
): Record<
	string,
	(
		groupConfigs: GroupConfig[],
		context: KnownStatsFactoryContext,
		options: NormalizedStatsOptions
	) => void
> => ({
	_: (groupConfigs, context, options) => {
		const groupByFlag = (name: string, type: unknown, exclude?: boolean) => {
			groupConfigs.push({
				getKeys: module => {
					return module[name] ? ["1"] : undefined;
				},
				getOptions: () => {
					return {
						groupChildren: !exclude,
						force: exclude
					};
				},
				// @ts-expect-error
				createGroup: (
					key,
					children: KnownStatsModule[],
					modules: KnownStatsModule[]
				) => {
					return {
						type,
						[name]: !!key,
						...(exclude ? { filteredChildren: modules.length } : { children }),
						...moduleGroup(children)
					};
				}
			});
		};
		const {
			groupModulesByCacheStatus,
			groupModulesByAttributes,
			groupModulesByType,
			groupModulesByPath,
			groupModulesByExtension
		} = options;
		if (groupModulesByAttributes) {
			groupByFlag("errors", "modules with errors");
			groupByFlag("warnings", "modules with warnings");
			groupByFlag("assets", "modules with assets");
			groupByFlag("optional", "optional modules");
		}
		if (groupModulesByCacheStatus) {
			groupByFlag("cacheable", "cacheable modules");
			groupByFlag("built", "built modules");
			groupByFlag("codeGenerated", "code generated modules");
		}
		if (groupModulesByCacheStatus || !options.cachedModules) {
			groupByFlag("cached", "cached modules", !options.cachedModules);
		}
		if (groupModulesByAttributes || !options.orphanModules) {
			groupByFlag("orphan", "orphan modules", !options.orphanModules);
		}
		if (groupModulesByAttributes || !options.dependentModules) {
			groupByFlag("dependent", "dependent modules", !options.dependentModules);
		}
		if (groupModulesByType || !options.runtimeModules) {
			groupConfigs.push({
				getKeys: (module: KnownStatsModule) => {
					if (!module.moduleType) return;
					if (groupModulesByType) {
						return [module.moduleType.split("/", 1)[0]];
					}
					if (module.moduleType === "runtime") {
						return ["runtime"];
					}
				},
				getOptions: key => {
					const exclude = key === "runtime" && !options.runtimeModules;
					return {
						groupChildren: !exclude,
						force: exclude
					};
				},
				// @ts-expect-error
				createGroup: (key, children: KnownStatsModule[], modules) => {
					const exclude = key === "runtime" && !options.runtimeModules;
					return {
						type: `${key} modules`,
						moduleType: key,
						...(exclude ? { filteredChildren: modules.length } : { children }),
						...moduleGroup(children)
					};
				}
			});
		}
		// not support groupModulesByLayer
		if (groupModulesByPath || groupModulesByExtension) {
			groupConfigs.push({
				getKeys: module => {
					if (!module.name) return;
					const resource = parseResource(module.name.split("!").pop()).path;
					const dataUrl = /^data:[^,;]+/.exec(resource);
					if (dataUrl) return [dataUrl[0]];
					const extensionMatch =
						groupModulesByExtension && GROUP_EXTENSION_REGEXP.exec(resource);
					const extension = extensionMatch ? extensionMatch[1] : "";
					const pathMatch =
						groupModulesByPath && GROUP_PATH_REGEXP.exec(resource);
					const path = pathMatch ? pathMatch[1].split(/[/\\]/) : [];
					const keys = [];
					if (groupModulesByPath) {
						if (extension)
							keys.push(
								path.length
									? `${path.join("/")}/*${extension}`
									: `*${extension}`
							);
						while (path.length > 0) {
							keys.push(path.join("/") + "/");
							path.pop();
						}
					} else {
						if (extension) keys.push(`*${extension}`);
					}
					return keys;
				},
				// @ts-expect-error
				createGroup: (key, children: KnownStatsModule[], modules) => {
					const isDataUrl = key.startsWith("data:");
					return {
						type: isDataUrl
							? "modules by mime type"
							: groupModulesByPath
								? "modules by path"
								: "modules by extension",
						name: isDataUrl ? key.slice(/* 'data:'.length */ 5) : key,
						children,
						...moduleGroup(children)
					};
				}
			});
		}
	},
	excludeModules: (groupConfigs, context, { excludeModules }) => {
		groupConfigs.push({
			getKeys: module => {
				const name = module.name;
				if (name) {
					const excluded = excludeModules.some(fn => fn(name, module, type));
					if (excluded) return ["1"];
				}
			},
			getOptions: () => ({
				groupChildren: false,
				force: true
			}),
			// @ts-expect-error
			createGroup: (key, children: KnownStatsModule[], modules) => ({
				type: "hidden modules",
				filteredChildren: children.length,
				...moduleGroup(children)
			})
		});
	}
});

const RESULT_GROUPERS: Record<
	string,
	Record<
		string,
		(
			groupConfigs: GroupConfig[],
			context: KnownStatsFactoryContext,
			options: NormalizedStatsOptions
		) => void
	>
> = {
	"compilation.assets": ASSETS_GROUPERS,
	"asset.related": ASSETS_GROUPERS,
	"compilation.modules": MODULES_GROUPERS("module"),
	"chunk.modules": MODULES_GROUPERS("chunk"),
	"chunk.rootModules": MODULES_GROUPERS("root-of-chunk"),
	"module.modules": MODULES_GROUPERS("nested")
};

const ASSET_SORTERS = {
	assetsSort: (
		comparators: Function[],
		_context: KnownStatsFactoryContext,
		{ assetsSort }: NormalizedStatsOptions
	) => {
		comparators.push(sortByField(assetsSort));
	},
	_: (comparators: Function[]) => {
		comparators.push(compareSelect((a: StatsAsset) => a.name, compareIds));
	}
};

const RESULT_SORTERS: Record<
	string,
	Record<
		string,
		(
			comparators: Function[],
			context: KnownStatsFactoryContext,
			options: NormalizedStatsOptions
		) => void
	>
> = {
	"compilation.chunks": {
		chunksSort: (comparators, context, { chunksSort }) => {
			comparators.push(sortByField(chunksSort));
		}
	},
	"compilation.modules": {
		modulesSort: (comparators, context, { modulesSort }) => {
			comparators.push(sortByField(modulesSort));
		}
	},
	"chunk.modules": {
		chunkModulesSort: (comparators, context, { chunkModulesSort }) => {
			comparators.push(sortByField(chunkModulesSort));
		}
	},
	"module.modules": {
		nestedModulesSort: (comparators, context, { nestedModulesSort }) => {
			comparators.push(sortByField(nestedModulesSort));
		}
	},
	"compilation.assets": ASSET_SORTERS,
	"asset.related": ASSET_SORTERS
};

const MODULES_SORTER: Record<
	string,
	(comparators: Function[], context: KnownStatsFactoryContext) => void
> = {
	_: comparators => {
		comparators.push(
			compareSelect(
				(m: JsStatsModule) => m.depth,
				compareNumbers as Comparator
			),
			compareSelect(
				(m: JsStatsModule) => m.preOrderIndex,
				compareNumbers as Comparator
			),
			compareSelect(
				(m: JsStatsModule) => m.identifier,
				compareIds as Comparator
			)
		);
	}
};

const SORTERS: Record<
	string,
	Record<
		string,
		(comparators: Function[], context: KnownStatsFactoryContext) => void
	>
> = {
	"compilation.chunks": {
		_: comparators => {
			comparators.push(compareSelect((c: StatsChunk) => c.id, compareIds));
		}
	},
	"compilation.modules": MODULES_SORTER,
	"chunk.rootModules": MODULES_SORTER,
	"chunk.modules": MODULES_SORTER,
	"module.modules": MODULES_SORTER,
	// not support module.reasons (missing Module.identifier())
	"chunk.origins": {
		_: comparators => {
			comparators.push(
				// compareSelect(
				// 	origin =>
				// 		origin.module ? chunkGraph.getModuleId(origin.module) : undefined,
				// 	compareIds
				// ),
				compareSelect((origin: JsOriginRecord) => origin.loc, compareIds),
				compareSelect((origin: JsOriginRecord) => origin.request, compareIds)
			);
		}
	}
};

const EXTRACT_ERROR: Record<
	string,
	(
		object: StatsError,
		error: JsStatsError | JsStatsWarning,
		context: KnownStatsFactoryContext,
		options: StatsOptions,
		factory: StatsFactory
	) => void
> = {
	_: (object, error) => {
		object.message = error.message;
		object.chunkName = error.chunkName;
		object.chunkEntry = error.chunkEntry;
		object.chunkInitial = error.chunkInitial;
		object.file = error.file;
		if (error.moduleIdentifier) {
			object.moduleIdentifier = error.moduleIdentifier.toString();
		}
		object.moduleName = error.moduleName;
	},
	ids: (object, error) => {
		object.chunkId = error.chunkId;
		object.moduleId = error.moduleId;
	},
	moduleTrace: (object, error, context, _, factory) => {
		const { type } = context;
		object.moduleTrace = factory.create(
			`${type}.moduleTrace`,
			error.moduleTrace,
			context
		);
	},
	errorDetails: (object, error) => {
		object.details = error.details;
	},
	errorStack: (object, error) => {
		object.stack = error.stack;
	}
};

const SIMPLE_EXTRACTORS: SimpleExtractors = {
	compilation: {
		_: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			options: StatsOptions
		) => {
			if (!context.makePathsRelative) {
				context.makePathsRelative = makePathsRelative.bindContextCache(
					compilation.compiler.context,
					compilation.compiler.root
				);
			}
			if (!context.cachedGetErrors) {
				context.cachedGetErrors = _compilation => {
					return context.getInner(compilation).getErrors();
				};
			}
			if (!context.cachedGetWarnings) {
				context.cachedGetWarnings = _compilation => {
					const warnings = context.getInner(compilation).getWarnings();

					return compilation.hooks.processWarnings.call(
						warnings as any
					) as unknown as typeof warnings;
				};
			}
			if (compilation.name) {
				object.name = compilation.name;
			}
			// TODO: support compilation.needAdditionalPass
			const logging = options.logging!;
			const loggingDebug = options.loggingDebug as ((
				value: string
			) => boolean)[];
			const loggingTrace = options.loggingTrace!;
			if (logging || (loggingDebug && loggingDebug.length > 0)) {
				let collapsedGroups = false;
				let acceptedTypes: number;
				if (
					logging === "verbose" ||
					(loggingDebug && loggingDebug.length > 0)
				) {
					acceptedTypes = getLogTypesBitFlag([
						LogType.error,
						LogType.warn,
						LogType.info,
						LogType.log,
						LogType.group,
						LogType.groupEnd,
						LogType.groupCollapsed,
						LogType.profile,
						LogType.profileEnd,
						LogType.time,
						LogType.status,
						LogType.clear,
						LogType.cache
					]);
					collapsedGroups = true;
				} else if (logging === "log" || logging === true) {
					acceptedTypes = getLogTypesBitFlag([
						LogType.error,
						LogType.warn,
						LogType.info,
						LogType.log,
						LogType.group,
						LogType.groupEnd,
						LogType.groupCollapsed,
						LogType.clear
					]);
				} else if (logging === "info") {
					acceptedTypes = getLogTypesBitFlag([
						LogType.error,
						LogType.warn,
						LogType.info
					]);
				} else if (logging === "warn") {
					acceptedTypes = getLogTypesBitFlag([LogType.error, LogType.warn]);
				} else if (logging === "error") {
					acceptedTypes = getLogTypesBitFlag([LogType.error]);
				} else {
					acceptedTypes = getLogTypesBitFlag([]);
				}
				object.logging = {};
				const compilationLogging = compilation.logging;
				for (const { name, ...rest } of context
					.getInner(compilation)
					.getLogging(acceptedTypes)) {
					const value = compilationLogging.get(name);
					const entry = {
						type: rest.type,
						trace: rest.trace,
						args: rest.args ?? []
					};
					if (value) {
						value.push(entry);
					} else {
						compilationLogging.set(name, [entry]);
					}
				}
				let depthInCollapsedGroup = 0;
				for (const [origin, logEntries] of compilationLogging) {
					const debugMode = loggingDebug.some(fn => fn(origin));
					if (logging === false && !debugMode) continue;
					const groupStack: KnownStatsLoggingEntry[] = [];
					const rootList: KnownStatsLoggingEntry[] = [];
					let currentList = rootList;
					let processedLogEntries = 0;
					for (const entry of logEntries) {
						let type = entry.type;
						const typeBitFlag = getLogTypeBitFlag(type as LogTypeEnum);
						if (!debugMode && (acceptedTypes & typeBitFlag) !== typeBitFlag)
							continue;
						// Expand groups in verbose and debug modes
						if (
							type === LogType.groupCollapsed &&
							(debugMode || collapsedGroups)
						)
							type = LogType.group;

						if (depthInCollapsedGroup === 0) {
							processedLogEntries++;
						}

						if (type === LogType.groupEnd) {
							groupStack.pop();
							if (groupStack.length > 0) {
								currentList = groupStack[groupStack.length - 1].children!;
							} else {
								currentList = rootList;
							}
							if (depthInCollapsedGroup > 0) depthInCollapsedGroup--;
							continue;
						}
						const message =
							entry.args && entry.args.length > 0
								? util.format(entry.args[0], ...entry.args.slice(1))
								: "";
						const newEntry: KnownStatsLoggingEntry = {
							type,
							message,
							trace: loggingTrace ? entry.trace : undefined,
							children:
								type === LogType.group || type === LogType.groupCollapsed
									? []
									: undefined
						};
						currentList.push(newEntry);
						if (newEntry.children) {
							groupStack.push(newEntry);
							currentList = newEntry.children;
							if (depthInCollapsedGroup > 0) {
								depthInCollapsedGroup++;
							} else if (type === LogType.groupCollapsed) {
								depthInCollapsedGroup = 1;
							}
						}
					}
					object.logging[origin] = {
						entries: rootList,
						filteredEntries: logEntries.length - processedLogEntries,
						debug: debugMode
					};
				}
			}
		},
		hash: (object, compilation, context: KnownStatsFactoryContext) => {
			object.hash = context.getInner(compilation).getHash() || undefined;
		},
		version: object => {
			const { version, webpackVersion } = require("../../package.json");
			object.version = webpackVersion;
			object.rspackVersion = version;
		},
		env: (object, _compilation, _context, { _env }) => {
			object.env = _env;
		},
		timings: (object, compilation) => {
			object.time = compilation.endTime! - compilation.startTime!;
		},
		builtAt: (object, compilation) => {
			object.builtAt = compilation.endTime;
		},
		publicPath: (object, compilation) => {
			object.publicPath = compilation.getPath(
				compilation.outputOptions.publicPath || ""
			);
		},
		outputPath: (object, compilation) => {
			object.outputPath = compilation.outputOptions.path;
		},
		assets: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			options,
			factory
		) => {
			const { assets: compilationAssets, assetsByChunkName } = context
				.getInner(compilation)
				.getAssets();

			const assetMap: Map<String, PreprocessedAsset> = new Map();
			const assets = new Set();

			for (const asset of compilationAssets) {
				const item: PreprocessedAsset = {
					...asset,
					type: "asset",
					related: []
				};
				assets.add(item);
				assetMap.set(asset.name, item);
			}

			for (const item of assetMap.values()) {
				const related = item.info.related;
				if (!related) continue;
				for (const { name: type, value: relatedEntry } of related) {
					const deps = Array.isArray(relatedEntry)
						? relatedEntry
						: [relatedEntry];
					for (const dep of deps) {
						const depItem = assetMap.get(dep);
						if (!depItem) continue;
						assets.delete(depItem);
						depItem.type = type;
						item.related = item.related || [];
						item.related.push(depItem);
					}
				}
			}

			object.assetsByChunkName = assetsByChunkName.reduce<
				Record<string, string[]>
			>((acc, cur) => {
				acc[cur.name] = cur.files;
				return acc;
			}, {});

			const groupedAssets = factory.create(
				`${context.type}.assets`,
				Array.from(assets),
				{
					...context
					// compilationFileToChunks
					// compilationAuxiliaryFileToChunks
				}
			);
			const limited = spaceLimited(
				groupedAssets,
				options.assetsSpace ?? Number.POSITIVE_INFINITY
			);
			object.assets = limited.children;
			object.filteredAssets = limited.filteredChildren;
		},
		chunks: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			options: NormalizedStatsOptions,
			factory
		) => {
			const { type } = context;
			const chunks = context.getInner(compilation).getChunks(options);
			object.chunks = factory.create(`${type}.chunks`, chunks, context);
		},
		modules: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			options: NormalizedStatsOptions,
			factory
		) => {
			const { type } = context;
			const array = context.getInner(compilation).getModules(options);
			const groupedModules = factory.create(`${type}.modules`, array, context);
			const limited = spaceLimited(groupedModules, options.modulesSpace!);
			object.modules = limited.children;
			object.filteredModules = limited.filteredChildren;
		},
		entrypoints: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			{ entrypoints, chunkGroups, chunkGroupAuxiliary, chunkGroupChildren },
			factory
		) => {
			const { type } = context;
			const array = context
				.getInner(compilation)
				.getEntrypoints(chunkGroupAuxiliary, chunkGroupChildren)
				.map(entrypoint => ({
					name: entrypoint.name,
					chunkGroup: entrypoint
				}));

			const chunks = Array.from(compilation.chunks).reduce<
				Record<string, Chunk>
			>((res, chunk) => {
				res[chunk.id!] = chunk;
				return res;
			}, {});

			if (entrypoints === "auto" && !chunkGroups) {
				if (array.length > 5) return;
				if (
					!chunkGroupChildren &&
					array.every(({ chunkGroup }) => {
						if (chunkGroup.chunks.length !== 1) return false;
						const chunk = chunks[chunkGroup.chunks[0]!];
						return (
							chunk &&
							chunk.files.size === 1 &&
							(!chunkGroupAuxiliary || chunk.auxiliaryFiles.size === 0)
						);
					})
				) {
					return;
				}
			}

			object.entrypoints = factory.create(
				`${type}.entrypoints`,
				array,
				context
			);
		},
		chunkGroups: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			{ chunkGroupAuxiliary, chunkGroupChildren },
			factory
		) => {
			const { type } = context;
			const namedChunkGroups = context
				.getInner(compilation)
				.getNamedChunkGroups(chunkGroupAuxiliary, chunkGroupChildren)
				.map(cg => ({
					name: cg.name,
					chunkGroup: cg
				}));

			object.namedChunkGroups = factory.create(
				`${type}.namedChunkGroups`,
				namedChunkGroups,
				context
			);
		},
		errors: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			_options,
			_factory
		) => {
			const { cachedGetErrors } = context;
			object.errors = cachedGetErrors!(compilation).map(error => {
				const { moduleIdentifier, ...rest } = error;
				const result: StatsError = rest;
				if (moduleIdentifier) {
					result.moduleIdentifier = moduleIdentifier.toString();
				}
				return result;
			});
		},
		errorsCount: (
			object,
			compilation,
			{ cachedGetErrors }: KnownStatsFactoryContext
		) => {
			object.errorsCount = countWithChildren(compilation, c =>
				cachedGetErrors!(c)
			);
		},
		warnings: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			_options,
			_factory
		) => {
			const { cachedGetWarnings } = context;
			object.warnings = cachedGetWarnings!(compilation).map(warning => {
				const { moduleIdentifier, ...rest } = warning;
				const result: StatsWarnings = rest;
				if (moduleIdentifier) {
					result.moduleIdentifier = moduleIdentifier.toString();
				}
				return result;
			});
		},
		warningsCount: (object, compilation, context: KnownStatsFactoryContext) => {
			const { cachedGetWarnings } = context;
			object.warningsCount = countWithChildren(compilation, c => {
				return cachedGetWarnings!(c);
			});
		},
		children: (object, compilation, context, _options, factory) => {
			const { type } = context;
			object.children = factory.create(
				`${type}.children`,
				compilation.children,
				context
			);
		}
	},
	asset: {
		_: (object, asset, context: KnownStatsFactoryContext, options, factory) => {
			object.type = asset.type;
			object.name = asset.name;
			object.size = asset.size;
			object.emitted = asset.emitted;
			object.info = {
				...asset.info,
				related: Object.fromEntries(
					asset.info.related.map(i => [i.name, i.value])
				)
			};
			// - comparedForEmit
			// - cached
			Object.assign(
				object,
				factory.create(`${context.type}$visible`, asset, context)
			);
		}
	},
	asset$visible: {
		_: (object, asset) => {
			object.chunkNames = asset.chunkNames;
			object.chunkIdHints = asset.chunkIdHints.filter(Boolean);
			object.auxiliaryChunkNames = asset.auxiliaryChunkNames;
			object.auxiliaryChunkIdHints =
				asset.auxiliaryChunkIdHints.filter(Boolean);
		},
		relatedAssets: (object, asset, context, options, factory) => {
			const { type } = context;
			object.related = factory.create(
				`${type.slice(0, -8)}.related`,
				asset.related,
				context
			);
			object.filteredRelated = asset.related
				? asset.related.length - object.related.length
				: undefined;
		},
		ids: (object, asset) => {
			object.chunks = asset.chunks;
			object.auxiliaryChunks = asset.auxiliaryChunks;
		}
	},
	chunkGroup: {
		_: (
			object,
			{ name, chunkGroup },
			_context: KnownStatsFactoryContext,
			{ chunkGroupMaxAssets }
		) => {
			object.name = name;
			object.chunks = chunkGroup.chunks;
			object.assets = chunkGroup.assets;
			object.filteredAssets =
				chunkGroup.assets.length <= chunkGroupMaxAssets
					? 0
					: chunkGroup.assets.length;
			object.assetsSize = chunkGroup.assetsSize;
			object.auxiliaryAssets = chunkGroup.auxiliaryAssets;
			object.auxiliaryAssetsSize = chunkGroup.auxiliaryAssetsSize;
			object.children = chunkGroup.children;
			// - childAssets
		}
	},
	module: {
		_: (
			object,
			module,
			context: KnownStatsFactoryContext,
			options,
			factory
		) => {
			const { type } = context;
			object.type = module.type;
			object.moduleType = module.moduleType;
			// TODO: object.layer = module.layer;
			object.size = module.size;
			const sizes = module.sizes.map(({ sourceType, size }) => [
				sourceType,
				size
			]);
			sizes.sort((a, b) => -compareIds(a, b));
			object.sizes = Object.fromEntries(sizes);
			object.built = module.built;
			object.codeGenerated = module.codeGenerated;
			object.buildTimeExecuted = module.buildTimeExecuted;
			object.cached = module.cached;
			if (module.built || module.codeGenerated || options.cachedModules) {
				Object.assign(
					object,
					factory.create(`${type}$visible`, module, context)
				);
			}
		}
	},
	module$visible: {
		_: (object, module, context, options, factory) => {
			const { type } = context;
			if (module.identifier) {
				object.identifier = module.identifier.toString();
			}
			object.name = module.name;
			object.nameForCondition = module.nameForCondition;
			object.index = module.preOrderIndex;
			object.preOrderIndex = module.preOrderIndex;
			object.index2 = module.postOrderIndex;
			object.postOrderIndex = module.postOrderIndex;
			object.cacheable = module.cacheable;
			object.optional = module.optional;
			object.orphan = module.orphan;
			object.dependent = module.dependent;
			object.issuer = module.issuer;
			object.issuerName = module.issuerName;
			object.issuerPath = factory.create(
				`${type.slice(0, -8)}.issuerPath`,
				module.issuerPath,
				context
			);
			object.failed = module.failed;
			object.errors = module.errors;
			object.warnings = module.warnings;
			const profile = module.profile;
			if (profile) {
				object.profile = factory.create(`${type}.profile`, profile, context);
			}
		},
		ids: (object, module) => {
			object.id = module.id;
			object.issuerId = module.issuerId;
			object.chunks = module.chunks;
		},
		moduleAssets: (object, module) => {
			object.assets = module.assets;
		},
		reasons: (object, module, context, options, factory) => {
			const { type } = context;
			object.reasons = factory.create(
				`${type.slice(0, -8)}.reasons`,
				module.reasons,
				context
			);
			// object.filteredReasons
		},
		source: (object, module) => {
			object.source = module.source;
		},
		usedExports: (object, module) => {
			if (typeof module.usedExports === "string") {
				if (module.usedExports === "null") {
					object.usedExports = null;
				} else {
					object.usedExports = module.usedExports === "true";
				}
			} else if (Array.isArray(module.usedExports)) {
				object.usedExports = module.usedExports;
			} else {
				object.usedExports = null;
			}
		},
		providedExports: (object, module) => {
			if (Array.isArray(module.providedExports)) {
				object.providedExports = module.providedExports;
			} else {
				object.providedExports = null;
			}
		},
		optimizationBailout: (object, module) => {
			object.optimizationBailout = module.optimizationBailout || null;
		},
		depth: (object, module) => {
			object.depth = module.depth;
		},
		nestedModules: (object, module, context, options, factory) => {
			const { type } = context;
			const innerModules =
				/** @type {Module & { modules?: Module[] }} */ module.modules;
			if (Array.isArray(innerModules) && innerModules.length > 0) {
				const groupedModules = factory.create(
					`${type.slice(0, -8)}.modules`,
					innerModules,
					context
				);
				const limited = spaceLimited(
					groupedModules,
					options.nestedModulesSpace
				);
				object.modules = limited.children;
				object.filteredModules = limited.filteredChildren;
			}
		}
	},
	profile: {
		_: (object, profile) => {
			const factory = resolveStatsMillisecond(profile.factory);
			const building = resolveStatsMillisecond(profile.building);
			const statsProfile: StatsProfile = {
				total: factory + building,
				resolving: factory,
				building
			};
			Object.assign(object, statsProfile);
		}
	},
	moduleIssuer: {
		_: (object, module, context, options, factory) => {
			if (module.identifier) {
				object.identifier = module.identifier.toString();
			}
			object.name = module.name;
		},
		ids: (object, module) => {
			object.id = module.id;
		}
	},
	moduleReason: {
		_: (object, reason) => {
			if (reason.moduleIdentifier) {
				object.moduleIdentifier = reason.moduleIdentifier.toString();
			}
			object.moduleName = reason.moduleName;
			object.type = reason.type;
			object.userRequest = reason.userRequest;
		},
		ids: (object, reason) => {
			object.moduleId = reason.moduleId;
		}
	},
	chunk: {
		_: (object, chunk) => {
			object.type = chunk.type;
			object.rendered = chunk.rendered;
			object.initial = chunk.initial;
			object.entry = chunk.entry;
			object.reason = chunk.reason;
			object.size = chunk.size;
			object.sizes = Object.fromEntries(
				chunk.sizes.map(({ sourceType, size }) => [sourceType, size])
			);
			object.names = chunk.names;
			object.idHints = chunk.idHints;
			object.runtime = chunk.runtime;
			object.files = chunk.files;
			object.auxiliaryFiles = chunk.auxiliaryFiles;
			object.hash = chunk.hash;
			object.childrenByOrder = chunk.childrenByOrder;
		},
		ids: (object, chunk) => {
			object.id = chunk.id;
		},
		chunkRelations: (object, chunk) => {
			object.siblings = chunk.siblings;
			object.parents = chunk.parents;
			object.children = chunk.children;
		},
		chunkModules: (object, chunk, context, options, factory) => {
			const { type } = context;
			object.modules = factory.create(
				`${type}.modules`,
				chunk.modules,
				context
			);
		},
		chunkOrigins: (object, chunk, context, options, factory) => {
			object.origins = chunk.origins.map<StatsChunkOrigin>(origin => ({
				...origin,
				module: origin.module.toString(),
				moduleIdentifier: origin.moduleIdentifier.toString()
			}));
		}
	},
	// chunkOrigin
	error: EXTRACT_ERROR,
	warning: EXTRACT_ERROR,
	moduleTraceItem: {
		_: (object, { origin, module }, context, { requestShortener }, factory) => {
			object.originIdentifier = origin.identifier.toString();
			object.originName = origin.name;
			object.moduleIdentifier = module.identifier.toString();
			object.moduleName = module.name;
		},
		ids: (object, { origin, module }) => {
			object.originId = origin.id;
			object.moduleId = module.id;
		}
	}
	// moduleTraceDependency
};

/**
 * only support below factories:
 * - compilation
 * - compilation.assets
 * - compilation.assets[].asset
 * - compilation.chunks
 * - compilation.chunks[].chunk
 * - compilation.modules
 * - compilation.modules[].module
 */
export class DefaultStatsFactoryPlugin {
	apply(compiler: Compiler) {
		compiler.hooks.compilation.tap("DefaultStatsFactoryPlugin", compilation => {
			compilation.hooks.statsFactory.tap(
				"DefaultStatsFactoryPlugin",
				// @ts-expect-error
				(stats: StatsFactory, options: NormalizedStatsOptions, context) => {
					iterateConfig(SIMPLE_EXTRACTORS, options, (hookFor, fn) => {
						stats.hooks.extract
							.for(hookFor)
							.tap("DefaultStatsFactoryPlugin", (obj, data, ctx) =>
								fn(obj, data, ctx, options, stats)
							);
					});
					// not support filter module.reasons.!orphanModules
					// iterateConfig(FILTER, options, (hookFor, fn) => {
					// 	stats.hooks.filter
					// 		.for(hookFor)
					// 		.tap("DefaultStatsFactoryPlugin", (item, ctx, idx, i) =>
					// 			fn(item, ctx, options, idx, i)
					// 		);
					// });
					iterateConfig(SORTERS, options, (hookFor, fn) => {
						stats.hooks.sort
							.for(hookFor)
							.tap("DefaultStatsFactoryPlugin", (comparators, ctx) =>
								fn(comparators, ctx, options)
							);
					});
					iterateConfig(RESULT_SORTERS, options, (hookFor, fn) => {
						stats.hooks.sortResults
							.for(hookFor)
							.tap("DefaultStatsFactoryPlugin", (comparators, ctx) =>
								fn(comparators, ctx, options)
							);
					});
					iterateConfig(RESULT_GROUPERS, options, (hookFor, fn) => {
						stats.hooks.groupResults
							.for(hookFor)
							.tap("DefaultStatsFactoryPlugin", (groupConfigs, ctx) =>
								fn(groupConfigs, ctx, options)
							);
					});
					for (const key of Object.keys(ITEM_NAMES)) {
						const itemName = ITEM_NAMES[key];
						stats.hooks.getItemName
							.for(key)
							.tap("DefaultStatsFactoryPlugin", () => itemName);
					}
					for (const key of Object.keys(MERGER)) {
						const merger = MERGER[key];
						stats.hooks.merge.for(key).tap("DefaultStatsFactoryPlugin", merger);
					}
				}
			);
		});
	}
}
