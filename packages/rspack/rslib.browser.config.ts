import { defineConfig } from "@rslib/core";
import * as rspack from "@rspack/core";
import { pluginNodePolyfill } from "@rsbuild/plugin-node-polyfill";
import path from "node:path";

const serviceShim = path.resolve("./src/browser/service.ts");

const bindingDir = path.resolve("../../crates/node_binding");
const browserEntry = path.join(bindingDir, "rspack.wasi-browser.js");

export default defineConfig({
	resolve: {
		alias: {
			"@rspack/binding": browserEntry,
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
			autoExternal: false,
			shims: {
				esm: {
					__dirname: true
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
		externals: ["@napi-rs/wasm-runtime"],
		copy: {
			patterns: [
				{
					from: path.join(bindingDir, "wasi-worker-browser.mjs"),
					to: "wasi-worker-browser.mjs"
				},
				{
					from: path.join(bindingDir, "rspack.wasm32-wasi.wasm"),
					to: "rspack.wasm32-wasi.wasm"
				}
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
						/(moduleFederationDefaultRuntime|css-extract|worker_threads|async_hooks|perf_hooks|inspector)/
				})
			);
		}
	}
});
