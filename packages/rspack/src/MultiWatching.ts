/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/MultiWatching.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import asyncLib from "neo-async";

import type { MultiCompiler } from "./MultiCompiler";
import type { Watching } from "./Watching";

class MultiWatching {
	watchings: Watching[];
	compiler: MultiCompiler;

	/**
	 * @param watchings - child compilers' watchers
	 * @param compiler - the compiler
	 */
	constructor(watchings: Watching[], compiler: MultiCompiler) {
		this.watchings = watchings;
		this.compiler = compiler;
	}
	// @ts-expect-error
	invalidate(callback) {
		if (callback) {
			asyncLib.each(
				this.watchings,
				(watching, callback) => watching.invalidate(callback),
				callback
			);
		} else {
			for (const watching of this.watchings) {
				watching.invalidate();
			}
		}
	}

	/**
	 * @param {Callback<void>} callback signals when the watcher is closed
	 * @returns {void}
	 */
	// @ts-expect-error
	close(callback) {
		asyncLib.forEach(
			this.watchings,
			(watching, finishedCallback) => {
				watching.close(finishedCallback);
			},
			err => {
				this.compiler.hooks.watchClose.call();
				if (typeof callback === "function") {
					this.compiler.running = false;
					callback(err);
				}
			}
		);
	}

	suspend() {
		for (const watching of this.watchings) {
			watching.suspend();
		}
	}

	resume() {
		for (const watching of this.watchings) {
			watching.resume();
		}
	}
}

export default MultiWatching;
