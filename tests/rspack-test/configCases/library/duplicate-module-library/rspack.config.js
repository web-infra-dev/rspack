/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		module: true,
		library: {
			type: "module"
		},
		enabledLibraryTypes: ["module", "module"]
	},
	target: ["es2022"],
};
