import fs from "node:fs";
import path from "node:path";
import { rspack } from "@rspack/core";

const loader = path.resolve(
	import.meta.dirname,
	"../configCases/loader/code-filter/transform-loader.mjs"
);

function runCompiler(options) {
	return new Promise((resolve, reject) => {
		const compiler = rspack(options);
		compiler.run((error, stats) => {
			const buildError =
				error ||
				(stats?.hasErrors()
					? new Error(stats.toString({ all: false, errors: true }))
					: null);
			compiler.close(closeError => {
				const finalError = buildError || closeError;
				if (finalError) reject(finalError);
				else resolve();
			});
		});
	});
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].map(newCache => {
	const name = newCache ? "new-cache" : "legacy-cache";
	let options;
	let sourceFile;
	let outputFile;

	return {
		description: `should retain code filter metadata with ${name} persistent cache`,
		options(context) {
			const root = context.getDist(`persistent-code-filter-${name}`);
			const sourceDir = path.join(root, "source");
			const outputDir = path.join(root, "output");
			sourceFile = path.join(sourceDir, "entry.js");
			outputFile = path.join(outputDir, "main.js");
			fs.rmSync(root, { recursive: true, force: true });
			fs.mkdirSync(sourceDir, { recursive: true });
			fs.writeFileSync(sourceFile, '// marker\nexport default "__VALUE__";\n');

			options = {
				context: sourceDir,
				entry: "./entry.js",
				mode: "development",
				devtool: false,
				output: {
					path: outputDir,
					filename: "main.js"
				},
				cache: {
					type: "persistent",
					storage: {
						type: "filesystem",
						location: path.join(root, "cache")
					}
				},
				experiments: {
					newCache: newCache
						? { codeGeneration: false, loader: true, minimize: false }
						: false
				},
				module: {
					rules: [
						{
							test: /entry\.js$/,
							use: [
								{
									loader,
									options: { value: "filtered" },
									filter: { code: newCache ? "marker" : /marker/ },
									cache: true
								}
							]
						}
					]
				}
			};
			return options;
		},
		async build(context) {
			await runCompiler(options);
			context.setValue("hit", fs.readFileSync(outputFile, "utf8"));
			await runCompiler(options);
			context.setValue("restored", fs.readFileSync(outputFile, "utf8"));

			fs.writeFileSync(sourceFile, 'export default "__VALUE__";\n');
			await runCompiler(options);
			context.setValue("miss", fs.readFileSync(outputFile, "utf8"));
		},
		check({ context }) {
			expect(context.getValue("hit")).toContain('"filtered"');
			expect(context.getValue("restored")).toContain('"filtered"');
			expect(context.getValue("miss")).toContain('"__VALUE__"');
			expect(context.getValue("miss")).not.toContain('"filtered"');
		}
	};
});
