// @ts-check
import { copyFileSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);

/** @type {import('prebundle').Config} */
export default {
	dependencies: [
		{
			name: "zod",
			copyDts: true
		},
		{
			name: "webpack-sources",
			copyDts: true
		},
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

				// Rspack only depend on the `CachedInputFileSystem` module of enhanced-resolve
				// so we can remove the index chunk because it is not used
				unlinkSync(join(distPath, "index.js"));

				// add an empty CachedInputFileSystem.d.ts file to prevent ts error
				writeFileSync(join(distPath, "CachedInputFileSystem.d.ts"), "");
			}
		},
		{
			name: "@swc/types"
		}
	]
};
