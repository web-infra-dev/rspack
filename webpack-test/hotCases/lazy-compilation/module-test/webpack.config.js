"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		lazyCompilation: {
			entries: false,
			test: module => !/moduleB/.test(module.nameForCondition())
		}
	}
};
