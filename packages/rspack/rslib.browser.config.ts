import path from "node:path";
import { pluginNodePolyfill } from "@rsbuild/plugin-node-polyfill";
import { defineConfig } from "@rslib/core";

const serviceShim = path.resolve("./src/browser/service.ts");

const bindingDir = path.resolve("../../crates/node_binding");

export default defineConfig({
	resolve: {
		alias: {
			"graceful-fs": "node:fs",
			"./service": serviceShim
			// "./moduleFederationDefaultRuntime.js": fallbackNodeShims,
			// worker_threads: fallbackNodeShims,
			// async_hooks: fallbackNodeShims,
			// perf_hooks: fallbackNodeShims,
			// inspector: fallbackNodeShims
		}
	},
	lib: [
		{
			format: "esm",
			syntax: "es2021",
			dts: { build: true },
			autoExternal: false
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
			{
				"@rspack/binding": "./rspack.wasi-browser.js"
			}
		],
		copy: {
			patterns: [
				path.join(bindingDir, "rspack.wasi-browser.js"),
				path.join(bindingDir, "wasi-worker-browser.mjs"),
				path.join(bindingDir, "rspack.wasm32-wasi.wasm")
			]
		}
	},
	plugins: [pluginNodePolyfill()],
	source: {
		tsconfigPath: "./tsconfig.browser.json",
		define: {
			WEBPACK_VERSION: JSON.stringify(require("./package.json").webpackVersion),
			RSPACK_VERSION: JSON.stringify(require("./package.json").version)
		}
	},
	tools: {
		rspack: (config, { rspack }) => {
			config.plugins.push(
				new rspack.IgnorePlugin({
					resourceRegExp:
						/(moduleFederationDefaultRuntime|worker_threads|async_hooks|perf_hooks|inspector)/
				})
			);
		}
	}
});
