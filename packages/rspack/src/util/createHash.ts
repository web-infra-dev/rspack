/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/util/createHash.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import crypto from "node:crypto";
import Hash from "./hash";
import createMd4 from "./hash/md4";
import createXXHash64 from "./hash/xxhash64";

const BULK_SIZE = 2000;

// We are using an object instead of a Map as this will stay static during the runtime
// so access to it can be optimized by v8
const digestCaches: Record<string, Map<string, string>> = {};

class BulkUpdateDecorator extends Hash {
	hash: Hash | undefined;
	hashFactory: (() => Hash) | undefined;
	hashKey: string | undefined;
	buffer: string;

	/**
	 * @param hashOrFactory function to create a hash
	 * @param hashKey key for caching
	 */
	constructor(hashOrFactory: Hash | (() => Hash), hashKey?: string) {
		super();
		this.hashKey = hashKey;
		if (typeof hashOrFactory === "function") {
			this.hashFactory = hashOrFactory;
			this.hash = undefined;
		} else {
			this.hashFactory = undefined;
			this.hash = hashOrFactory;
		}
		this.buffer = "";
	}

	/**
	 * Update hash {@link https://nodejs.org/api/crypto.html#crypto_hash_update_data_inputencoding}
	 * @param data data
	 * @param inputEncoding data encoding
	 * @returns updated hash
	 */
	update(data: string, inputEncoding: string): this;
	update(data: Buffer): this;
	update(data: string | Buffer, inputEncoding?: string): this {
		if (
			inputEncoding !== undefined ||
			typeof data !== "string" ||
			data.length > BULK_SIZE
		) {
			if (this.hash === undefined) this.hash = this.hashFactory!();
			if (this.buffer.length > 0) {
				this.hash.update(Buffer.from(this.buffer));
				this.buffer = "";
			}
			if (Buffer.isBuffer(data)) {
				this.hash.update(data);
			} else {
				this.hash.update(data, inputEncoding as BufferEncoding);
			}
		} else {
			this.buffer += data;
			if (this.buffer.length > BULK_SIZE) {
				if (this.hash === undefined) this.hash = this.hashFactory!();
				this.hash.update(Buffer.from(this.buffer));
				this.buffer = "";
			}
		}
		return this;
	}

	/**
	 * Calculates the digest without encoding
	 * @returns {Buffer} digest
	 */
	digest(): Buffer;
	/**
	 * Calculates the digest with encoding
	 * @param encoding encoding of the return value
	 * @returns {string} digest
	 */
	digest(encoding: string): string;
	/**
	 * Calculates the digest {@link https://nodejs.org/api/crypto.html#crypto_hash_digest_encoding}
	 * @param {string=} encoding encoding of the return value
	 * @returns {string|Buffer} digest
	 */
	digest(encoding?: string): string | Buffer {
		let digestCache: Map<string, string> | undefined;
		const buffer = this.buffer;
		if (this.hash === undefined) {
			// short data for hash, we can use caching
			const cacheKey = `${this.hashKey}-${encoding}`;
			digestCache = digestCaches[cacheKey];
			if (digestCache === undefined) {
				digestCache = digestCaches[cacheKey] = new Map();
			}
			const cacheEntry = digestCache.get(buffer);
			if (cacheEntry !== undefined)
				return encoding ? cacheEntry : Buffer.from(cacheEntry, "hex");
			this.hash = this.hashFactory!();
		}
		if (buffer.length > 0) {
			this.hash.update(Buffer.from(buffer));
		}
		const result = encoding ? this.hash.digest(encoding) : this.hash.digest();
		if (digestCache !== undefined && typeof result === "string") {
			digestCache.set(buffer, result);
		}
		return result;
	}
}

/* istanbul ignore next */
class DebugHash extends Hash {
	string: string;

	constructor() {
		super();
		this.string = "";
	}

