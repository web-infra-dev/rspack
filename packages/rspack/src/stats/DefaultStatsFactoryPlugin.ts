/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/stats
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import { compareSelect, compareIds as _compareIds } from "../util/comparators";
import { makePathsRelative } from "../util/identifier";
import type { Compiler } from "../compiler";
import type { StatsOptions } from "../config";
import type { GroupConfig } from "../util/smartGrouping";

import type { StatsFactory, KnownStatsFactoryContext } from "./StatsFactory";
import {
	iterateConfig,
	// spaceLimited,
	moduleGroup,
	// uniqueOrderedArray,
	countWithChildren,
	sortByField,
	assetGroup
} from "./statsFactoryUtils";
import type {
	KnownStatsAsset,
	KnownStatsModule,
	KnownStatsChunkGroup,
	SimpleExtractors,
	StatsAsset,
	StatsChunk,
	NormalizedStatsOptions
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
	"module.children[]": "module"
	// "moduleTrace[]": "moduleTraceItem",
	// "moduleTraceItem.dependencies[]": "moduleTraceDependency"
};

// const MERGER: Record<
// 	string,
// 	(
// 		items: {
// 			[key: string]: any;
// 			name: string;
// 		}[]
// 	) => any
// > = {
// 	"compilation.entrypoints": mergeToObject,
// 	"compilation.namedChunkGroups": mergeToObject
// };

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
	}
	// not support groupAssetsByInfo / groupAssetsByChunk / excludeAssets
};

const MODULES_GROUPERS = (
	_type: "module" | "chunk" | "root-of-chunk" | "nested"
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
			groupModulesByLayer,
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
					} else if (module.moduleType === "runtime") {
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
		// not support groupModulesByLayer / groupModulesByPath / groupModulesByExtension
	}
	// not support excludeModules
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
	}
	// not support compilation.modules / chunk.rootModules / chunk.modules / module.modules  (missing Module.moduleGraph)
	// "compilation.modules": MODULES_SORTER,
	// "chunk.rootModules": MODULES_SORTER,
	// "chunk.modules": MODULES_SORTER,
	// "module.modules": MODULES_SORTER
	// not support module.reasons (missing Module.identifier())
	// not support chunk.origins (missing compilation.chunkGraph)
};

const SIMPLE_EXTRACTORS: SimpleExtractors = {
	compilation: {
		_: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			_options: StatsOptions
		) => {
			if (!context.makePathsRelative) {
				context.makePathsRelative = makePathsRelative.bindContextCache(
					compilation.compiler.context,
					compilation.compiler.root
				);
			}
			if (!context.cachedGetErrors) {
				context.cachedGetErrors = _compilation => {
					return context._inner.getErrors();
				};
			}
			if (!context.cachedGetWarnings) {
				context.cachedGetWarnings = _compilation => {
					return context._inner.getWarnings();
				};
			}
			if (compilation.name) {
				object.name = compilation.name;
			}
			// TODO: support compilation.needAdditionalPass
			// TODO: support compilation.logging
		},
		hash: (object, _compilation, context: KnownStatsFactoryContext) => {
			object.hash = context._inner.getHash();
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
			_compilation,
			context: KnownStatsFactoryContext,
			_options,
			_factory
		) => {
			const { assets, assetsByChunkName } = context._inner.getAssets();
			object.assetsByChunkName = assetsByChunkName.reduce<
				Record<string, string[]>
			>((acc, cur) => {
				acc[cur.name] = cur.files;
				return acc;
			}, {});

			object.assets = assets;

			// const groupedAssets = factory.create(
			// 	`${type}.assets`,
			// 	Array.from(assets),
			// 	context
			// );
			// const limited = spaceLimited(groupedAssets, options.assetsSpace);
			// object.assets = limited.children;
		},
		chunks: (
			object,
			_compilation,
			context: KnownStatsFactoryContext,
			options: StatsOptions,
			factory
		) => {
			const { type } = context;
			const chunks = context._inner.getChunks(
				options.chunkModules!,
				options.chunkRelations!,
				options.reasons!,
				options.moduleAssets!,
				options.nestedModules!
				// options.source!
			);
			// object.chunks = factory.create(
			// 	`${type}.chunks`,
			// 	Array.from(chunks),
			// 	context
			// );

			object.chunks = chunks;
		},
		modules: (
			object,
			_compilation,
			context: KnownStatsFactoryContext,
			options: StatsOptions,
			_factory
		) => {
			const { type } = context;
			const modules = context._inner.getModules(
				options.reasons!,
				options.moduleAssets!,
				options.nestedModules!
				// options.source!
			);
			// const array = Array.from(modules);
			// const groupedModules = factory.create(`${type}.modules`, array, context);
			// // options.modulesSpace
			// const limited = spaceLimited(groupedModules, 15);
			// object.modules = limited.children;
			// object.filteredModules = limited.filteredChildren;

			object.modules = modules;
			if (options.modules && modules.length > 15) {
				object.modules = modules.slice(0, 15);
				object.filteredModules = modules.length - 15;
			}
		},
		entrypoints: (
			object,
			_compilation,
			context: KnownStatsFactoryContext,
			_data,
			_factory
		) => {
			// const { type } = context;
			const array = context._inner.getEntrypoints();

			// object.entrypoints = factory.create(
			// 	`${type}.entrypoints`,
			// 	array,
			// 	context
			// );

			object.entrypoints = array.reduce<Record<string, KnownStatsChunkGroup>>(
				(acc, cur) => {
					acc[cur.name] = cur;
					return acc;
				},
				{}
			);
		},
		chunkGroups: (
			object,
			_compilation,
			context: KnownStatsFactoryContext,
			_options,
			factory
		) => {
			const { type } = context;
			const namedChunkGroups = context._inner.getNamedChunkGroups();
			// object.namedChunkGroups = factory.create(
			// 	`${type}.namedChunkGroups`,
			// 	namedChunkGroups,
			// 	context
			// );
			object.namedChunkGroups = namedChunkGroups.reduce<
				Record<string, KnownStatsChunkGroup>
			>((acc, cur) => {
				acc[cur.name] = cur;
				return acc;
			}, {});
		},
		errors: (
			object,
			compilation,
			context: KnownStatsFactoryContext,
			options,
			factory
		) => {
			const { type, cachedGetErrors } = context;
			object.errors = cachedGetErrors!(compilation);
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
			factory
		) => {
			const { type, cachedGetWarnings } = context;
			object.warnings = cachedGetWarnings!(compilation);
		},
		warningsCount: (object, compilation, context: KnownStatsFactoryContext) => {
			const { type, cachedGetWarnings } = context;
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
	}
};

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
					// for (const key of Object.keys(MERGER)) {
					// 	const merger = MERGER[key];
					// 	stats.hooks.merge.for(key).tap("DefaultStatsFactoryPlugin", merger);
					// }

					if (options.children) {
						if (Array.isArray(options.children)) {
							stats.hooks.getItemFactory
								.for("compilation.children[].compilation")
								// @ts-expect-error
								.tap("DefaultStatsFactoryPlugin", (_comp, { _index: idx }) => {
									if (idx < options.children.length) {
										return compilation.createStatsFactory(
											compilation.createStatsOptions(
												options.children[idx],
												context
											)
										);
									}
								});
						} else if (options.children !== true) {
							const childFactory = compilation.createStatsFactory(
								compilation.createStatsOptions(options.children, context)
							);
							stats.hooks.getItemFactory
								.for("compilation.children[].compilation")
								// @ts-expect-error
								.tap("DefaultStatsFactoryPlugin", () => {
									return childFactory;
								});
						}
					}
				}
			);
		});
	}
}
