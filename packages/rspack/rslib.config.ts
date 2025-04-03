import { type LibConfig, defineConfig, rspack } from "@rslib/core";
import prebundleConfig from "./prebundle.config.mjs";

const externalAlias = ({ request }: { request?: string }, callback) => {
	const { dependencies } = prebundleConfig;

	for (const item of dependencies) {
		const depName = typeof item === "string" ? item : item.name;
		if (new RegExp(`^${depName}$`).test(request!)) {
			return callback(null, `../compiled/${depName}/index.js`);
		}
	}

	if (/..\/package\.json/.test(request!)) {
		return callback(null, "../package.json");
	}

	return callback();
};

const commonLibConfig: LibConfig = {
	format: "cjs",
	syntax: ["node 16"],
	output: {
		externals: [externalAlias]
	}
};

export default defineConfig({
	lib: [
		{
			...commonLibConfig,
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
				...commonLibConfig.output,
				minify: false,
				externals: [externalAlias, "./moduleFederationDefaultRuntime.js"]
			},
			footer: {
				js: "module.exports = __webpack_exports__.default;"
			},
			tools: {
				rspack: {
					// optimization: {
					// 	concatenateModules: false
					// }
					// plugins: [new rspack.CircularDependencyRspackPlugin({})]
				}
			}
		},
		{
			...commonLibConfig,
			source: {
				entry: {
					cssExtractLoader: "./src/builtin-plugin/css-extract/loader.ts"
				}
			}
		},
		{
			...commonLibConfig,
			syntax: "es2015",
			source: {
				entry: {
					cssExtractHmr: "./src/runtime/cssExtractHmr.ts"
				}
			}
		},
		{
			...commonLibConfig,
			source: {
				entry: {
					worker: "./src/loader-runner/worker.ts"
				}
			}
		}
	]
});
