/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	module: {
		parser: {
			json: {
				exportsDepth: Infinity
			}
		}
	}
};
