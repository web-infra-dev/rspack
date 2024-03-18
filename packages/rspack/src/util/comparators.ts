/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/util
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { JsStatsChunk as Chunk } from "@rspack/binding";
import { ChunkGroup } from "../ChunkGroup";

export type Comparator = <T>(arg0: T, arg1: T) => -1 | 0 | 1;

type Selector<A, B> = (input: A) => B;

class TwoKeyWeakMap<K1, K2, T> {
	private _map: WeakMap<any, WeakMap<any, T>>;
	constructor() {
		this._map = new WeakMap();
	}

	get(key1: K1, key2: K2): T | undefined {
		const childMap = this._map.get(key1);
		if (childMap === undefined) {
			return undefined;
		}
		return childMap.get(key2);
	}

	set(key1: K1, key2: K2, value: T) {
		let childMap = this._map.get(key1);
		if (childMap === undefined) {
			childMap = new WeakMap();
			this._map.set(key1, childMap);
		}
		childMap.set(key2, value);
	}
}

const concatComparatorsCache: TwoKeyWeakMap<
	Comparator,
	Comparator,
	Comparator
> = new TwoKeyWeakMap();

export const concatComparators = (
	c1: Comparator,
	c2: Comparator,
	...cRest: Comparator[]
): Comparator => {
	if (cRest.length > 0) {
		const [c3, ...cRest2] = cRest;
		return concatComparators(c1, concatComparators(c2, c3, ...cRest2));
	}

	if (c2 === undefined) {
		return c1;
	}

	const cacheEntry = concatComparatorsCache.get(c1, c2);
	if (cacheEntry !== undefined) return cacheEntry;

	const result = <T>(a: T, b: T) => {
		const res = c1(a, b);
		if (res !== 0) return res;
		return c2(a, b);
	};
	concatComparatorsCache.set(c1, c2, result);
	return result;
};

export const compareIds = (
	a: string | number,
	b: string | number
): -1 | 0 | 1 => {
	if (typeof a !== typeof b) {
		return typeof a < typeof b ? -1 : 1;
	}
	if (a < b) return -1;
	if (a > b) return 1;
	return 0;
};

export const compareChunksById = (a: Chunk, b: Chunk): -1 | 0 | 1 => {
	return compareIds(a.id || "", b.id || "");
};

export const compareChunkGroupsByIndex = (
	a: ChunkGroup,
	b: ChunkGroup
): -1 | 0 | 1 => {
	//@ts-expect-error copy from webpack
	return a.index < b.index ? -1 : 1;
};

const compareSelectCache: TwoKeyWeakMap<
	Selector<any, any>,
	Comparator,
	Comparator
> = new TwoKeyWeakMap();

export const compareSelect = <T, R>(
	getter: Selector<T, R>,
	comparator: Comparator
): Comparator => {
	const cacheEntry = compareSelectCache.get(getter, comparator);
	if (cacheEntry !== undefined) return cacheEntry;

	const result = <A>(a: A, b: A) => {
		const aValue = getter(a as unknown as T);
		const bValue = getter(b as unknown as T);
		if (aValue !== undefined && aValue !== null) {
			if (bValue !== undefined && bValue !== null) {
				return comparator(aValue, bValue);
			}
			return -1;
		} else {
			if (bValue !== undefined && bValue !== null) {
				return 1;
			}
			return 0;
		}
	};
	compareSelectCache.set(getter, comparator, result);
	return result;
};

export const compareNumbers = (a: number, b: number) => {
	if (typeof a !== typeof b) {
		return typeof a < typeof b ? -1 : 1;
	}
	if (a < b) return -1;
	if (a > b) return 1;
	return 0;
};