	/**
	 * Update hash {@link https://nodejs.org/api/crypto.html#crypto_hash_update_data_inputencoding}
	 * @param data data
	 * @param inputEncoding data encoding
	 * @returns updated hash
	 */
	update(data: string, inputEncoding: string): this;
	update(data: Buffer): this;
	update(data: string | Buffer, inputEncoding?: string): this {
		let normalizedData: string;
		if (Buffer.isBuffer(data)) {
			normalizedData = data.toString("utf-8");
		} else {
			normalizedData = data;
		}

		if (normalizedData.startsWith("debug-digest-")) {
			normalizedData = Buffer.from(
				normalizedData.slice("debug-digest-".length),
				"hex"
			).toString();
		}

		this.string += `[${normalizedData}](${new Error().stack?.split("\n", 3)[2]})\n`;
		return this;
	}

	/**
	 * Calculates the digest without encoding
	 * @returns {Buffer} digest
	 */
	digest(): Buffer;
	/**
	 * Calculates the digest with encoding
	 * @param encoding encoding of the return value
	 * @returns {string} digest
	 */
	digest(encoding: string): string;
	/**
	 * Calculates the digest {@link https://nodejs.org/api/crypto.html#crypto_hash_digest_encoding}
	 * @param {string=} encoding encoding of the return value
	 * @returns {string|Buffer} digest
	 */
	digest(encoding?: string): string | Buffer {
		const result = `debug-digest-${Buffer.from(this.string).toString("hex")}`;
		return encoding ? result : Buffer.from(result);
	}
}

class WasmHashAdapter extends Hash {
	private wasmHash: any;

	constructor(wasmHash: any) {
		super();
		this.wasmHash = wasmHash;
	}

	/**
	 * Update hash {@link https://nodejs.org/api/crypto.html#crypto_hash_update_data_inputencoding}
	 * @param data data
	 * @param inputEncoding data encoding
	 * @returns updated hash
	 */
	update(data: string, inputEncoding: string): this;
	update(data: Buffer): this;
	update(data: string | Buffer, inputEncoding?: string): this {
		if (Buffer.isBuffer(data)) {
			this.wasmHash.update(data);
		} else {
			this.wasmHash.update(data, inputEncoding as BufferEncoding);
		}
		return this;
	}

	/**
	 * Calculates the digest without encoding
	 * @returns {Buffer} digest
	 */
	digest(): Buffer;
	/**
	 * Calculates the digest with encoding
	 * @param encoding encoding of the return value
	 * @returns {string} digest
	 */
	digest(encoding: string): string;
	/**
	 * Calculates the digest {@link https://nodejs.org/api/crypto.html#crypto_hash_digest_encoding}
	 * @param {string=} encoding encoding of the return value
	 * @returns {string|Buffer} digest
	 */
	digest(encoding?: string): string | Buffer {
		return encoding ? this.wasmHash.digest(encoding) : this.wasmHash.digest();
	}
}

/**
 * Creates a hash by name or function
 * @param algorithm the algorithm name or a constructor creating a hash
 * @returns the hash
 */
export const createHash = (
	algorithm:
		| "debug"
		| "xxhash64"
		| "md4"
		| "native-md4"
		| (string & {})
		| (new () => Hash)
): Hash => {
	if (typeof algorithm === "function") {
		return new BulkUpdateDecorator(() => new algorithm());
	}
	switch (algorithm) {
		// TODO add non-cryptographic algorithm here
		case "debug":
			return new DebugHash();
		case "xxhash64": {
			const hash = createXXHash64();
			return new WasmHashAdapter(hash);
		}
		case "md4": {
			const hash = createMd4();
			return new WasmHashAdapter(hash);
		}
		case "native-md4":
			return new BulkUpdateDecorator(() => crypto.createHash("md4"), "md4");
		default:
			return new BulkUpdateDecorator(
				() => crypto.createHash(algorithm),
				algorithm
			);
	}
};
