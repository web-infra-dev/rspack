import assert from "assert";
import { Rspack } from "@rspack/core";
import path from "path";
describe("config", () => {
	it("default config snapshot", () => {
		const resolvedOptions = new Rspack({}).options;
		assert.equal(resolvedOptions.context, process.cwd());
		assert.equal(
			resolvedOptions.dev.static.directory,
			path.resolve(process.cwd(), "./dist")
		);

		// TypeScript will throw `The operand of a 'delete' operator must be optional`.
		// But we remove these configurations with absolute paths.
		delete (resolvedOptions as any).context;
		delete (resolvedOptions as any).dev.static.directory;
		assert.equal(
			JSON.stringify(resolvedOptions),
			JSON.stringify({
				mode: "development",
				dev: { port: 8080, static: {} },
				entry: {},
				output: {},
				define: {},
				target: ["web"],
				external: {},
				plugins: [],
				builtins: [],
				module: { rules: [] },
				resolve: {
					preferRelative: false,
					extensions: [".tsx", ".jsx", ".ts", ".js", ".json", ".d.ts"],
					mainFiles: ["index"],
					mainFields: ["module", "main"],
					browserField: true,
					conditionNames: ["module", "import"]
				}
			})
		);
	});
});
