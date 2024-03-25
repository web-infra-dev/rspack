/** @type {import("../../../../dist").Configuration} */
module.exports = {
	output: {
		library: {
			type: "module",
		},
		enabledLibraryTypes: ["module", "module"],
	},
	target: ["es2022"],
	experiments: {
		outputModule: true,
		rspackFuture: {
			newTreeshaking: true,
		}
	}
};
