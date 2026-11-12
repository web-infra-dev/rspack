import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";
import { removeTree } from "../_remove-tree.mjs";

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: "should preserve plugin externals and ignores when loader request rebasing is requested, and keep rebasing next to an ignore pattern that reads both spellings alike",
	async run() {
		const root = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-request-guards-"))
		);
		try {
			fs.mkdirSync(path.join(root, ".git"));
			fs.mkdirSync(path.join(root, "src"));
			fs.mkdirSync(path.join(root, "node_modules/api"), { recursive: true });
			fs.writeFileSync(path.join(root, "node_modules/api/index.js"), 'module.exports = "api";');
			fs.writeFileSync(path.join(root, "src/req.gen"), "");
			fs.writeFileSync(path.join(root, "src/index.js"), `
try { module.exports = require("./req.gen"); }
catch { module.exports = "ignored"; }
`);
			fs.writeFileSync(path.join(root, "loader.cjs"), `
const path = require("node:path");
module.exports = function () {
	return "var api = require(" + JSON.stringify(path.join(this.rootContext, "node_modules/api/index.js")) + "); module.exports = api;";
};
`);
			const absoluteApi = /^(?:\/|[A-Z]:[\\/]).*node_modules[\\/]api[\\/]index\.js$/i;
			const external = ({ request }, callback) => {
				if (/node_modules[\\/]api/.test(request)) callback(null, `commonjs ${request}`);
				else callback();
			};
			const variants = {
				externalsPlugin: () => ({
					plugins: [{ apply(compiler) { new rspack.ExternalsPlugin("commonjs", external).apply(compiler); } }]
				}),
				externalsOption: () => ({ externals: external }),
				externalsObject: () => ({ externals: { unused: "Unused" } }),
				// The pattern matches the absolute spelling only, so this request keeps it.
				ignorePlugin: () => ({
					plugins: [{ apply(compiler) {
						new rspack.IgnorePlugin({ resourceRegExp: absoluteApi }).apply(compiler);
					} }]
				}),
				// A function can't be compared across spellings, so it turns rebasing off.
				ignoreCheckResource: () => ({
					plugins: [new rspack.IgnorePlugin({ checkResource: resource => absoluteApi.test(resource) })]
				}),
				// A pattern that reads both spellings alike leaves rebasing on.
				ignoreOtherRequest: () => ({
					plugins: [new rspack.IgnorePlugin({ resourceRegExp: /moment/, contextRegExp: /pikaday/ })]
				})
			};
			const ignored = ["ignorePlugin", "ignoreCheckResource"];
			const disabled = ["externalsPlugin", "externalsOption", "externalsObject", "ignoreCheckResource"];
			for (const [name, configure] of Object.entries(variants)) {
				for (const rebaseLoaderRequests of [false, true]) {
					const compiler = rspack({
						context: root,
						mode: "development",
						devtool: false,
						target: "node",
						entry: "./src/index.js",
						output: {
							path: path.join(root, "build/server"),
							library: { type: "commonjs2" }
						},
						cache: {
							type: "persistent",
							portable: { rebaseLoaderRequests },
							storage: { type: "filesystem", location: path.join(root, ".cache", name, String(rebaseLoaderRequests)) }
						},
						module: {
							rules: [{ test: /\.gen$/, type: "javascript/auto", use: [path.join(root, "loader.cjs")] }]
						},
						...configure()
					});
					try {
						const stats = await runCompiler(compiler);
						expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
						const filename = path.join(root, "build/server/main.js");
						const runtime = { exports: {} };
						new Function("module", "exports", "require", fs.readFileSync(filename, "utf8"))(
							runtime, runtime.exports, createRequire(filename)
						);
						expect({ name, rebaseLoaderRequests, value: runtime.exports }).toEqual({
							name, rebaseLoaderRequests, value: ignored.includes(name) ? "ignored" : "api"
						});
						const warnings = (stats.toJson({ all: false, logging: "warn" }).logging?.["rspack.persistentCache"]?.entries ?? [])
							.map(entry => entry.message).filter(message => message.includes("rebaseLoaderRequests"));
						expect({ name, warnings: warnings.length }).toEqual({
							name, warnings: rebaseLoaderRequests && disabled.includes(name) ? 1 : 0
						});
						if (warnings.length) expect(warnings[0]).toContain("disabled");
						if (name === "ignoreOtherRequest") {
							// The request the loader wrote is spelled relative to the module only when rebased.
							const requests = stats.toJson({ all: false, modules: true, reasons: true }).modules
								.find(module => module.name.includes("node_modules/api/index.js"))
								.reasons.filter(reason => reason.type === "cjs require")
								.map(reason => path.isAbsolute(reason.userRequest));
							expect({ rebaseLoaderRequests, requests }).toEqual({
								rebaseLoaderRequests, requests: [!rebaseLoaderRequests]
							});
						}
					} finally {
						await closeCompiler(compiler);
					}
				}
			}
		} finally {
			removeTree(root);
		}
	}
};
