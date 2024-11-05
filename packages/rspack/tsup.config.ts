import { type Options, defineConfig } from "tsup";
import prebundleConfig from "./prebundle.config.mjs";

const aliasPlugin = {
	name: "alias-compiled-plugin",
	setup(build) {
		const { dependencies } = prebundleConfig;

		for (const item of dependencies) {
			const depName = typeof item === "string" ? item : item.name;
			build.onResolve({ filter: new RegExp(`^${depName}$`) }, () => ({
				path: `../compiled/${depName}/index.js`,
				external: true
			}));
		}

		build.onResolve({ filter: /..\/package\.json/ }, () => ({
			path: "../package.json",
			external: true
		}));
	}
};

const commonConfig: Options = {
	format: ["cjs"],
	target: "node16",
	esbuildPlugins: [aliasPlugin]
};

export default defineConfig([
	{
		...commonConfig,
		entry: ["./src/index.ts"],
		external: ["./moduleFederationDefaultRuntime.js"]
	},
	{
		...commonConfig,
		entry: {
			cssExtractLoader: "./src/builtin-plugin/css-extract/loader.ts"
		}
	},
	{
		...commonConfig,
		entry: {
			cssExtractHmr: "./src/runtime/cssExtractHmr.ts"
		},
		target: ["es2015"]
	}
]);
