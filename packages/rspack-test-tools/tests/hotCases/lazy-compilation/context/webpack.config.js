

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		lazyCompilation: {
			cacheable: false,
			entries: false,
			imports: true
		}
	}
};
