// @ts-check
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/** @type {import('prebundle').Config} */
export default {
	dependencies: [
		{
			name: "webpack-sources",
			copyDts: true
		},
		"graceful-fs",
		"browserslist-load-config",
		{
			name: "watchpack",
			externals: {
				"graceful-fs": "../graceful-fs/index.js"
			},
			afterBundle(task) {
				const dtsPath = join(task.distPath, "index.d.ts");
				const content = readFileSync(dtsPath, "utf-8");
				// Ensure the type of graceful-fs is imported from the correct path
				writeFileSync(
					dtsPath,
					content.replace(
						"from 'graceful-fs'",
						"from '../graceful-fs/index.js'"
					)
				);
			}
		},
		{
			name: "@swc/types"
		}
	]
};
