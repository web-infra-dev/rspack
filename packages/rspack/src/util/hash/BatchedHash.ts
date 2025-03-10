/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/util/hash/BatchedHash.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import Hash from ".";
import { MAX_SHORT_STRING } from "./wasm-hash";

export default class BatchedHash extends Hash {
	string: string | undefined;
	encoding: string | undefined;
	hash: Hash;

	constructor(hash: Hash) {
		super();
		this.string = undefined;
		this.encoding = undefined;
		this.hash = hash;
	}

	/**
	 * Update hash {@link https://nodejs.org/api/crypto.html#crypto_hash_update_data_inputencoding}
	 * @param data data
	 * @param inputEncoding data encoding
	 * @returns updated hash
	 */
	update(data: string | Buffer, inputEncoding?: string) {
		if (this.string !== undefined) {
			if (
				typeof data === "string" &&
				inputEncoding === this.encoding &&
				this.string.length + data.length < MAX_SHORT_STRING
			) {
				this.string += data;
				return this;
			}
			if (this.string && this.encoding) {
				this.hash.update(this.string, this.encoding);
			} else if (this.string) {
				this.hash.update(Buffer.from(this.string));
			}
			this.string = undefined;
		}
		if (typeof data === "string") {
			if (
				data.length < MAX_SHORT_STRING &&
				// base64 encoding is not valid since it may contain padding chars
				(!inputEncoding || !inputEncoding.startsWith("ba"))
			) {
				this.string = data;
				this.encoding = inputEncoding;
			} else if (inputEncoding) {
				this.hash.update(data, inputEncoding);
			} else {
				this.hash.update(Buffer.from(data));
			}
		} else {
			this.hash.update(data);
		}
		return this;
	}

	digest(): Buffer;

	digest(encoding: string): string;

	/**
	 * Calculates the digest {@link https://nodejs.org/api/crypto.html#crypto_hash_digest_encoding}
	 * @param encoding encoding of the return value
	 * @returns digest
	 */
	digest(encoding?: string): string | Buffer {
		if (this.string !== undefined) {
			if (this.string && this.encoding) {
				this.hash.update(this.string, this.encoding);
			} else if (this.string) {
				this.hash.update(Buffer.from(this.string));
			}
		}
		return this.hash.digest(encoding!);
	}
}
