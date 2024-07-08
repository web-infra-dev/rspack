"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		lazyCompilation: {
			cacheable: false,
			entries: false,
			imports: true,
			backend: {
				listen: {
					host: "127.0.0.1"
				}
			}
		}
	}
};
