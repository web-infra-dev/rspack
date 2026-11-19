import path from "node:path";

const excludes = {
	normal: [],
	config: [
		// This case expects stateful loaders to build again on every compiler.
		/^css-extract\/rebuild-module$/,

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
