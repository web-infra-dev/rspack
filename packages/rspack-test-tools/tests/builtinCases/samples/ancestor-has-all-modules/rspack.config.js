/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
	},
	optimization: {
		providedExports: true,
		usedExports: "global"
	},
};
