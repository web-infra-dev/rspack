import * as binding from "@rspack/binding";

import type { Compilation, NormalizedStatsOptions } from "../Compilation";
import {
	type Comparator,
	compareIds,
	compareSelect
} from "../util/comparators";
import type { StatsFactory, StatsFactoryContext } from "./StatsFactory";

export type KnownStatsChunkGroup = binding.JsStatsChunkGroup;

export type KnownStatsChunk = Omit<binding.JsStatsChunk, "sizes"> & {
	sizes: Record<string, number>;
};

export type StatsChunkGroup = binding.JsStatsChunkGroup & Record<string, any>;

export type KnownStatsAsset = binding.JsStatsAsset;

export type StatsAsset = KnownStatsAsset & Record<string, any>;

export type StatsChunk = KnownStatsChunk & Record<string, any>;

export type KnownStatsModule = Omit<
	binding.JsStatsModule,
	"usedExports" | "providedExports" | "optimizationBailout" | "sizes"
> & {
	profile?: StatsProfile;
	usedExports?: null | string[] | boolean;
	providedExports?: null | string[];
	optimizationBailout?: null | string[];
	sizes: Record<string, number>;
};

export type StatsProfile = KnownStatsProfile & Record<string, any>;

export type KnownStatsProfile = {
	total: number;
	resolving: number;
	integration: number;
	building: number;
};

export type StatsModule = KnownStatsModule & Record<string, any>;

export type StatsModuleIssuer = binding.JsStatsModuleIssuer &
	Record<string, any>;

export type StatsError = binding.JsStatsError & Record<string, any>;

export type StatsWarnings = binding.JsStatsWarning & Record<string, any>;

export type StatsModuleReason = binding.JsStatsModuleReason &
	Record<string, any>;

export type KnownStatsCompilation = {
	/**
	 * webpack version.
	 * this is a hack to be compatible with plugin which detect webpack's version
	 */
	version?: string;
	/** rspack version */
	rspackVersion?: string;
	name?: string;
	hash?: string;
	time?: number;
	builtAt?: number;
	publicPath?: string;
	outputPath?: string;
	assets?: StatsAsset[];
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
	logging?: Record<string, StatsLogging>;

	// TODO: not aligned with webpack
	// env?: any;
	// needAdditionalPass?: boolean;
	// filteredAssets?: number;
};

export type StatsCompilation = KnownStatsCompilation & Record<string, any>;

export type StatsLogging = KnownStatsLogging & Record<string, any>;

export type KnownStatsLogging = {
	entries: StatsLoggingEntry[];
	filteredEntries: number;
	debug: boolean;
};

export type StatsLoggingEntry = KnownStatsLoggingEntry & Record<string, any>;

export type KnownStatsLoggingEntry = {
	type: string;
	message: string;
	trace?: string[] | undefined;
	children?: StatsLoggingEntry[] | undefined;
	args?: any[] | undefined;
	time?: number | undefined;
};

export type KnownStatsChunkOrigin = {
	module?: string | undefined;
	moduleIdentifier?: string | undefined;
	moduleName?: string | undefined;
	loc?: string | undefined;
	request?: string | undefined;
	moduleId?: (string | number) | undefined;
};

type ExtractorsByOption<T, O> = {
	[x: string]: (
		object: O,
		data: T,
		context: StatsFactoryContext,
		options: any,
		factory: StatsFactory
	) => void;
};

type PreprocessedAsset = StatsAsset & {
	type: string;
	related: PreprocessedAsset[];
};

export type SimpleExtractors = {
	compilation: ExtractorsByOption<Compilation, StatsCompilation>;
	asset$visible: ExtractorsByOption<PreprocessedAsset, StatsAsset>;
	asset: ExtractorsByOption<PreprocessedAsset, StatsAsset>;
	// chunkGroup: ExtractorsByOption<
	// 	{
	// 		name: string;
	// 		chunkGroup: binding.JsChunkGroup;
	// 	},
	// 	StatsChunkGroup
	// >;
	module: ExtractorsByOption<binding.JsStatsModule, StatsModule>;
	module$visible: ExtractorsByOption<binding.JsStatsModule, StatsModule>;
	moduleIssuer: ExtractorsByOption<
		binding.JsStatsModuleIssuer,
		StatsModuleIssuer
	>;
	profile: ExtractorsByOption<binding.JsStatsModuleProfile, StatsProfile>;
	moduleReason: ExtractorsByOption<
		binding.JsStatsModuleReason,
		StatsModuleReason
	>;
	chunk: ExtractorsByOption<binding.JsStatsChunk, KnownStatsChunk>;
	// chunkOrigin: ExtractorsByOption<OriginRecord, StatsChunkOrigin>;
	// error: ExtractorsByOption<binding.JsStatsError, StatsError>;
	// warning: ExtractorsByOption<binding.JsStatsWarning, StatsError>;
};

