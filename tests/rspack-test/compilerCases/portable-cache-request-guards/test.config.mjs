import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: "should preserve plugin externals and ignores when loader request rebasing is requested",
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
				ignorePlugin: () => ({
					plugins: [{ apply(compiler) {
						new rspack.IgnorePlugin({ resourceRegExp: /^(?:\/|[A-Z]:[\\/]).*node_modules[\\/]api[\\/]index\.js$/i }).apply(compiler);
					} }]
				})
			};
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
							name, rebaseLoaderRequests, value: name === "ignorePlugin" ? "ignored" : "api"
						});
						const warnings = (stats.toJson({ all: false, logging: "warn" }).logging?.["rspack.persistentCache"]?.entries ?? [])
							.map(entry => entry.message).filter(message => message.includes("rebaseLoaderRequests"));
						expect(warnings).toHaveLength(rebaseLoaderRequests ? 1 : 0);
						if (rebaseLoaderRequests) expect(warnings[0]).toContain("disabled");
					} finally {
						await closeCompiler(compiler);
					}
				}
			}
		} finally {
			fs.rmSync(root, { recursive: true, force: true });
		}
	}
};
