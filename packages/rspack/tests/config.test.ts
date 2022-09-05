import { test } from "uvu";
import * as assert from "uvu/assert";
import { Rspack } from "@rspack/core";
import path from "path";

test("default config snapshot", () => {
	const resolvedOptions = new Rspack({}).options;

	assert.equal(resolvedOptions.context, process.cwd());
	assert.equal(
		resolvedOptions.dev.static.directory,
		path.resolve(process.cwd(), "./dist")
	);

	// TypeScript will throw `The operand of a 'delete' operator must be optional`.
	// But we remove these configurations with absolute paths.
	// @ts-expect-error
	delete resolvedOptions.context;
	// @ts-expect-error
	delete resolvedOptions.dev.static.directory;

	assert.equal(resolvedOptions, {
		mode: "development",
		dev: { port: 8080, static: {}, hmr: true, open: true },
		entry: {},
		output: {
			path: undefined,
			publicPath: undefined,
			chunkFilename: undefined,
			filename: undefined,
			assetModuleFilename: undefined,
			uniqueName: undefined
		},
		define: {},
		target: "",
		external: {},
		plugins: [],
		module: { rules: [] }
	});
});

test.run();
