/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/util/hash/BatchedHash.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

const Hash = require(".");
const MAX_SHORT_STRING = require("./wasm-hash").MAX_SHORT_STRING;

class BatchedHash extends Hash {
	constructor(hash) {
		super();
		this.string = undefined;
		this.encoding = undefined;
		this.hash = hash;
	}

	/**
	 * Update hash {@link https://nodejs.org/api/crypto.html#crypto_hash_update_data_inputencoding}
	 * @param {string|Buffer} data data
	 * @param {string=} inputEncoding data encoding
	 * @returns {this} updated hash
	 */
	update(data, inputEncoding) {
		if (this.string !== undefined) {
			if (
				typeof data === "string" &&
				inputEncoding === this.encoding &&
				this.string.length + data.length < MAX_SHORT_STRING
			) {
				this.string += data;
				return this;
			}
			this.hash.update(this.string, this.encoding);
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
			} else {
				this.hash.update(data, inputEncoding);
			}
		} else {
			this.hash.update(data);
		}
		return this;
	}

	/**
	 * Calculates the digest {@link https://nodejs.org/api/crypto.html#crypto_hash_digest_encoding}
	 * @param {string=} encoding encoding of the return value
	 * @returns {string|Buffer} digest
	 */
	digest(encoding) {
		if (this.string !== undefined) {
			this.hash.update(this.string, this.encoding);
		}
		return this.hash.digest(encoding);
	}
}

module.exports = BatchedHash;
