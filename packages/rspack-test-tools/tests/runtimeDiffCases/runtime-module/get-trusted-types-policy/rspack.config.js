/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		bundle: {
			import: "./src/index",
			chunkLoading: "import-scripts"
		}
	},
	output: {
		trustedTypes: {
			policyName: "my-application#webpack"
		}
	},
	target: "web"
};
