import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { promisify } from "node:util";
import rspack from "@rspack/core";

const [phase, root] = process.argv.slice(2);
const sourceDirectory = path.join(root, "src");
const valueFile = path.join(sourceDirectory, "value.js");
const outputFile = path.join(root, "dist", "main.js");
const initialValue = "initial value";
const updatedValue = "updated value from watch";

if (phase === "warm") {
	fs.mkdirSync(sourceDirectory, { recursive: true });
	fs.writeFileSync(path.join(sourceDirectory, "index.js"),
		'import value from "./value.js";\nconsole.log(value);\n');
	fs.writeFileSync(valueFile, `export default ${JSON.stringify(initialValue)};\n`);
}

const compiler = rspack({
	context: sourceDirectory,
	mode: "development",
	devtool: false,
	target: "node",
	entry: "./index.js",
	experiments: { newCache: true },
	cache: {
		type: "persistent",
		portable: false,
		version: "persistent-cache-watch",
		storage: { type: "filesystem", directory: path.join(root, "cache") }
	},
	output: { path: path.dirname(outputFile), filename: "main.js", clean: true }
});

function checkStats(stats) {
	assert.ok(stats, "Watch build returned no stats");
	assert.equal(stats.hasErrors(), false,
		stats.toString({ all: false, errors: true, errorDetails: true }));
}

try {
	if (phase === "warm") {
		checkStats(await promisify(compiler.run.bind(compiler))());
	} else {
		assert.equal(phase, "watch");
		await new Promise((resolve, reject) => {
			let watching;
			let changeTimer;
			let settled = false;
			let buildCount = 0;
			const timeout = setTimeout(
				() => finish(new Error("Timed out waiting for the restored watcher to rebuild")),
				process.platform === "win32" ? 15_000 : 10_000
			);
			const finish = error => {
				if (settled) return;
				settled = true;
				clearTimeout(timeout);
				clearTimeout(changeTimer);
				const done = closeError => {
					const finalError = error || closeError;
					if (finalError) reject(finalError);
					else resolve();
				};
				if (watching) watching.close(done);
				else done();
			};
			watching = compiler.watch({ aggregateTimeout: 50, ignored: [] }, (error, stats) => {
				if (settled) return;
				try {
					if (error) return finish(error);
					checkStats(stats);
					buildCount++;
					const output = fs.readFileSync(outputFile, "utf8");
					if (buildCount === 1) {
						const modules = stats.toJson({ all: false, cacheInfo: true })
							.cacheInfo["|Compilation/modules"];
						assert.equal(modules.reused, 2, "Both modules must be restored from disk");
						assert.ok([...stats.compilation.fileDependencies].includes(valueFile),
							"The restored watch dependency must retain its native absolute path");
						assert.ok(output.includes(initialValue));
						// Allow native directory watchers to register, especially on Windows CI.
						changeTimer = setTimeout(() => {
							try {
								fs.writeFileSync(valueFile, `export default ${JSON.stringify(updatedValue)};\n`);
							} catch (writeError) {
								finish(writeError);
							}
						}, process.platform === "win32" ? 500 : 100);
					} else {
						assert.equal(buildCount, 2);
						assert.ok(output.includes(updatedValue));
						finish();
					}
				} catch (callbackError) {
					finish(callbackError);
				}
			});
		});
	}
} finally {
	await promisify(compiler.close.bind(compiler))();
}
