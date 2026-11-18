import path from "node:path";

const excludes = {
	normal: [],
	config: [
		// This case expects stateful loaders to build again on every compiler.
		/^css-extract\/rebuild-module$/,

		// Module cache misses after restarting the compiler.
		/^asset-modules\/custom-encoder$/,
		/^asset-modules\/keep-source-maps$/,
		/^asset-modules\/query-and-custom-encoder$/,
		/^asset-modules\/rule-generator-publicPath$/,
		/^asset\/filename$/,
		/^contenthash\/assets$/,
		/^css\/import-filter$/,
		/^custom-modules\/json-custom$/,
		/^json\/top-level-json-parser$/,
		/^loader-import-module\/execute-failed$/,
		/^loader-parallel-import-module\/execute-failed$/,
		/^module\/rspack-issue-8785$/,

		// Loader-emitted errors or warnings are not fully restored.
		/^source-map\/extract-source-map$/,

		// Existing bundle/compiler assertions fail in the compatibility suite.
		/^loader\/context-module$/,

		// Shared extracted CSS can retain different producers' loader dependencies.
		/^plugins\/mini-css-extract-plugin$/
	]
};

export function getCacheTestOptions(type, src) {
	// Persistent cache suites are disabled for WASM in rstack.config.mts.
	if (process.env.WASM) return;

	const source = `${type}Cases`;
	const name = path
		.relative(path.join(import.meta.dirname, source), src)
		.replace(/\\/g, "/");
	const category = name.split("/")[0];
	if (
		excludes[type].some(exclude => exclude.test(category) || exclude.test(name))
	) {
		return;
	}

	return {
		directory: path.join(
			import.meta.dirname,
			"js",
			"temp",
			"new-cache",
			source,
			name,
			".cache"
		)
	};
}
