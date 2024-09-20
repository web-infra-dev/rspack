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

import type { Callback } from "@rspack/lite-tapable";
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
	invalidate(callback: Callback<Error, void>) {
		if (callback) {
			asyncLib.each(
				this.watchings,
				(watching, callback) => watching.invalidate(callback),
				// cannot be resolved without assertion
				// Type 'Error | null | undefined' is not assignable to type 'Error | null'
				callback as (err: Error | null | undefined) => void
			);
		} else {
			for (const watching of this.watchings) {
				watching.invalidate();
			}
		}
	}

	close(callback: Callback<Error, void>) {
		asyncLib.forEach(
			this.watchings,
			(watching, finishedCallback) => {
				watching.close(finishedCallback);
			},
			err => {
				this.compiler.hooks.watchClose.call();
				if (typeof callback === "function") {
					this.compiler.running = false;
					// cannot be resolved without assertion
					// Type 'Error | null | undefined' is not assignable to type 'Error | null'
					callback(err as Error | null);
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
