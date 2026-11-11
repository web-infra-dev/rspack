import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { setImmediate } from "node:timers/promises";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

export default async function run(runtimeMode, chunkLoading, scenario) {
	const context = await fs.realpath(
		await fs.mkdtemp(path.join(os.tmpdir(), "rspack-hmr-boundaries-"))
	);
	const syntax = scenario.startsWith("syntax-");
	const requestedPhase = scenario.replace(/-current$/, "");
	const phase = ["check", "prepare", "ready"].includes(requestedPhase)
		? requestedPhase
		: undefined;
	const currentVersion = scenario.endsWith("-current");
	const compiler = rspack({
		context,
		mode: "development",
		target: "node",
		entry: "./index.js",
		devtool: false,
		output: {
			path: path.join(context, "dist"),
			filename: "bundle.cjs",
			chunkFilename: "[name].cjs",
			chunkLoading,
			chunkLoadTimeout: 1000,
			library: { type: "commonjs2" },
			environment: { destructuring: false }
		},
		experiments: { runtimeMode },
		plugins: [new rspack.HotModuleReplacementPlugin()]
	});
	let modifiedFiles;
	compiler.hooks.beforeRun.tap("AsyncHmrBoundaries", () => {
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
	try {
		const multiple = scenario === "syntax-multiple";
		await build({
			"index.js": syntax
				? `
import value from "./dep";
${multiple ? 'import other from "./other";' : ""}
export const current = () => [value${multiple ? ", other" : ""}];
export const hot = module.hot;
module.hot.accept(${multiple ? '["./dep", "./other"]' : '"./dep"'}${scenario === "syntax-no-callback" ? "" : ", () => {}"});
`
				: `
require("./dep");
require("./declined");
module.hot.decline("./declined");
export const hot = module.hot;
export const errors = [];
export const accepted = [];
let pending, reached, release;
export const waiting = new Promise(resolve => { reached = resolve; });
export const resume = () => release();
export const load = () => pending || (pending = import(/* webpackChunkName: "lazy" */ "./lazy").then(m => m.default));
module.hot.addStatusHandler(status => {
	if (status === ${JSON.stringify(phase)} && !pending) {
		load();
		reached();
		return new Promise(resolve => { release = resolve; });
	}
});
module.hot.accept("./dep", () => {
	if (require("./dep") === 2 && ${JSON.stringify(scenario.includes("error"))}) throw new Error("nested accept failed");
	return load();
});
export const options = {
	ignoreErrored: ${JSON.stringify(scenario.startsWith("ignore-error"))},
	ignoreDeclined: ${JSON.stringify(scenario === "ignore-declined")},
	onErrored: error => errors.push(error),
	onAccepted: info => accepted.push(info.moduleId)
};
`,
			"dep.js": syntax
				? "export default await Promise.resolve(0);"
				: "module.exports = 0;",
			"other.js": "export default await Promise.resolve(0);",
			"declined.js": "module.exports = 0;",
			"lazy.js": "export default 0;"
		});
		const entry = await createRequire(import.meta.url)(
			path.join(context, "dist/bundle.cjs")
		);
		if (syntax) {
			const bundle = await fs.readFile(
				path.join(context, "dist/bundle.cjs"),
				"utf8"
			);
			const start = bundle.indexOf(".hot.accept(");
			const end = bundle.indexOf("__rspack_async_done()", start);
			assert(start >= 0 && end > start);
			assert.doesNotMatch(
				bundle.slice(start, end),
				/\(\[[^\]]*\]\s*=/,
				"accept wrappers must honor output.environment.destructuring"
			);
			await build({
				"dep.js": "export default await Promise.resolve(1);",
				"other.js": "export default await Promise.resolve(1);"
			});
			await entry.hot.check(true);
			assert.deepEqual(entry.current(), multiple ? [1, 1] : [1]);
		} else {
			await build({
				"dep.js": "module.exports = 1;",
				...(currentVersion ? { "lazy.js": "export default 1;" } : {})
			});
			if (!currentVersion)
				await build({
					"lazy.js": "export default 2;",
					...(phase ? {} : { "dep.js": "module.exports = 2;" }),
					...(scenario === "ignore-declined"
						? { "declined.js": "module.exports = 2;" }
						: {})
				});
			let loaded;
			const chunkRead = new Promise(resolve => {
				loaded = resolve;
			});
			globalThis.__asyncHmrChunkRead = loaded;
			const chunk = path.join(context, "dist/lazy.cjs");
			await fs.writeFile(
				chunk,
				"globalThis.__asyncHmrChunkRead();\n" +
					(await fs.readFile(chunk, "utf8"))
			);
			const update =
				phase === "ready" || scenario === "ignore-error-manual"
					? entry.hot.check().then(() => entry.hot.apply(entry.options))
					: entry.hot.check(entry.options);
			const outcome = update.then(
				ids => ({ ids }),
				error => ({ error })
			);
			if (phase) {
				await entry.waiting;
				await chunkRead;
				await setImmediate();
				if (phase === "ready" && entry.hot.status() === "prepare") {
					await new Promise(resolve => {
						const onStatus = status => {
							if (status === "ready") {
								resolve();
								queueMicrotask(() => entry.hot.removeStatusHandler(onStatus));
							}
						};
						entry.hot.addStatusHandler(onStatus);
					});
				}
				assert.ok(
					(phase === "prepare" ? ["prepare", "ready"] : [phase]).includes(
						entry.hot.status()
					)
				);
				entry.resume();
			}
			const result = await outcome;
			if (scenario === "report-error") {
				assert.match(result.error?.message || "", /nested accept failed/);
				assert.equal(entry.hot.status(), "fail");
			} else {
				assert.ifError(result.error);
				assert.ok(result.ids.length);
				assert.equal(await entry.load(), currentVersion ? 1 : 2);
				assert.equal(entry.hot.status(), "idle");
			}
			if (!phase)
				assert.equal(entry.accepted.filter(id => id === "./dep.js").length, 2);
			if (scenario.includes("error")) {
				assert.equal(entry.errors.length, 1);
				assert.equal(entry.errors[0].type, "accept-errored");
				assert.equal(entry.errors[0].error.message, "nested accept failed");
			}
		}
	} finally {
		delete globalThis.__asyncHmrChunkRead;
		await closeCompiler(compiler);
		await fs.rm(context, { recursive: true, force: true });
	}
}
