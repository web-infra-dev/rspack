import path from "node:path";
import { describeByWalk, createCacheCase } from "@rspack/test-tools";

const suites = [
	["newCacheCases", "cache"],
	["normalCases", "normal"],
	["configCases", "config"]
];

const excludes = {
	cache: [
		// Expects legacy storage metadata; newCache does not implement maxAge yet.
		/^storage\/max-age$/
	],
	normal: [
		// Module restoration does not retain loader-emitted diagnostics yet.
		/^errors\/loader-error-warning$/,
		// These modules do not yet produce reusable persistent cache entries.
		/^loaders\/coffee-loader$/,
		/^parsing\/extract-amd$/,

		// Module cache misses after restarting the compiler.
		/^errors\/import-module-cycle$/,
		/^errors\/import-module-cycle-multiple$/,

		// Code generation cache misses after restarting the compiler.
		/^optimize\/side-effects-simple-unused$/,

		// Resolver cache misses, including unresolved requests.
		/^cjs-tree-shaking\/full-require$/,
		/^parsing\/extract-amd\.nominimize$/,
		/^parsing\/webpack-is-included$/,
		/^runtime\/missing-module-exception$/,
		/^runtime\/missing-module-exception-dynamic-import$/,
		/^runtime\/missing-module-exception-require$/,
		/^warnings\/not-fully-specified$/,

		// Existing bundle/compiler assertions fail in the compatibility suite.
		/^parsing\/rspack-issue-4816$/
	],
	config: [
		// Restored async blocks are not yet available through the binding API.
		/^compilation\/modules$/,
		// This case expects its stateful loader to build again on every compiler.
		/^compilation\/rebuild-module(?:-getters)?$/,

		// Module cache misses after restarting the compiler.
		/^asset-modules\/custom-encoder$/,
		/^asset-modules\/keep-source-maps$/,
		/^asset-modules\/query-and-custom-encoder$/,
		/^asset-modules\/rule-generator-publicPath$/,
		/^asset\/filename$/,
		/^assets\/shared-by-mf$/,
		/^chunk-ids\/stable-chunk-ids-with-duplicate-chunks$/,
		/^concatenate-modules\/consume-shared$/,
		/^container-1-0\/consume-module-mjs-default-export$/,
		/^container-1-5\/consume-shared-output-module-without-chunk-loading$/,
		/^container-1-5\/container-entry-overridables$/,
		/^container-1-5\/exposed-overridables$/,
		/^container-1-5\/fallback-context$/,
		/^container-1-5\/manifest$/,
		/^container-1-5\/manifest-disable-assets-analyze$/,
		/^container-1-5\/manifest-entry-filter$/,
		/^container-1-5\/manifest-file-name$/,
		/^container-1-5\/manifest-prefix-singleton$/,
		/^container-1-5\/provide-sharing-extra-data$/,
		/^container-1-5\/remote-array-share-scope$/,
		/^container-1-5\/runtime-plugin-with-params$/,
		/^container-1-5\/runtime-plugin-with-used-exports$/,
		/^container-1-5\/share-strategy$/,
		/^container-1-5\/tree-shaking-shared-infer-mode$/,
		/^container-1-5\/tree-shaking-shared-pnpm-versions$/,
		/^container-1-5\/tree-shaking-shared-server-mode$/,
		/^container\/container-entry-overridables$/,
		/^container\/exposed-overridables$/,
		/^contenthash\/assets$/,
		/^css\/import-filter$/,
		/^custom-modules\/json-custom$/,
		/^json\/top-level-json-parser$/,
		/^loader-import-module\/execute-failed$/,
		/^loader-parallel-import-module\/execute-failed$/,
		/^module\/rspack-issue-8785$/,
		/^resolve\/mf-by-dependency-esm$/,
		/^sharing\/consume-module-namespace$/,
		/^sharing\/consume-multiple-versions$/,
		/^sharing\/consume-self-reference$/,
		/^sharing\/no-override-loaded$/,
		/^sharing\/share-multiple-versions$/,
		/^sharing\/share-plugin$/,
		/^sharing\/share-plugin-dual-mode$/,
		/^sharing\/share-plugin-monorepo$/,
		/^sharing\/share-plugin-secondary-entry-point$/,
		/^sri\/mf$/,
		/^tree-shaking\/side-effects-consume-shared$/,

		// Code generation cache misses after restarting the compiler.
		/^entry\/public-path$/,
		/^sharing\/tree-shaking-shared$/,

		// Resolver cache misses, including unresolved requests.
		/^rstest\/mock$/,
		/^rstest\/mock-dynamic-import-external$/,

		// Source map cache misses after restarting the compiler.
		/^asset\/real-content-hash-related-source-map$/,
		/^asset\/webpack-sources$/,
		/^builtin-swc-loader\/source-map$/,
		/^builtins\/banner$/,
		/^builtins\/banner-stage$/,
		/^less-loader\/with-source-map$/,
		/^loader-parallel\/binary-with-source-map$/,
		/^loader\/binary-with-source-map$/,
		/^source-map\/devtool-skip-non-js-css-assets$/,
		/^source-map\/empty-mapping$/,
		/^source-map\/exclude-chunks-source-map$/,

		// Child compiler metadata is not fully restored.
		/^plugins\/rspack-issue-4982$/,

		// The child compilation is not awaited, so its cache may not be stored.
		/^chunk-loading\/issue-4754$/,

		// Stats assertions require build-time modules that cache restoration skips.
		/^stats\/issuer-information-cache$/,

		// Loader-emitted errors or warnings are not fully restored.
		/^compilation\/stats-artifact-fallback-warning$/,
		/^loader\/emit-error$/,
		/^source-map\/extract-source-map$/,

		// afterExecute removes the temporary Browserslist packages before cache builds.
		/^ecmaVersion\/browserslist-config-env-extends$/,
		/^ecmaVersion\/browserslist-config-extends$/,
		/^ecmaVersion\/browserslist-extends$/,

		// Reads a hard-coded output path from the regular Config suite.
		/^library\/esm-external$/,

		// Existing bundle/compiler assertions fail in the compatibility suite.
		/^builtin-lightningcss-loader\/custom-media$/,
		/^compilation\/errors$/,
		/^context\/import-meta-glob-case-insensitive-pruning$/,
		/^css-chunking-plugin\/base-global-css-chunking$/,
		/^filename-template\/filename-function-entry-chunk$/,
		/^hooks\/build-module$/,
		/^hooks\/original-source$/,
		/^hooks\/original-source-map-presence$/,
		/^hooks\/still-valid-module$/,
		/^hooks\/succeed-module$/,
		/^lazy-barrel\/basic$/,
		/^loader\/context-module$/,
		/^loader\/context-path$/,
		/^loader\/context-resource$/,
		/^module\/shared-accessors$/,
		/^plugins\/circular-check-rspack-plugin$/,
		/^plugins\/progress-plugin$/,
		/^rstest\/dynamic-import-origin$/,
		/^rstest\/dynamic-import-origin-function-name$/,
		/^rstest\/import-meta-rstest$/,
		/^rstest\/module-path-names$/,
		/^rstest\/require-resolve-origin$/,
		/^split-chunks\/batch-functions$/,
		/^split-chunks\/callback-order$/
	]
};

if (process.platform === "win32") {
	// Resolver cache misses after restarting the compiler on Windows.
	excludes.normal.push(/^parsing\/evaluate$/);
	// Cache builds query filesystem metadata with resource queries in the path.
	excludes.config.push(
		/^context-modules\/replacement-d$/,
		/^context-replacement\/d$/
	);
}

for (const [source, type] of suites) {
	describeByWalk(
		import.meta.filename,
		(name, src, dist) => {
			createCacheCase(
				name,
				src,
				dist,
				"async-node",
				path.join(
					import.meta.dirname,
					"js",
					"temp",
					"new-cache",
					source,
					path.relative(path.join(import.meta.dirname, source), src)
				),
				{ type, newCache: true }
			);
		},
		{
			source: path.join(import.meta.dirname, source),
			dist: path.join(import.meta.dirname, "js", "new-cache", source),
			exclude: excludes[type]
		}
	);
}
