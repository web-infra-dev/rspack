// @ts-check
import { copyFileSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/** @type {import('prebundle').Config} */
export default {
	dependencies: [
		"zod",
		"zod-validation-error",
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
				tapable: "tapable",
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
		}
	]
};
