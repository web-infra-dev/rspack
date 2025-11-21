/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		chunkLoadingGlobal: "__LOADED_CHUNKS__"
	},
	target: "web"
};
