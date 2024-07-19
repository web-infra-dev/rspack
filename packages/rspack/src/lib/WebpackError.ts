/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Jarid Margolin @jaridmargolin
*/

import util from "node:util";
import type { Chunk, Module } from "../exports";

const inspect = util.inspect.custom;

type DependencyLocation = any;

class WebpackError extends Error {
	module: Module | undefined;
	details: string | undefined;
	loc: DependencyLocation | undefined;
	hideStack: boolean | undefined;
	chunk: Chunk | undefined;
	file: string | undefined;
	/**
	 * Creates an instance of WebpackError.
	 * @param {string=} message error message
	 */
	constructor(message: string) {
		super(message);

		this.details = undefined;

		this.module = undefined;

		this.loc = undefined;

		this.hideStack = undefined;

		this.chunk = undefined;

		this.file = undefined;
	}

	[inspect]() {
		return this.stack + (this.details ? `\n${this.details}` : "");
	}

	// serialize({ write }) {
	// 	write(this.name);
	// 	write(this.message);
	// 	write(this.stack);
	// 	write(this.details);
	// 	write(this.loc);
	// 	write(this.hideStack);
	// }
	//
	// deserialize({ read }) {
	// 	this.name = read();
	// 	this.message = read();
	// 	this.stack = read();
	// 	this.details = read();
	// 	this.loc = read();
	// 	this.hideStack = read();
	// }
}

// makeSerializable(WebpackError, "webpack/lib/WebpackError");

export { WebpackError };
