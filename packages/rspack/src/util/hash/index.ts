/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/util/hash
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import AbstractMethodError from "../../lib/AbstractMethodError";

export default class Hash {
	/* istanbul ignore next */
	/**
	 * Update hash {@link https://nodejs.org/api/crypto.html#crypto_hash_update_data_inputencoding}
	 * @abstract
	 * @param data data
	 * @param inputEncoding data encoding
	 * @returns updated hash
	 */
	update(data: string | Buffer, inputEncoding?: string): this {
		throw new AbstractMethodError();
	}

	/* istanbul ignore next */
	/**
	 * Calculates the digest {@link https://nodejs.org/api/crypto.html#crypto_hash_digest_encoding}
	 * @abstract
	 * @param encoding encoding of the return value
	 * @returns  digest
	 */
	digest(encoding?: string): string | Buffer {
		throw new AbstractMethodError();
	}
}
