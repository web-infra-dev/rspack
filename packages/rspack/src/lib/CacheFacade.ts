/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/CacheFacade.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import asyncLib from "neo-async";
import { mergeEtags } from "./cache/mergeEtags.js";
import { getter as getLazyHashedEtag } from "./cache/getLazyHashedEtag.js";

/**
 * @template T
 * @template Z
 * @param {T[]} array array
 * @param {(arg0: T, arg1: (err?: null|Error, result?: null|Z) => void , arg2: number) => void} iterator iterator
 * @param {(err?: null|Error, result?: null|Z, i?: number) => void} callback callback after all items are iterated
 * @returns {void}
 */
function forEachBail<T, Z>(
	array: T[],
	iterator: (
		arg0: T,
		arg1: (err?: null | Error, result?: null | Z) => void,
		arg2: number
	) => void,
	callback: (err?: null | Error, result?: null | Z, i?: number) => void
): void {
	if (array.length === 0) {
		callback();
		return;
	}

	let i = 0;
	const next = () => {
		/** @type {boolean|undefined} */
		let loop: boolean | undefined = undefined;
		iterator(
			array[i++],
			(err, result) => {
				if (err || result !== undefined || i >= array.length) {
					return callback(err, result, i);
				}
				if (loop === false) while (next());
				loop = true;
			},
			i
		);
		if (!loop) loop = false;
		return loop;
	};
	while (next());
}

/** @typedef {import("./Cache")} Cache */
/** @typedef {import("./Cache").Etag} Etag */
/** @typedef {import("./WebpackError")} WebpackError */
/** @typedef {import("./cache/getLazyHashedEtag").HashableObject} HashableObject */
// /** @typedef {typeof import("./util/Hash")} HashConstructor */
/** @typedef {any} HashConstructor */

/**
 * @template T
 * @callback CallbackCache
 * @param {(WebpackError | null)=} err
 * @param {T=} result
 * @returns {void}
 */

/**
 * @template T
 * @callback CallbackNormalErrorCache
 * @param {(Error | null)=} err
 * @param {T=} result
 * @returns {void}
 */

class MultiItemCache {
	/**
	 * @param {ItemCacheFacade[]} items item caches
	 */
	constructor(items: ItemCacheFacade[]) {
		this._items = items;
		if (items.length === 1) return /** @type {any} */ items[0];
	}

	/**
	 * @template T
	 * @param {CallbackCache<T>} callback signals when the value is retrieved
	 * @returns {void}
	 */
	get<T>(callback: CallbackCache<T>): void {
		// @ts-expect-error
		forEachBail(this._items, (item, callback) => item.get(callback), callback);
	}

	/**
	 * @template T
	 * @returns {Promise<T>} promise with the data
	 */
	getPromise<T>(): Promise<T> {
		// @ts-expect-error
		const next = i => {
			// @ts-ignore if your typescript version >= 5.5, this line will throw an error
			return this._items[i].getPromise().then(result => {
				if (result !== undefined) return result;
				if (++i < this._items.length) return next(i);
			});
		};
		return next(0);
	}

	/**
	 * @template T
	 * @param {T} data the value to store
	 * @param {CallbackCache<void>} callback signals when the value is stored
	 * @returns {void}
	 */
	store<T>(data: T, callback: CallbackCache<void>): void {
		asyncLib.each(
			this._items,
			(item, callback) => item.store(data, callback),
			callback
		);
	}

	/**
	 * @template T
	 * @param {T} data the value to store
	 * @returns {Promise<void>} promise signals when the value is stored
	 */
	storePromise<T>(data: T): Promise<void> {
		return Promise.all(this._items.map(item => item.storePromise(data))).then(
			() => {}
		);
	}
}

class ItemCacheFacade {
	/**
	 * @param {Cache} cache the root cache
	 * @param {string} name the child cache item name
	 * @param {Etag | null} etag the etag
	 */
	constructor(cache: Cache, name: string, etag: Etag | null) {
		this._cache = cache;
		this._name = name;
		this._etag = etag;
	}

	/**
	 * @template T
	 * @param {CallbackCache<T>} callback signals when the value is retrieved
	 * @returns {void}
	 */
	get<T>(callback: CallbackCache<T>): void {
		this._cache.get(this._name, this._etag, callback);
	}

	/**
	 * @template T
	 * @returns {Promise<T>} promise with the data
	 */
	getPromise<T>(): Promise<T> {
		return new Promise((resolve, reject) => {
			this._cache.get(this._name, this._etag, (err, data) => {
				if (err) {
					reject(err);
				} else {
					resolve(data);
				}
			});
		});
	}

	/**
	 * @template T
	 * @param {T} data the value to store
	 * @param {CallbackCache<void>} callback signals when the value is stored
	 * @returns {void}
	 */
	store<T>(data: T, callback: CallbackCache<void>): void {
		this._cache.store(this._name, this._etag, data, callback);
	}

	/**
	 * @template T
	 * @param {T} data the value to store
	 * @returns {Promise<void>} promise signals when the value is stored
	 */
	storePromise<T>(data: T): Promise<void> {
		return new Promise((resolve, reject) => {
			this._cache.store(this._name, this._etag, data, err => {
				if (err) {
					reject(err);
				} else {
					resolve();
				}
			});
		});
	}

	/**
	 * @template T
	 * @param {function(CallbackNormalErrorCache<T>): void} computer function to compute the value if not cached
	 * @param {CallbackNormalErrorCache<T>} callback signals when the value is retrieved
	 * @returns {void}
	 */
	provide<T>(
		computer: (arg0: CallbackNormalErrorCache<T>) => void,
		callback: CallbackNormalErrorCache<T>
	): void {
		this.get((err, cacheEntry) => {
			if (err) return callback(err);
			if (cacheEntry !== undefined) return cacheEntry;
			computer((err, result) => {
				if (err) return callback(err);
				this.store(result, err => {
					if (err) return callback(err);
					callback(null, result);
				});
			});
		});
	}

