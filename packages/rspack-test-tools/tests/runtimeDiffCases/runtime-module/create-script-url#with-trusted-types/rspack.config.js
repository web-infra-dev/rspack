/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		trustedTypes: "customPolicyName",
		chunkLoading: "import-scripts"
	},
	entry: {
		other: "./src/index"
	}
};
