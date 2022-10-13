import assert from "assert";
import { resolveOptions } from "@rspack/core";
import path from "path";

describe("config", () => {
	it("default config snapshot", () => {
		const resolvedOptions = resolveOptions({});
		assert.deepStrictEqual(resolvedOptions.context, process.cwd());
		assert.deepStrictEqual(
			resolvedOptions.devServer.static.directory,
			path.resolve(process.cwd(), "./dist")
		);

		// TypeScript will throw `The operand of a 'delete' operator must be optional`.
		// But we remove these configurations with absolute paths.
		delete (resolvedOptions as any).context;
		delete (resolvedOptions as any).devServer.static.directory;
		assert.deepStrictEqual(resolvedOptions, {
			mode: "development",
			devServer: {
				port: 8080,
				static: {
					watch: {}
				},
				devMiddleware: {},
				open: true,
				hmr: false,
				liveReload: true,
				webSocketServer: {}
			},
			entry: {},
			output: {
				assetModuleFilename: undefined,
				chunkFilename: undefined,
				filename: undefined,
				path: undefined,
				publicPath: undefined,
				uniqueName: undefined
			},
			define: {},
			target: ["web"],
			external: {},
			externalType: "",
			plugins: [],
			builtins: { browserslist: [] },
			module: { rules: [] },
			resolve: {
				alias: {},
				preferRelative: false,
				extensions: [".tsx", ".jsx", ".ts", ".js", ".json", ".d.ts"],
				mainFiles: ["index"],
				mainFields: ["module", "main"],
				browserField: true,
				conditionNames: ["module", "import"]
			},
			devtool: ""
		});
	});
});
