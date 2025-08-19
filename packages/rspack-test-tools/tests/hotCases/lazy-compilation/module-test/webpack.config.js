"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	lazyCompilation: {
		entries: false,
		cacheable: false,
		test: /moduleA/
	}
};
