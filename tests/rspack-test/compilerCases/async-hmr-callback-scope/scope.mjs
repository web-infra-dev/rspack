import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";

export default async function run(runtimeMode, asyncDependency, scenario) {
	const context = await fs.realpath(
		await fs.mkdtemp(path.join(os.tmpdir(), "rspack-hmr-scope-"))
	);
	const compiler = rspack({
		context,
		mode: "development",
		target: "node",
		entry: "./index.js",
		devtool: false,
		module: { rules: [{ test: /\.js$/, type: "javascript/esm" }] },
		output: {
			path: path.join(context, "dist"),
			filename: "bundle.cjs",
			environment: { arrowFunction: scenario !== "fallback" },
			library: { type: "commonjs2" }
		},
		experiments: { runtimeMode },
		plugins: [new rspack.HotModuleReplacementPlugin()]
	});
	let modifiedFiles;
	compiler.hooks.beforeRun.tap("CallbackScope", () => {
		compiler.modifiedFiles = modifiedFiles;
	});
	const build = async changes => {
		modifiedFiles = new Set();
		for (const [name, source] of Object.entries(changes)) {
			const file = path.join(context, name);
			await fs.writeFile(file, source);
			modifiedFiles.add(file);
		}
		compiler.inputFileSystem.purge();
		const stats = await runCompiler(compiler);
		assert(!stats.hasErrors(), stats.toString({ all: false, errors: true }));
	};
	const dependency = value => `export default ${asyncDependency ? `await Promise.resolve(${value})` : value};`;
	const callback = expression => `import.meta.webpackHot.accept("./dep", async () => {
	await Promise.resolve();
	values.push([value, ${expression}]);
});`;
	const registrations = {
		arguments: `function register(token) { ${callback("arguments[0]")} } register("token");`,
		"new-target": `function Token() { ${callback("new.target.name")} } new Token();`,
		super: `class Base { label() { return "token"; } }
class Register extends Base { constructor() { super(); ${callback("super.label()")} } }
new Register();`,
		fallback: `function register() { ${callback("this.label")} } register.call({ label: "token" });`
	};
	try {
		await build({
			"index.js": `import value from "./dep";
export const hot = import.meta.webpackHot;
export const values = [];
${registrations[scenario]}`,
			"dep.js": dependency(0)
		});
		const entry = await createRequire(import.meta.url)(path.join(context, "dist/bundle.cjs"));
		for (const value of [1, 2]) {
			await build({ "dep.js": dependency(value) });
			await entry.hot.check(true);
			assert.equal(entry.hot._disposeHandlers.length, 0);
		}
		const label = scenario === "new-target" ? "Token" : "token";
		assert.deepEqual(entry.values, [[1, label], [2, label]]);
		assert.equal(entry.hot.status(), "idle");
	} finally {
		await closeCompiler(compiler);
		await fs.rm(context, { recursive: true, force: true });
	}
}
