import { defineConfig } from "@rslib/core";
import { pluginNodePolyfill } from "@rsbuild/plugin-node-polyfill";
import path from "node:path";

const fallbackNodeShims = path.resolve("./src/browser/fallbackNodeShims.ts");
const browserEntry = path.resolve(
	"../../crates/node_binding/rspack.wasi-browser.js"
);

export default defineConfig({
	resolve: {
		alias: {
			"@rspack/binding": browserEntry,
			"./moduleFederationDefaultRuntime.js": fallbackNodeShims,
			"./service": fallbackNodeShims,
			worker_threads: fallbackNodeShims,
			async_hooks: fallbackNodeShims,
			perf_hooks: fallbackNodeShims,
			inspector: fallbackNodeShims
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
		target: "web",
		distPath: {
			root: "../rspack-browser/dist"
		}
	},
	plugins: [pluginNodePolyfill()],
	source: {
		tsconfigPath: "./tsconfig.browser.json"
	}
});
