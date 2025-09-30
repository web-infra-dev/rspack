/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/cache/getLazyHashedEtag.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { createHash } from "../../util/createHash";
import type Hash from "../../util/hash";

export type HashConstructor = typeof Hash;

export interface HashableObject {
	updateHash(hash: Hash): void;
}

class LazyHashedEtag {
	_obj: HashableObject;
	_hash?: string;
	_hashFunction: string | HashConstructor;

	/**
	 * @param obj object with updateHash method
	 * @param hashFunction the hash function to use
	 */
	constructor(
		obj: HashableObject,
		hashFunction: string | HashConstructor = "xxhash64"
	) {
		this._obj = obj;
		this._hash = undefined;
		this._hashFunction = hashFunction;
	}

	/**
	 * @returns hash of object
	 */
	toString(): string {
		if (this._hash === undefined) {
			const hash = createHash(this._hashFunction);
			this._obj.updateHash(hash);
			this._hash = hash.digest("base64");
		}
		return this._hash;
	}
}

const mapStrings = new Map<
	string | HashConstructor,
	WeakMap<HashableObject, LazyHashedEtag>
>();

const mapObjects = new WeakMap<
	HashConstructor,
	WeakMap<HashableObject, LazyHashedEtag>
>();

/**
 * @param obj object with updateHash method
 * @param ashFunction the hash function to use
 * @returns etag
 */
export const getter = (
	obj: HashableObject,
	hashFunction: string | HashConstructor = "xxhash64"
): LazyHashedEtag => {
	let innerMap: WeakMap<HashableObject, LazyHashedEtag> | undefined;
	if (typeof hashFunction === "string") {
		innerMap = mapStrings.get(hashFunction);
		if (innerMap === undefined) {
			const newHash = new LazyHashedEtag(obj, hashFunction);
			innerMap = new WeakMap();
			innerMap.set(obj, newHash);
			mapStrings.set(hashFunction, innerMap);
			return newHash;
		}
	} else {
		innerMap = mapObjects.get(hashFunction);
		if (innerMap === undefined) {
			const newHash = new LazyHashedEtag(obj, hashFunction);
			innerMap = new WeakMap();
			innerMap.set(obj, newHash);
			mapObjects.set(hashFunction, innerMap);
			return newHash;
		}
	}
	const hash = innerMap.get(obj);
	if (hash !== undefined) return hash;
	const newHash = new LazyHashedEtag(obj, hashFunction);
	innerMap.set(obj, newHash);
	return newHash;
};

export default getter;
