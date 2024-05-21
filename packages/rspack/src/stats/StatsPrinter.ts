/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Compilation.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import type { AsArray, Hook } from "tapable";
import { HookMap, SyncBailHook, SyncWaterfallHook } from "tapable";

import type {
	StatsAsset,
	StatsChunk,
	StatsChunkGroup,
	StatsCompilation,
	StatsModule,
	StatsModuleReason
} from "./statsFactoryUtils";

type PrintedElement = {
	element: string;
	content: string;
};

type KnownStatsPrinterContext = {
	type?: string;
	compilation?: StatsCompilation;
	chunkGroup?: StatsChunkGroup;
	asset?: StatsAsset;
	module?: StatsModule;
	chunk?: StatsChunk;
	moduleReason?: StatsModuleReason;
	bold?: (str: string) => string;
	yellow?: (str: string) => string;
	red?: (str: string) => string;
	green?: (str: string) => string;
	magenta?: (str: string) => string;
	cyan?: (str: string) => string;
	formatFilename?: (file: string, oversize?: boolean) => string;
	formatModuleId?: (id: string) => string;
	formatChunkId?:
		| ((id: string, direction?: "parent" | "child" | "sibling") => string)
		| undefined;
	formatSize?: (size: number) => string;
	formatDateTime?: (dateTime: number) => string;
	formatFlag?: (flag: string) => string;
	formatTime?: (time: number, boldQuantity?: boolean) => string;
	chunkGroupKind?: string;
};

export type StatsPrinterContext = KnownStatsPrinterContext &
	Record<string, any>;

export class StatsPrinter {
	private _levelHookCache: Map<
		HookMap<Hook<any, any>>,
		Map<string, Hook<any, any>[]>
	>;
	private _inPrint: boolean;

	hooks: Readonly<{
		sortElements: HookMap<
			SyncBailHook<[string[], StatsPrinterContext], true | void>
		>;
		printElements: HookMap<
			SyncBailHook<[PrintedElement[], StatsPrinterContext], string>
		>;
		sortItems: HookMap<SyncBailHook<[any[], StatsPrinterContext], true>>;
		getItemName: HookMap<SyncBailHook<[any, StatsPrinterContext], string>>;
		printItems: HookMap<SyncBailHook<[string[], StatsPrinterContext], string>>;
		print: HookMap<SyncBailHook<[{}, StatsPrinterContext], string>>;
		result: HookMap<SyncWaterfallHook<[string, StatsPrinterContext]>>;
	}>;

	constructor() {
		this.hooks = Object.freeze({
			sortElements: new HookMap(
				() =>
					new SyncBailHook<[string[], StatsPrinterContext], true | void>([
						"elements",
						"context"
					])
			),
			printElements: new HookMap(
				() =>
					new SyncBailHook<[PrintedElement[], StatsPrinterContext], string>([
						"printedElements",
						"context"
					])
			),
			sortItems: new HookMap(
				() =>
					new SyncBailHook<[any[], StatsPrinterContext], true>([
						"items",
						"context"
					])
			),
			getItemName: new HookMap(
				() =>
					new SyncBailHook<[any, StatsPrinterContext], string>([
						"item",
						"context"
					])
			),
			printItems: new HookMap(
				() =>
					new SyncBailHook<[string[], StatsPrinterContext], string>([
						"printedItems",
						"context"
					])
			),
			print: new HookMap(
				() =>
					new SyncBailHook<[{}, StatsPrinterContext], string>([
						"object",
						"context"
					])
			),
			result: new HookMap(
				() =>
					new SyncWaterfallHook<[string, StatsPrinterContext]>([
						"result",
						"context"
					])
			)
		});
		this._levelHookCache = new Map();
		this._inPrint = false;
	}

	/**
	 * get all level hooks
	 */
	private _getAllLevelHooks<T extends Hook<any, any>>(
		hookMap: HookMap<T>,
		type: string
	): T[] {
		// @ts-expect-error
		let cache: Map<string, T[]> | undefined = this._levelHookCache.get(hookMap);
		if (cache === undefined) {
			cache = new Map();
			// @ts-expect-error
			this._levelHookCache.set(hookMap, cache);
		}
		const cacheEntry = cache.get(type);
		if (cacheEntry !== undefined) {
			return cacheEntry;
		}
		const hooks: T[] = [];
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

	private _forEachLevel<T, R>(
		hookMap: HookMap<SyncBailHook<T, R>>,
		type: string,
		fn: (hook: SyncBailHook<T, R>) => R
	): R | undefined {
		for (const hook of this._getAllLevelHooks(hookMap, type)) {
			const result = fn(hook);
			if (result !== undefined) return result;
		}
	}

	private _forEachLevelWaterfall<T>(
		hookMap: HookMap<SyncWaterfallHook<T>>,
		type: string,
		data: AsArray<T>[0],
		fn: (hook: SyncWaterfallHook<T>, data: AsArray<T>[0]) => AsArray<T>[0]
	): AsArray<T>[0] {
		for (const hook of this._getAllLevelHooks(hookMap, type)) {
			data = fn(hook, data);
		}
		return data;
	}

	print(
		type: string,
		object: {
			[key: string]: any;
		},
		baseContext?: {
			[key: string]: any;
		}
	): string {
		if (this._inPrint) {
			return this._print(type, object, baseContext);
		} else {
			try {
				this._inPrint = true;
				return this._print(type, object, baseContext);
			} finally {
				this._levelHookCache.clear();
				this._inPrint = false;
			}
		}
	}

	private _print(
		type: string,
		object: {
			[key: string]: any;
		},
		baseContext: Object | undefined
	): string {
		const context = {
			...baseContext,
			type,
			[type]: object
		};

		let printResult = this._forEachLevel(this.hooks.print, type, hook =>
			hook.call(object, context)
		);
		if (printResult === undefined) {
			if (Array.isArray(object)) {
				const sortedItems = object.slice();
				this._forEachLevel(this.hooks.sortItems, type, h =>
					h.call(sortedItems, context)
				);
				const printedItems = sortedItems.map((item, i) => {
					const itemContext: {
						[key: string]: any;
						_index: number;
						type: string;
					} = {
						...context,
						_index: i
					};
					const itemName = this._forEachLevel(
						this.hooks.getItemName,
						`${type}[]`,
						h => h.call(item, itemContext)
					);
					if (itemName) itemContext[itemName] = item;
					return this.print(
						itemName ? `${type}[].${itemName}` : `${type}[]`,
						item,
						itemContext
					);
				});
				printResult = this._forEachLevel(this.hooks.printItems, type, h =>
					h.call(printedItems, context)
				);
				if (printResult === undefined) {
					const result = printedItems.filter(Boolean);
					if (result.length > 0) printResult = result.join("\n");
				}
			} else if (object !== null && typeof object === "object") {
				const elements = Object.keys(object).filter(
					key => object[key] !== undefined
				);
				this._forEachLevel(this.hooks.sortElements, type, h =>
					h.call(elements, context)
				);
				const printedElements = elements.map(element => {
					const content = this.print(`${type}.${element}`, object[element], {
						...context,
						_parent: object,
						_element: element,
						[element]: object[element]
					});
					return { element, content };
				});
				printResult = this._forEachLevel(this.hooks.printElements, type, h =>
					h.call(printedElements, context)
				);
				if (printResult === undefined) {
					const result = printedElements.map(e => e.content).filter(Boolean);
					if (result.length > 0) printResult = result.join("\n");
				}
			}
		}

		return this._forEachLevelWaterfall(
			this.hooks.result,
			type,
			printResult!,
			(h, r) => h.call(r, context)
		);
	}
}
