// One build per process, so a writer and a reader share nothing but the cache on disk.
import path from "node:path";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

const { context, writer, kinds, loaderCache, cached } = JSON.parse(process.argv[2]);

const values = {
	set: (origin) => new Set([origin]),
	function: (origin) => () => origin,
	plain: () => ({ list: [1, "x", null, undefined, true], nested: { ratio: 1.5 } })
};

// Static options whose objects share their children.
function sharedGraph() {
	let graph = { leaf: 1 };
	for (let depth = 0; depth < 24; depth++) graph = { left: graph, right: graph };
	return { value: "shared", graph };
}

const compiler = rspack({
	context,
	mode: "development",
	devtool: false,
	incremental: false,
	target: "node",
	entry: "./src/index.js",
	output: {
		path: path.join(context, "dist"),
		filename: "main.js",
		library: { type: "commonjs2" }
	},
	optimization: { minimize: false },
	infrastructureLogging: { level: "error" },
	module: {
		rules: kinds.map((kind) => ({
			test: new RegExp(`${kind}-dep\\.js$`),
			use: kind === "shared"
				? [{ loader: "./observe.cjs", cache: loaderCache, options: sharedGraph() }]
				: (info) => [{
					loader: "./observe.cjs",
					cache: loaderCache,
					options: {
						value: values[kind](info.resource.startsWith(writer + path.sep) ? "writer" : "reader")
					}
				}]
		}))
	},
	cache: cached ? { type: "persistent", portable: true } : false
});

try {
	const stats = await runCompiler(compiler);
	const { errors } = stats.toJson({ all: false, errors: true });
	if (errors.length > 0) throw new Error(JSON.stringify(errors));
	process.stdout.write(`${JSON.stringify({
		executed: createRequire(import.meta.url)(path.join(context, "dist", "main.js")),
		assets: Object.fromEntries(stats.compilation.getAssets().map(({ name, source }) => [
			name, source.source().toString()
		])),
		built: [...stats.compilation.builtModules].map((module) => module.identifier())
	})}\n`);
} finally {
	await closeCompiler(compiler);
}
