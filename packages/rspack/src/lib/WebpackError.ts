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
import type { Chunk } from "../Chunk";
import type { Module } from "../Module";

// Waiting to adapt
type DependencyLocation = any;

export class WebpackError extends Error {
	loc?: DependencyLocation;
	file?: string;
	chunk?: Chunk;
	module?: Module;
	details?: string;
	hideStack?: boolean;

	[inspect.custom]() {
		return this.stack + (this.details ? `\n${this.details}` : "");
	}
}

export default WebpackError;
