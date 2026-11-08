import fs from "node:fs";
import path from "node:path";

const CASE_NAME = "native-watcher-removed-file-once";
const AGGREGATE_TIMEOUT = 200;
const wait = ms => new Promise(resolve => setTimeout(resolve, ms));

// Keep initial writes outside the first build's mtime accuracy window.
function writeBeforeStart(file, content) {
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
	const past = new Date(Date.now() - 10_000);
	fs.utimesSync(file, past, past);
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should rebuild once when a removed file stays a file dependency",
	options(context) {
		const source = path.join(context.getDist(CASE_NAME), "src");
		fs.rmSync(context.getDist(CASE_NAME), { recursive: true, force: true });
		const watched = path.join(source, "watched.txt");
		writeBeforeStart(path.join(source, "index.js"), "module.exports = 1;\n");
		writeBeforeStart(watched, "watched\n");
		context.setValue("watched", watched);
		return {
			context: source,
			mode: "development",
			devtool: false,
			target: "node",
			entry: "./index.js",
			output: { path: path.join(context.getDist(CASE_NAME), "dist") },
			experiments: { nativeWatcher: true },
			plugins: [
				compiler => {
					compiler.hooks.thisCompilation.tap(CASE_NAME, compilation => {
						compilation.fileDependencies.add(watched);
					});
				}
			]
		};
	},
	async build(context, compiler) {
		let builds = 0;
		let failure;
		let watching;
		let resolveSecondBuild;
		let rejectSecondBuild;
		let secondBuildTimeout;
		let secondRebuildMs;
		const firstBuild = new Promise((resolve, reject) => {
			watching = compiler.watch(
				{ aggregateTimeout: AGGREGATE_TIMEOUT },
				(error, stats) => {
					if (error || stats.hasErrors()) {
						failure = error || new Error(stats.toString());
						reject(failure);
						rejectSecondBuild?.(failure);
						return;
					}
					builds++;
					if (builds === 2) {
						secondRebuildMs = stats.endTime - stats.startTime;
						context.setValue("removedFiles", [...compiler.removedFiles]);
						resolveSecondBuild?.();
					}
					resolve();
				}
			);
		});
		try {
			await firstBuild;
			await wait(AGGREGATE_TIMEOUT * 2);
			if (failure) throw failure;
			const secondBuild = new Promise((resolve, reject) => {
				resolveSecondBuild = resolve;
				rejectSecondBuild = reject;
				secondBuildTimeout = setTimeout(
					() => reject(new Error("Timed out waiting for the removal rebuild")),
					10_000
				);
			});
			fs.unlinkSync(context.getValue("watched"));
			await secondBuild;
			clearTimeout(secondBuildTimeout);
			await wait(Math.max(AGGREGATE_TIMEOUT * 6, 3 * secondRebuildMs));
			if (failure) throw failure;
			context.setValue("builds", builds);
		} finally {
			clearTimeout(secondBuildTimeout);
			await new Promise((resolve, reject) =>
				watching.close(error => (error ? reject(error) : resolve()))
			);
		}
	},
	async check({ context }) {
		expect(context.getValue("removedFiles")).toContain(context.getValue("watched"));
		expect(context.getValue("builds")).toBe(2);
	}
};
