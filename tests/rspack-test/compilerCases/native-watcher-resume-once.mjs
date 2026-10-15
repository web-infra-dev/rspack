import fs from "node:fs";
import path from "node:path";

const CASE_NAME = "native-watcher-resume-once";

function write(file, content) {
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
}

// Sources written right before the first build fall within its start time
// once padded by the mtime accuracy, so the first watch would report them as
// changed and rebuild right away.
function writeBeforeStart(file, content) {
	write(file, content);
	const past = new Date(Date.now() - 10_000);
	fs.utimesSync(file, past, past);
}

const wait = ms => new Promise(resolve => setTimeout(resolve, ms));

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should not rebuild again for an edit made while suspended once resume has built it",
	options(context) {
		const source = path.join(context.getDist(CASE_NAME), "src");
		fs.rmSync(context.getDist(CASE_NAME), { recursive: true, force: true });
		writeBeforeStart(path.join(source, "index.js"), 'import b from "./b.js";\nconsole.log(1, b);\n');
		writeBeforeStart(path.join(source, "b.js"), "export default 1;\n");
		context.setValue("source", source);
		return {
			context: source,
			mode: "development",
			devtool: false,
			target: "node",
			entry: "./index.js",
			output: { path: path.join(context.getDist(CASE_NAME), "dist") },
			experiments: { nativeWatcher: true }
		};
	},
	async build(context, compiler) {
		const source = context.getValue("source");
		let builds = 0;
		let watching;
		const firstBuild = new Promise((resolve, reject) => {
			watching = compiler.watch({ aggregateTimeout: 100 }, (error, stats) => {
				if (error) return reject(error);
				if (stats.hasErrors()) return reject(new Error(stats.toString()));
				builds++;
				resolve();
			});
		});
		try {
			await firstBuild;
			await wait(300);

			// The first edit aggregates while suspended, pausing the watcher; the
			// second lands while paused and stays pending until resume folds it
			// into the build it starts.
			watching.suspend();
			write(path.join(source, "index.js"), 'import b from "./b.js";\nconsole.log(2, b);\n');
			await wait(600);
			write(path.join(source, "b.js"), "export default 2;\n");
			await wait(600);
			watching.resume();

			await wait(1500);
			context.setValue("builds", builds);
		} finally {
			await new Promise((resolve, reject) =>
				watching.close(error => (error ? reject(error) : resolve()))
			);
		}
	},
	async check({ context }) {
		expect(context.getValue("builds")).toBe(2);
	}
};
