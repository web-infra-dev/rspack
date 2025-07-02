import path from "node:path";
import { pluginNodePolyfill } from "@rsbuild/plugin-node-polyfill";
import { defineConfig } from "@rslib/core";

const bindingDir = path.resolve("../../crates/node_binding");

export default defineConfig({
	resolve: {
		alias: {
			"graceful-fs": "node:fs"
		}
	},
	lib: [
		{
			format: "esm",
			syntax: "es2021",
			dts: { build: true },
			autoExternal: false,
			source: {
				entry: {
					index: "./src/browser/index.ts"
				}
			}
		}
	],
	output: {
		cleanDistPath: true,
		target: "web",
		distPath: {
			root: "../rspack-browser/dist"
		},
		externals: [
			"@napi-rs/wasm-runtime",
			"@rspack/lite-tapable",
			{
				"@rspack/binding": "./rspack.wasi-browser.js"
			}
		],
		copy: {
			patterns: [
				path.resolve(bindingDir, "rspack.wasi-browser.js"),
				path.resolve(bindingDir, "wasi-worker-browser.mjs"),
				path.resolve(bindingDir, "rspack.wasm32-wasi.wasm")
			]
		}
	},
	plugins: [
		pluginNodePolyfill({
			overrides: {
				fs: path.resolve("./src/browser/fs")
			}
		})
	],
	source: {
		tsconfigPath: "./tsconfig.browser.json",
		define: {
			WEBPACK_VERSION: JSON.stringify(require("./package.json").webpackVersion),
			RSPACK_VERSION: JSON.stringify(require("./package.json").version),
			IS_BROWSER: JSON.stringify(true)
		}
	},
	tools: {
		rspack: (config, { rspack }) => {
			config.plugins.push(
				new rspack.IgnorePlugin({
					resourceRegExp: /(moduleFederationDefaultRuntime|inspector)/
				}),
				new rspack.NormalModuleReplacementPlugin(
					/src\/loader-runner\/service\.ts/,
					path.resolve("./src/browser/service.ts")
				),
				new rspack.NormalModuleReplacementPlugin(
					/src\/index\.ts/,
					path.resolve("./src/browser/index.ts")
				)
			);
		}
	}
});