	/**
	 * @template T
	 * @param {function(): Promise<T> | T} computer function to compute the value if not cached
	 * @returns {Promise<T>} promise with the data
	 */
	async providePromise<T>(computer: () => Promise<T> | T): Promise<T> {
		const cacheEntry = await this.getPromise();
		if (cacheEntry !== undefined) return cacheEntry;
		const result = await computer();
		await this.storePromise(result);
		return result;
	}
}

class CacheFacade {
	/**
	 * @param {Cache} cache the root cache
	 * @param {string} name the child cache name
	 * @param {string | HashConstructor} hashFunction the hash function to use
	 */
	constructor(
		cache: Cache,
		name: string,
		hashFunction: string | HashConstructor
	) {
		this._cache = cache;
		this._name = name;
		this._hashFunction = hashFunction;
	}

	/**
	 * @param {string} name the child cache name#
	 * @returns {CacheFacade} child cache
	 */
	getChildCache(name: string): CacheFacade {
		return new CacheFacade(
			this._cache,
			`${this._name}|${name}`,
			this._hashFunction
		);
	}

	/**
	 * @param {string} identifier the cache identifier
	 * @param {Etag | null} etag the etag
	 * @returns {ItemCacheFacade} item cache
	 */
	getItemCache(identifier: string, etag: Etag | null): ItemCacheFacade {
		return new ItemCacheFacade(
			this._cache,
			`${this._name}|${identifier}`,
			etag
		);
	}

	/**
	 * @param {HashableObject} obj an hashable object
	 * @returns {Etag} an etag that is lazy hashed
	 */
	getLazyHashedEtag(obj: HashableObject): Etag {
		return getLazyHashedEtag(obj, this._hashFunction);
	}

	/**
	 * @param {Etag} a an etag
	 * @param {Etag} b another etag
	 * @returns {Etag} an etag that represents both
	 */
	mergeEtags(a: Etag, b: Etag): Etag {
		return mergeEtags(a, b);
	}

	/**
	 * @template T
	 * @param {string} identifier the cache identifier
	 * @param {Etag | null} etag the etag
	 * @param {CallbackCache<T>} callback signals when the value is retrieved
	 * @returns {void}
	 */
	get<T>(
		identifier: string,
		etag: Etag | null,
		callback: CallbackCache<T>
	): void {
		this._cache.get(`${this._name}|${identifier}`, etag, callback);
	}

	/**
	 * @template T
	 * @param {string} identifier the cache identifier
	 * @param {Etag | null} etag the etag
	 * @returns {Promise<T>} promise with the data
	 */
	getPromise<T>(identifier: string, etag: Etag | null): Promise<T> {
		return new Promise((resolve, reject) => {
			this._cache.get(`${this._name}|${identifier}`, etag, (err, data) => {
				if (err) {
					reject(err);
				} else {
					resolve(data);
				}
			});
		});
	}

	/**
	 * @template T
	 * @param {string} identifier the cache identifier
	 * @param {Etag | null} etag the etag
	 * @param {T} data the value to store
	 * @param {CallbackCache<void>} callback signals when the value is stored
	 * @returns {void}
	 */
	store<T>(
		identifier: string,
		etag: Etag | null,
		data: T,
		callback: CallbackCache<void>
	): void {
		this._cache.store(`${this._name}|${identifier}`, etag, data, callback);
	}

	/**
	 * @template T
	 * @param {string} identifier the cache identifier
	 * @param {Etag | null} etag the etag
	 * @param {T} data the value to store
	 * @returns {Promise<void>} promise signals when the value is stored
	 */
	storePromise<T>(
		identifier: string,
		etag: Etag | null,
		data: T
	): Promise<void> {
		return new Promise((resolve, reject) => {
			this._cache.store(`${this._name}|${identifier}`, etag, data, err => {
				if (err) {
					reject(err);
				} else {
					resolve();
				}
			});
		});
	}

	/**
	 * @template T
	 * @param {string} identifier the cache identifier
	 * @param {Etag | null} etag the etag
	 * @param {function(CallbackNormalErrorCache<T>): void} computer function to compute the value if not cached
	 * @param {CallbackNormalErrorCache<T>} callback signals when the value is retrieved
	 * @returns {void}
	 */
	provide<T>(
		identifier: string,
		etag: Etag | null,
		computer: (arg0: CallbackNormalErrorCache<T>) => void,
		callback: CallbackNormalErrorCache<T>
	): void {
		this.get(identifier, etag, (err, cacheEntry) => {
			if (err) return callback(err);
			if (cacheEntry !== undefined) return cacheEntry;
			computer((err, result) => {
				if (err) return callback(err);
				this.store(identifier, etag, result, err => {
					if (err) return callback(err);
					callback(null, result);
				});
			});
		});
	}

	/**
	 * @template T
	 * @param {string} identifier the cache identifier
	 * @param {Etag | null} etag the etag
	 * @param {function(): Promise<T> | T} computer function to compute the value if not cached
	 * @returns {Promise<T>} promise with the data
	 */
	async providePromise<T>(
		identifier: string,
		etag: Etag | null,
		computer: () => Promise<T> | T
	): Promise<T> {
		const cacheEntry = await this.getPromise(identifier, etag);
		if (cacheEntry !== undefined) return cacheEntry;
		const result = await computer();
		await this.storePromise(identifier, etag, result);
		return result;
	}
}

module.exports = CacheFacade;
module.exports.ItemCacheFacade = ItemCacheFacade;
module.exports.MultiItemCache = MultiItemCache;
