"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	lazyCompilation: {
		cacheable: false,
		entries: false,
		imports: true
	}
};
