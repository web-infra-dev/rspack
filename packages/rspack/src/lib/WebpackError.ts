/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/WebpackError.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { inspect } from "node:util";
import { Chunk } from "./Chunk";
import { Module } from "./Module";
import { DependencyLocation } from "./Dependency";

export class WebpackError extends Error {
	loc?: DependencyLocation;
	file?: string;
	chunk?: Chunk;
	module?: Module;
	details?: string;
	hideStack?: boolean;

	/**
	 * Creates an instance of WebpackError.
	 * @param message error message
	 */
	constructor(message?: string) {
		super(message);
	}

	[inspect.custom]() {
		return this.stack + (this.details ? `\n${this.details}` : "");
	}
}

export default WebpackError;
