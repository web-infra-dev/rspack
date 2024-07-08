// @ts-check
import { copyFileSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

/** @type {import('prebundle').Config} */
export default {
	dependencies: [
		"zod",
		{
			name: "zod-validation-error",
			externals: {
				zod: "../zod"
			}
		},
		"json-parse-even-better-errors",
		"neo-async",
		"graceful-fs",
		{
			name: "watchpack",
			externals: {
				"graceful-fs": "../graceful-fs/index.js"
			}
		},
		{
			name: "browserslist",
			ignoreDts: true,
			externals: {
				"caniuse-lite": "caniuse-lite",
				"/^caniuse-lite(/.*)/": "caniuse-lite$1"
			},
			// preserve the `require(require.resolve())`
			beforeBundle(task) {
				const nodeFile = join(task.depPath, "node.js");
				const content = readFileSync(nodeFile, "utf-8");
				writeFileSync(
					nodeFile,
					content.replaceAll(
						"require(require.resolve",
						'eval("require")(require.resolve'
					)
				);
			}
		},
		{
			name: "enhanced-resolve",
			externals: {
				tapable: "@rspack/lite-tapable",
				"graceful-fs": "../graceful-fs/index.js"
			},
			afterBundle({ depPath, distPath }) {
				copyFileSync(
					join(depPath, "lib/CachedInputFileSystem.js"),
					join(distPath, "CachedInputFileSystem.js")
				);

				// ResolveRequest type is used by Rspack but not exported
				const dtsFile = join(distPath, "index.d.ts");
				const content = readFileSync(dtsFile, "utf-8");
				writeFileSync(
					dtsFile,
					content.replace(
						"type ResolveRequest =",
						"export type ResolveRequest ="
					)
				);
			}
		},
		{
			name: "webpack-sources",
			ignoreDts: true,
			afterBundle(task) {
				const __filename = fileURLToPath(import.meta.url);
				const __dirname = dirname(__filename);
				const dtsInputPath = join(
					__dirname,
					"declarations/webpack-sources.d.ts"
				);
				const dtsContent = readFileSync(dtsInputPath, "utf-8");
				const dtsOutputPath = join(task.distPath, "index.d.ts");
				writeFileSync(dtsOutputPath, dtsContent, "utf-8");
			}
		}
	]
};
