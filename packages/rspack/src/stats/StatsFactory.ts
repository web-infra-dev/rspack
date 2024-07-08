/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/stats
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import { JsStats, JsStatsError, JsStatsWarning } from "@rspack/binding";
import { HookMap, SyncBailHook, SyncWaterfallHook } from "@rspack/lite-tapable";

import type { Compilation } from "../Compilation";
import { Comparator, concatComparators } from "../util/comparators";
import { GroupConfig, smartGrouping } from "../util/smartGrouping";

export type KnownStatsFactoryContext = {
	type: string;
	makePathsRelative?: ((arg0: string) => string) | undefined;
	compilation?: Compilation | undefined;
	// rootModules?: Set<Module> | undefined;
	// compilationFileToChunks?: Map<string, Chunk[]> | undefined;
	// compilationAuxiliaryFileToChunks?: Map<string, Chunk[]> | undefined;
	// runtime?: RuntimeSpec | undefined;
	cachedGetErrors?: ((arg0: Compilation) => JsStatsError[]) | undefined;
	cachedGetWarnings?: ((arg0: Compilation) => JsStatsWarning[]) | undefined;
	getInner: (compilation: Compilation) => JsStats;
};

export type StatsFactoryContext = KnownStatsFactoryContext &
	Record<string, any>;

type Hooks = Readonly<{
	extract: HookMap<SyncBailHook<[Object, any, StatsFactoryContext], undefined>>;
	filter: HookMap<
		SyncBailHook<[any, StatsFactoryContext, number, number], undefined>
	>;
	filterSorted: HookMap<
		SyncBailHook<[any, StatsFactoryContext, number, number], undefined>
	>;
	groupResults: HookMap<
		SyncBailHook<[GroupConfig[], StatsFactoryContext], undefined>
	>;
	filterResults: HookMap<
		SyncBailHook<[any, StatsFactoryContext, number, number], undefined>
	>;
	sort: HookMap<
		SyncBailHook<
			[((arg1: any, arg2: any) => number)[], StatsFactoryContext],
			undefined
		>
	>;
	sortResults: HookMap<
		SyncBailHook<
			[((arg1: any, arg2: any) => number)[], StatsFactoryContext],
			undefined
		>
	>;
	result: HookMap<SyncWaterfallHook<[any[], StatsFactoryContext]>>;
	merge: HookMap<SyncBailHook<[any[], StatsFactoryContext], undefined>>;
	getItemName: HookMap<
		SyncBailHook<[any, StatsFactoryContext], string | undefined>
	>;
	getItemFactory: HookMap<SyncBailHook<[any, StatsFactoryContext], undefined>>;
}>;

type CacheHookMap = Map<
	string,
	SyncBailHook<[any[], StatsFactoryContext], any>[]
>;

type CallFn = (...args: any[]) => any;

type CacheKey = keyof Hooks;

type Cache = Record<CacheKey, CacheHookMap>;

export class StatsFactory {
	hooks: Hooks;

	private _caches: Cache;

	private _inCreate: boolean;

	constructor() {
		this.hooks = Object.freeze({
			extract: new HookMap(
				() =>
					new SyncBailHook<[Object, any, StatsFactoryContext], undefined>([
						"object",
						"data",
						"context"
					])
			),
			filter: new HookMap(
				() =>
					new SyncBailHook<
						[any, StatsFactoryContext, number, number],
						undefined
					>(["item", "context", "index", "unfilteredIndex"])
			),
			sort: new HookMap(
				() =>
					new SyncBailHook<
						[((arg1: any, arg2: any) => number)[], StatsFactoryContext],
						undefined
					>(["comparators", "context"])
			),
			filterSorted: new HookMap(
				() =>
					new SyncBailHook<
						[any, StatsFactoryContext, number, number],
						undefined
					>(["item", "context", "index", "unfilteredIndex"])
			),
			groupResults: new HookMap(
				() =>
					new SyncBailHook<[GroupConfig[], StatsFactoryContext], undefined>([
						"groupConfigs",
						"context"
					])
			),
			sortResults: new HookMap(
				() =>
					new SyncBailHook<
						[((arg1: any, arg2: any) => number)[], StatsFactoryContext],
						undefined
					>(["comparators", "context"])
			),
			filterResults: new HookMap(
				() =>
					new SyncBailHook<
						[any, StatsFactoryContext, number, number],
						undefined
					>(["item", "context", "index", "unfilteredIndex"])
			),
			merge: new HookMap(
				() =>
					new SyncBailHook<[any[], StatsFactoryContext], undefined>([
						"items",
						"context"
					])
			),
			result: new HookMap(
				() =>
					new SyncWaterfallHook<[any[], StatsFactoryContext]>([
						"result",
						"context"
					])
			),
			getItemName: new HookMap(
				() =>
					new SyncBailHook<[any, StatsFactoryContext], string | undefined>([
						"item",
						"context"
					])
			),
			getItemFactory: new HookMap(
				() =>
					new SyncBailHook<[any, StatsFactoryContext], undefined>([
						"item",
						"context"
					])
			)
		});

		const hooks = this.hooks;
		this._caches = Object.keys(hooks).reduce((prev, curr) => {
			return {
				...prev,
				[curr]: new Map()
			};
		}, {} as Cache);
		this._inCreate = false;
	}

