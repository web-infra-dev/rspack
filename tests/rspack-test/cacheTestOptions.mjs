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

		// Source map cache misses after restarting the compiler.
		/^source-map\/empty-mapping$/,

		// Independent shared fallback compilers write to the same cache database as their parent.
		/^container-1-5\/tree-shaking-shared-infer-mode$/,
		/^container-1-5\/tree-shaking-shared-pnpm-versions$/,
		/^container-1-5\/tree-shaking-shared-provide-only$/,
		/^container-1-5\/tree-shaking-shared-server-mode$/,
		/^sharing\/reshake-share$/,
		/^sharing\/tree-shaking-shared$/,

		// Unawaited child compilation races with closing/flushing the parent cache.
		/^chunk-loading\/issue-4754$/,
		/^plugins\/rspack-issue-4982$/,

		// Restored modules skip loader importModule(), but this case asserts both graphs.
		/^stats\/issuer-information-cache$/,

		// Loader-emitted errors or warnings are not fully restored.
		/^loader\/emit-error$/,
		/^source-map\/extract-source-map$/,

		// Existing bundle/compiler assertions fail in the compatibility suite.
		/^context\/import-meta-glob-case-insensitive-pruning$/,
		/^loader\/context-module$/,
		/^module\/shared-accessors$/,
		/^plugins\/circular-check-rspack-plugin$/,

		// Module cache entries are not fully reused after restarting the compiler.
		/^builtin-swc-loader\/react-compiler-long-expressions$/,
		/^builtins\/provide-issue-8553$/,
		/^chunk-loading\/worker-loader$/,
		/^loader\/hook-context-initialization$/,
		// Shared extracted CSS can retain different producers' loader dependencies.
		/^plugins\/mini-css-extract-plugin$/,
		/^plugins\/virtual-modules-context$/,
		/^rsc-plugin\/client-chunk-grouping$/,
		/^rsc-plugin\/css-link-props$/,
		/^rsc-plugin\/entry-with-multiple-modules$/,
		/^rsc-plugin\/on-manifest$/,
		/^rsc-plugin\/rsc-entry-debug-comments$/,
		/^rsc-plugin\/server-css$/,
		/^rsc-plugin\/server-css-async-chunks-false$/,
		/^rsc-plugin\/server-entry$/,
		/^rsc-plugin\/tree-shaking$/,
		/^rsc-plugin\/tree-shaking-client-barrel$/,
		/^rsc-plugin\/with-concatenated-module$/,
		/^rsc-plugin\/worker$/,
		/^split-chunks\/batch-name-output-order$/,
		/^split-chunks\/dedup-depth-plugin$/,
		/^split-chunks\/intersections-min-size$/
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
