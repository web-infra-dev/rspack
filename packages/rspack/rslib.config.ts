import fs from "node:fs";
import path from "node:path";
import type { RsbuildPlugin } from "@rsbuild/core";
import { type LibConfig, defineConfig, rsbuild } from "@rslib/core";
import prebundleConfig from "./prebundle.config.mjs";

const merge = rsbuild.mergeRsbuildConfig;

const externalAlias = ({ request }: { request?: string }, callback) => {
	const { dependencies } = prebundleConfig;

	for (const item of dependencies) {
		const depName = typeof item === "string" ? item : item.name;
		if (new RegExp(`^${depName}$`).test(request!)) {
			return callback(null, `../compiled/${depName}/index.js`);
		}
	}

	if (new RegExp(/^tinypool$/).test(request!)) {
		return callback(null, "../compiled/tinypool/dist/index.js");
	}

	return callback();
};

const commonLibConfig: LibConfig = {
	format: "cjs",
	syntax: ["node 16"],
	source: {
		define: {
			WEBPACK_VERSION: JSON.stringify(require("./package.json").webpackVersion),
			RSPACK_VERSION: JSON.stringify(require("./package.json").version)
		}
	},
	output: {
		externals: ["@rspack/binding/package.json", externalAlias],
		minify: {
			js: true,
			jsOptions: {
				minimizerOptions: {
					// preserve variable name and disable minify for easier debugging
					mangle: false,
					minify: false,
					compress: {
						// enable to compress import.meta.url shims in top level scope
						toplevel: true,
						// keep debugger so we can debug in the debug terminal without need to search in minified dist
						drop_debugger: false
					}
				}
			}
		}
	},
	tools: {
		bundlerChain: (chain, { CHAIN_ID }) => {
			// remove the entry loader in Rslib to avoid
			// "Cannot access 'Compiler' before initialization" error caused by circular dependency
			chain.module
				.rule(`Rslib:${CHAIN_ID.RULE.JS}-entry-loader`)
				.uses.delete("rsbuild:lib-entry-module");
		}
	}
};

/**
 * The `zod` dependency is bundled by Rslib. Since Rspack's public APIs
 * do not depend on `zod` types, we add `@ts-ignore` to prevent type errors
 * when users set `skipLibCheck: false` in their tsconfig.json file.
 */
const fixZodTypePlugin: RsbuildPlugin = {
	name: "fix-zod-type",
	setup(api) {
		api.onAfterBuild(async () => {
			const zodDts = path.join(api.context.distPath, "config/zod.d.ts");

			if (!fs.existsSync(zodDts)) {
				throw new Error(`Zod type file not found: ${zodDts}`);
			}

			const content = await fs.promises.readFile(zodDts, "utf-8");
			const newContent = content.replace(
				`import * as z from "zod/v4";`,
				"// @ts-ignore\nimport * as z from 'zod/v4';"
			);

			await fs.promises.writeFile(zodDts, newContent);
		});
	}
};

export default defineConfig({
	plugins: [fixZodTypePlugin],
	lib: [
		merge(commonLibConfig, {
			dts: {
				build: true
			},
			source: {
				entry: {
					index: "./src/index.ts"
				},
				tsconfigPath: "./tsconfig.build.json"
			},
			output: {
				externals: [externalAlias, "./moduleFederationDefaultRuntime.js"]
			},
			footer: {
				// make default export in cjs work
				js: "module.exports = __webpack_exports__.default;"
			}
		}),
		merge(commonLibConfig, {
			source: {
				entry: {
					cssExtractLoader: "./src/builtin-plugin/css-extract/loader.ts"
				}
			}
		}),
		merge(commonLibConfig, {
			syntax: "es2015",
			source: {
				entry: {
					cssExtractHmr: "./src/runtime/cssExtractHmr.ts"
				}
			}
		}),
		merge(commonLibConfig, {
			source: {
				entry: {
					worker: "./src/loader-runner/worker.ts"
				}
			},
			footer: {
				// make default export in cjs work
				js: "module.exports = __webpack_exports__.default;"
			}
		})
	]
});