export const uniqueArray = <T, I>(
	items: Iterable<T>,
	selector: (arg: T) => Iterable<I>
): I[] => {
	const set: Set<I> = new Set();
	for (const item of items) {
		for (const i of selector(item)) {
			set.add(i);
		}
	}
	return Array.from(set);
};

export const uniqueOrderedArray = <T, I>(
	items: Iterable<T>,
	selector: (arg: T) => Iterable<I>,
	comparator: Comparator
): I[] => {
	return uniqueArray(items, selector).sort(comparator);
};

export const iterateConfig = (
	config: Record<string, Record<string, Function>>,
	options: NormalizedStatsOptions,
	fn: (a1: string, a2: Function) => void
) => {
	for (const hookFor of Object.keys(config)) {
		const subConfig = config[hookFor];
		for (const option of Object.keys(subConfig)) {
			if (option !== "_") {
				if (option.startsWith("!")) {
					if (options[option.slice(1)]) continue;
				} else {
					const value = options[option];
					if (
						value === false ||
						value === undefined ||
						(Array.isArray(value) && value.length === 0)
					)
						continue;
				}
			}
			fn(hookFor, subConfig[option]);
		}
	}
};

type Child = {
	children?: ItemChildren;
	filteredChildren?: number;
};

type ItemChildren = Child[];

export const getTotalItems = (children: ItemChildren) => {
	let count = 0;
	for (const child of children) {
		if (!child.children && !child.filteredChildren) {
			count++;
		} else {
			if (child.children) count += getTotalItems(child.children);
			if (child.filteredChildren) count += child.filteredChildren;
		}
	}
	return count;
};

export const collapse = (children: ItemChildren) => {
	// After collapse each child must take exactly one line
	const newChildren = [];
	for (const child of children) {
		if (child.children) {
			let filteredChildren = child.filteredChildren || 0;
			filteredChildren += getTotalItems(child.children);
			newChildren.push({
				...child,
				children: undefined,
				filteredChildren
			});
		} else {
			newChildren.push(child);
		}
	}
	return newChildren;
};

const getTotalSize = (children: ItemChildren) => {
	let size = 0;
	for (const child of children) {
		size += getItemSize(child);
	}
	return size;
};

const getItemSize = (item: Child) => {
	// Each item takes 1 line
	// + the size of the children
	// + 1 extra line when it has children and filteredChildren
	return !item.children
		? 1
		: item.filteredChildren
			? 2 + getTotalSize(item.children)
			: 1 + getTotalSize(item.children);
};

