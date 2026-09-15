module.exports = {
	findBundle() {
		return ["a.js"];
	},
	snapshotContent(content) {
		return content.replace(
			/\/\/ webpack\/runtime\/jsonp_chunk_loading[\s\S]*$/,
			"// webpack/runtime/jsonp_chunk_loading\n/* runtime omitted */"
		);
	}
};
