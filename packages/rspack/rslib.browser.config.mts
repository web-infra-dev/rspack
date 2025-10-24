import fs from "node:fs/promises";
import path from "node:path";
import vm from "node:vm";
import { pluginNodePolyfill } from "@rsbuild/plugin-node-polyfill";
import { defineConfig, type rsbuild, rspack } from "@rslib/core";

const bindingDir = path.resolve("../../crates/node_binding");
const distDir = path.resolve("../rspack-browser/dist");

const MF_RUNTIME_CODE = await getModuleFederationRuntimeCode();

export default defineConfig({
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
		},
		{
			format: "esm",
			syntax: "es2021",
			dts: false,
			autoExtension: false,
			source: {
				entry: {
					worker: {
						import: path.resolve(bindingDir, "./wasi-worker-browser.mjs"),
						filename: "wasi-worker-browser.mjs"
					}
				}
			}
		}
	],
	output: {
		cleanDistPath: true,
		target: "web",
		distPath: {
			root: distDir
		},
		externals: [
			"@rspack/lite-tapable",
			{
				"@rspack/binding": "./rspack.wasi-browser.js"
			}
		],
		copy: {
			patterns: [
				path.resolve(bindingDir, "napi-binding.d.ts"),
				path.resolve(bindingDir, "binding.d.ts"),
				{
					from: path.resolve(bindingDir, "rspack.browser.wasm"),
					to: "rspack.wasm32-wasi.wasm",
					noErrorOnMissing: true
				},
				// For CI
				{
					from: path.resolve(
						"../../artifacts/bindings-wasm32-wasip1-threads/rspack.browser.wasm"
					),
					to: "rspack.wasm32-wasi.wasm",
					noErrorOnMissing: true
				}
			]
		}
	},
	plugins: [
		pluginNodePolyfill({
			overrides: {
				fs: path.resolve("./src/browser/fs"),
				buffer: path.resolve("./src/browser/buffer")
			}
		}),
		replaceDtsPlugin(),
		copyRspackBrowserRuntimePlugin()
	],
	source: {
		tsconfigPath: "./tsconfig.browser.json",
		define: {
			WEBPACK_VERSION: JSON.stringify(require("./package.json").webpackVersion),
			RSPACK_VERSION: JSON.stringify(require("./package.json").version),
			IS_BROWSER: JSON.stringify(true),
			// In `@rspack/browser`, runtime code like loaders and hmr should be written into something like memfs ahead of time.
			// Requiring these files should resolve to `@rspack/browser/xx`
			__dirname: JSON.stringify("@rspack/browser"),
			// Runtime code
			MF_RUNTIME_CODE: JSON.stringify(MF_RUNTIME_CODE)
		}
	},
	tools: {
		bundlerChain: (chain, { CHAIN_ID }) => {
			// remove the entry loader in Rslib to avoid
			// "Cannot access 'Compiler' before initialization" error caused by circular dependency
			chain.module
				.rule(`Rslib:${CHAIN_ID.RULE.JS}-entry-loader`)
				.uses.delete("rsbuild:lib-entry-module");
		},
		rspack: (config, { rspack }) => {
			config.plugins.push(
				new rspack.IgnorePlugin({
					resourceRegExp: /(moduleFederationDefaultRuntime|inspector)/
				}),
				new rspack.NormalModuleReplacementPlugin(
					/src[/\\]loader-runner[/\\]service\.ts/,
					path.resolve("./src/browser/service.ts")
				)
			);
		}
	}
});

function copyRspackBrowserRuntimePlugin(): rsbuild.RsbuildPlugin {
	return {
		name: "copy-rspack-browser-runtime-plugin",
		setup(api) {
			api.onAfterBuild(async () => {
				const runtimeCode = await fs.readFile(
					path.resolve(bindingDir, "rspack.wasi-browser.js")
				);
				const workerUrl =
					// biome-ignore lint/suspicious/noTemplateCurlyInString: we need to escape "${}"
					"${new URL('./wasi-worker-browser.mjs', import.meta.url)}";
				const modifiedRuntimeCode = runtimeCode
					.toString()
					.replaceAll(`type: 'module'`, `type: 'classic'`)
					.replaceAll(
						`new URL('./wasi-worker-browser.mjs', import.meta.url)`,
						`URL.createObjectURL(new Blob([\`importScripts("${workerUrl}")\`], { type: 'text/javascript' }))`
					)
					.replaceAll(
						"const __wasmUrl =",
						"const __wasmUrl = window.RSPACK_WASM_URL ||"
					);

				await fs.writeFile(
					path.resolve(distDir, "./rspack.wasi-browser.js"),
					modifiedRuntimeCode
				);
			});
		}
	};
}

/**
 * Since `@rspack/browser` doesn't depend on `@rspack/binding`, we should directly bundle the type declarations to it.
 * This plugin will replace the usages of `@rspack/binding` to the relative dts path in the generated .d.ts files.
 * The `binding.d.ts` and the `napi.binding.d.ts` will be copied to the output directory with RspackCopyPlugin.
 *
 * The reason that we don't use `paths` in `tsconfig.json` is that it can't rewrite the module idents in `declare module`,
 * so we decided to simply replace all instances of it.
 */
function replaceDtsPlugin(): rsbuild.RsbuildPlugin {
	return {
		name: "replace-dts-plugin",
		setup(api) {
			api.onAfterBuild(async () => {
				const outFiles = await fs.readdir(distDir, { recursive: true });
				for (const file of outFiles) {
					// Filter *.d.ts
					if (!file.endsWith(".d.ts")) {
						continue;
					}
					const filePath = path.join(distDir, file);

					const dts = (await fs.readFile(filePath)).toString();
					let relativeBindingDts = path.relative(
						path.dirname(filePath),
						path.join(distDir, "binding")
					);

					// Ensure relative path starts with "./"
					if (!relativeBindingDts.startsWith("../")) {
						relativeBindingDts = `./${relativeBindingDts}`;
					}

					// There are three cases that @rspack/binding may be used
					// 1. import("@rspack/binding").XXX
					// 2. import { XX } from "@rspack/binding"
					// 3. declare module "@rspack/binding" { XX }
					const replacedDts = dts
						.replaceAll(
							'import("@rspack/binding")',
							`import("${relativeBindingDts}")`
						)
						.replaceAll(
							'from "@rspack/binding"',
							`from "${relativeBindingDts}"`
						)
						.replaceAll(
							'declare module "@rspack/binding"',
							`declare module "${relativeBindingDts}"`
						);
					await fs.writeFile(filePath, replacedDts);
				}
			});
		}
	};
}

async function getModuleFederationRuntimeCode() {
	const { swc } = rspack.experiments;
	const runtime = await fs.readFile(
		path.resolve(__dirname, "src/runtime/moduleFederationDefaultRuntime.js"),
		"utf-8"
	);

	const { code: downgradedRuntime } = await swc.transform(runtime, {
		jsc: {
			target: "es2015"
		}
	});

	const minimizedRuntime = await swc.minify(downgradedRuntime, {
		compress: false,
		mangle: false,
		ecma: 2015
	});

	const sandbox = { module: { exports: undefined } } as any;
	vm.createContext(sandbox);
	vm.runInContext(minimizedRuntime.code, sandbox);

	const functionContent = rspack.Template.getFunctionContent(
		sandbox.module.exports
	);
	return functionContent;
}