export const spaceLimited = (
	itemsAndGroups: ItemChildren,
	max: number,
	filteredChildrenLineReserved = false
): {
	children: any;
	filteredChildren: any;
} => {
	if (max < 1) {
		return {
			children: undefined,
			filteredChildren: getTotalItems(itemsAndGroups)
		};
	}
	let children: any[] | undefined = undefined;
	let filteredChildren: number | undefined = undefined;
	// This are the groups, which take 1+ lines each
	const groups = [];
	// The sizes of the groups are stored in groupSizes
	const groupSizes = [];
	// This are the items, which take 1 line each
	const items = [];
	// The total of group sizes
	let groupsSize = 0;

	for (const itemOrGroup of itemsAndGroups) {
		// is item
		if (!itemOrGroup.children && !itemOrGroup.filteredChildren) {
			items.push(itemOrGroup);
		} else {
			groups.push(itemOrGroup);
			const size = getItemSize(itemOrGroup);
			groupSizes.push(size);
			groupsSize += size;
		}
	}

	if (groupsSize + items.length <= max) {
		// The total size in the current state fits into the max
		// keep all
		children = groups.length > 0 ? groups.concat(items) : items;
	} else if (groups.length === 0) {
		// slice items to max
		// inner space marks that lines for filteredChildren already reserved
		const limit = max - (filteredChildrenLineReserved ? 0 : 1);
		filteredChildren = items.length - limit;
		items.length = limit;
		children = items;
	} else {
		// limit is the size when all groups are collapsed
		const limit =
			groups.length +
			(filteredChildrenLineReserved || items.length === 0 ? 0 : 1);
		if (limit < max) {
			// calculate how much we are over the size limit
			// this allows to approach the limit faster
			let oversize;
			// If each group would take 1 line the total would be below the maximum
			// collapse some groups, keep items
			while (
				(oversize =
					groupsSize +
					items.length +
					(filteredChildren && !filteredChildrenLineReserved ? 1 : 0) -
					max) > 0
			) {
				// Find the maximum group and process only this one
				const maxGroupSize = Math.max(...groupSizes);
				if (maxGroupSize < items.length) {
					filteredChildren = items.length;
					items.length = 0;
					continue;
				}
				for (let i = 0; i < groups.length; i++) {
					if (groupSizes[i] === maxGroupSize) {
						// @ts-expect-error
						const group = groups[i];
						// run this algorithm recursively and limit the size of the children to
						// current size - oversize / number of groups
						// So it should always end up being smaller
						const headerSize = group.filteredChildren ? 2 : 1;
						const limited = spaceLimited(
							group.children,
							maxGroupSize -
								// we should use ceil to always feet in max
								Math.ceil(oversize / groups.length) -
								// we substitute size of group head
								headerSize,
							headerSize === 2
						);
						groups[i] = {
							...group,
							children: limited.children,
							filteredChildren: limited.filteredChildren
								? (group.filteredChildren || 0) + limited.filteredChildren
								: group.filteredChildren
						};
						const newSize = getItemSize(groups[i]);
						groupsSize -= maxGroupSize - newSize;
						groupSizes[i] = newSize;
						break;
					}
				}
			}
			children = groups.concat(items);
		} else if (limit === max) {
			// If we have only enough space to show one line per group and one line for the filtered items
			// collapse all groups and items
			children = collapse(groups);
			filteredChildren = items.length;
		} else {
			// If we have no space
			// collapse complete group
			filteredChildren = getTotalItems(itemsAndGroups);
		}
	}

	return {
		children,
		filteredChildren
	};
};

export const countWithChildren = (
	compilation: Compilation,
	getItems: (compilation: Compilation, key: string) => any[]
): number => {
	let count = getItems(compilation, "").length;
	for (const child of compilation.children) {
		count += countWithChildren(child, (c, type) =>
			getItems(c, `.children[].compilation${type}`)
		);
	}
	return count;
};

// remove a prefixed "!" that can be specified to reverse sort order
const normalizeFieldKey = (field: string) => {
	if (field[0] === "!") {
		return field.slice(1);
	}
	return field;
};

// if a field is prefixed by a "!" reverse sort order
const sortOrderRegular = (field: string) => {
	if (field[0] === "!") {
		return false;
	}
	return true;
};

export const sortByField = (
	field: string
): ((a1: Object, a2: Object) => number) => {
	if (!field) {
		const noSort = (a: any, b: any) => 0;
		return noSort;
	}

	const fieldKey = normalizeFieldKey(field);

	let sortFn = compareSelect(
		(m: Record<string, any>) => m[fieldKey],
		// @ts-expect-error
		compareIds
	);

	// if a field is prefixed with a "!" the sort is reversed!
	const sortIsRegular = sortOrderRegular(field);

	if (!sortIsRegular) {
		const oldSortFn = sortFn;
		sortFn = (a, b) => oldSortFn(b, a);
	}

	return sortFn;
};

export const assetGroup = (children: StatsAsset[]) => {
	let size = 0;
	for (const asset of children) {
		size += asset.size;
	}
	return {
		size
	};
};

export const moduleGroup = (children: KnownStatsModule[]) => {
	let size = 0;
	const sizes: Record<string, number> = {};
	for (const module of children) {
		size += module.size;
		for (const key of Object.keys(module.sizes)) {
			sizes[key] = (sizes[key] || 0) + module.sizes[key];
		}
	}
	return {
		size,
		sizes
	};
};

export const mergeToObject = (
	items: {
		[key: string]: any;
		name: string;
	}[]
): Object => {
	const obj = Object.create(null);
	for (const item of items) {
		obj[item.name] = item;
	}

	return obj;
};

export function resolveStatsMillisecond(s: binding.JsStatsMillisecond) {
	return s.secs * 1000 + s.subsecMillis;
}