	_getAllLevelHooks(hookMap: HookMap<any>, cache: CacheHookMap, type: string) {
		const cacheEntry = cache.get(type);
		if (cacheEntry !== undefined) {
			return cacheEntry;
		}
		const hooks = [];
		const typeParts = type.split(".");
		for (let i = 0; i < typeParts.length; i++) {
			const hook = hookMap.get(typeParts.slice(i).join("."));
			if (hook) {
				hooks.push(hook);
			}
		}
		cache.set(type, hooks);
		return hooks;
	}

	_forEachLevel(
		hookMap: HookMap<any>,
		cache: CacheHookMap,
		type: string,
		fn: CallFn
	) {
		for (const hook of this._getAllLevelHooks(hookMap, cache, type)) {
			const result = fn(hook);
			if (result !== undefined) return result;
		}
	}

	_forEachLevelWaterfall(
		hookMap: HookMap<any>,
		cache: CacheHookMap,
		type: string,
		data: any,
		fn: CallFn
	) {
		for (const hook of this._getAllLevelHooks(hookMap, cache, type)) {
			data = fn(hook, data);
		}
		return data;
	}

	_forEachLevelFilter(
		hookMap: HookMap<any>,
		cache: CacheHookMap,
		type: string,
		items: any[],
		fn: CallFn,
		forceClone: boolean
	) {
		const hooks = this._getAllLevelHooks(hookMap, cache, type);
		if (hooks.length === 0) return forceClone ? items.slice() : items;
		let i = 0;
		return items.filter((item, idx) => {
			for (const hook of hooks) {
				const r = fn(hook, item, idx, i);
				if (r !== undefined) {
					if (r) i++;
					return r;
				}
			}
			i++;
			return true;
		});
	}

	create(
		type: string,
		data: any,
		baseContext: Omit<StatsFactoryContext, "type">
	) {
		if (this._inCreate) {
			return this._create(type, data, baseContext);
		} else {
			try {
				this._inCreate = true;
				return this._create(type, data, baseContext);
			} finally {
				for (const key of Object.keys(this._caches) as CacheKey[])
					this._caches[key].clear();
				this._inCreate = false;
			}
		}
	}

	private _create(
		type: string,
		data: any,
		baseContext: Omit<StatsFactoryContext, "type">
	) {
		const context = {
			...baseContext,
			type,
			[type]: data
		};
		if (Array.isArray(data)) {
			// run filter on unsorted items
			const items = this._forEachLevelFilter(
				this.hooks.filter,
				this._caches.filter,
				type,
				data,
				(h, r, idx, i) => h.call(r, context, idx, i),
				true
			);

			// sort items
			const comparators: Comparator[] = [];
			this._forEachLevel(this.hooks.sort, this._caches.sort, type, h =>
				h.call(comparators, context)
			);
			if (comparators.length > 0) {
				items.sort(
					// @ts-expect-error number of arguments is correct
					concatComparators(...comparators)
				);
			}

			// run filter on sorted items
			const items2 = this._forEachLevelFilter(
				this.hooks.filterSorted,
				this._caches.filterSorted,
				type,
				items,
				(h, r, idx, i) => h.call(r, context, idx, i),
				false
			);

			// for each item
			let resultItems = items2.map((item, i) => {
				const itemContext: {
					[key: string]: any;
					_index: number;
					type: string;
				} = {
					...context,
					_index: i
				};

				// run getItemName
				const itemName = this._forEachLevel(
					this.hooks.getItemName,
					this._caches.getItemName,
					`${type}[]`,
					h => h.call(item, itemContext)
				);
				if (itemName) itemContext[itemName] = item;
				const innerType = itemName ? `${type}[].${itemName}` : `${type}[]`;

				// run getItemFactory
				const itemFactory =
					this._forEachLevel(
						this.hooks.getItemFactory,
						this._caches.getItemFactory,
						innerType,
						h => h.call(item, itemContext)
					) || this;

				// run item factory
				return itemFactory.create(innerType, item, itemContext);
			});

			// sort result items
			const comparators2: Comparator[] = [];
			this._forEachLevel(
				this.hooks.sortResults,
				this._caches.sortResults,
				type,
				h => h.call(comparators2, context)
			);
			if (comparators2.length > 0) {
				resultItems.sort(
					// @ts-expect-error number of arguments is correct
					concatComparators(...comparators2)
				);
			}

			// group result items
			const groupConfigs: GroupConfig[] = [];
			this._forEachLevel(
				this.hooks.groupResults,
				this._caches.groupResults,
				type,
				h => h.call(groupConfigs, context)
			);
			if (groupConfigs.length > 0) {
				resultItems = smartGrouping(resultItems, groupConfigs);
			}

			// run filter on sorted result items
			const finalResultItems = this._forEachLevelFilter(
				this.hooks.filterResults,
				this._caches.filterResults,
				type,
				resultItems,
				(h, r, idx, i) => h.call(r, context, idx, i),
				false
			);

			// run merge on mapped items
			let result = this._forEachLevel(
				this.hooks.merge,
				this._caches.merge,
				type,
				h => h.call(finalResultItems, context)
			);
			if (result === undefined) result = finalResultItems;

			// run result on merged items
			return this._forEachLevelWaterfall(
				this.hooks.result,
				this._caches.result,
				type,
				result,
				(h, r) => h.call(r, context)
			);
		} else {
			const object = {};

			// run extract on value
			this._forEachLevel(this.hooks.extract, this._caches.extract, type, h =>
				h.call(object, data, context)
			);

			// run result on extracted object
			return this._forEachLevelWaterfall(
				this.hooks.result,
				this._caches.result,
				type,
				object,
				(h, r) => h.call(r, context)
			);
		}
	}
}
