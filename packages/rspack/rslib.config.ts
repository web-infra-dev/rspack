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

export default defineConfig({
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
