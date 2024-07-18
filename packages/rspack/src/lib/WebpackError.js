/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Jarid Margolin @jaridmargolin
*/

"use strict";

const inspect = require("node:util").inspect.custom;

// /** @typedef {import("./Chunk")} Chunk */
// /** @typedef {import("./Dependency").DependencyLocation} DependencyLocation */
// /** @typedef {import("./Module")} Module */
/** @typedef {any} Chunk */
/** @typedef {any} DependencyLocation */
/** @typedef {any} Module */

class WebpackError extends Error {
	/**
	 * Creates an instance of WebpackError.
	 * @param {string=} message error message
	 */
	constructor(message) {
		super(message);
		// @ts-expect-error
		this.details = undefined;
		/** @type {Module} */
		this.module = undefined;
		/** @type {DependencyLocation} */
		this.loc = undefined;
		/** @type {boolean} */
		// @ts-expect-error
		this.hideStack = undefined;
		/** @type {Chunk} */
		this.chunk = undefined;
		/** @type {string} */
		// @ts-expect-error
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

module.exports = WebpackError;
